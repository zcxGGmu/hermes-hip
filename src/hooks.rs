use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const HOOK_NAME: &str = "hermeship";
const MANAGED_MARKER_FILE: &str = ".hermeship-managed.json";
const DEFAULT_HERMESHIP_BIN_PLACEHOLDER: &str = "__HERMESHIP_BIN__";
pub const HOOK_MANIFEST_TEMPLATE: &str = include_str!("../templates/hermes-hook/HOOK.yaml");
pub const HOOK_HANDLER_TEMPLATE: &str = include_str!("../templates/hermes-hook/handler.py");
const HERMES_HOME_ENV_VAR: &str = "HERMES_HOME";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookInstallOptions {
    pub hermes_home: PathBuf,
    pub hermeship_bin: Option<PathBuf>,
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookInstallReport {
    pub hook_dir: PathBuf,
    pub planned_files: Vec<PathBuf>,
    pub written_files: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookUninstallReport {
    pub hook_dir: PathBuf,
    pub planned_paths: Vec<PathBuf>,
    pub removed_paths: Vec<PathBuf>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ManagedHooksMarker {
    version: u32,
    files: Vec<ManagedHookFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ManagedHookFile {
    path: String,
    checksum: String,
}

pub fn default_hermes_home() -> PathBuf {
    default_hermes_home_with_env(|name| env::var(name).ok())
}

fn default_hermes_home_with_env<F>(mut get_env: F) -> PathBuf
where
    F: FnMut(&str) -> Option<String>,
{
    if let Some(home) = normalize_path(get_env(HERMES_HOME_ENV_VAR)) {
        return home;
    }

    normalize_path(get_env("HOME"))
        .or_else(|| normalize_path(get_env("USERPROFILE")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hermes")
}

pub fn install_hermes_hooks(options: &HookInstallOptions) -> Result<HookInstallReport> {
    let hook_dir = hermes_hook_dir(&options.hermes_home);
    let files = managed_hook_files(&hook_dir, options.hermeship_bin.as_deref())?;
    let planned_files = files
        .iter()
        .map(|(path, _, _)| path.clone())
        .chain(std::iter::once(hook_dir.join(MANAGED_MARKER_FILE)))
        .collect::<Vec<_>>();
    let mut report = HookInstallReport {
        hook_dir,
        planned_files,
        written_files: Vec::new(),
        skipped_files: Vec::new(),
        dry_run: options.dry_run,
    };

    if options.dry_run {
        return Ok(report);
    }

    fs::create_dir_all(&report.hook_dir).with_context(|| {
        format!(
            "failed to create Hermes hook directory {}",
            report.hook_dir.display()
        )
    })?;

    let mut managed_entries = Vec::new();

    for (path, content, marker) in files {
        if path.exists() && !options.force {
            report.skipped_files.push(path.clone());
            if fs::read_to_string(&path).ok().as_deref() == Some(content.as_str()) {
                managed_entries.push(marker);
            }
            continue;
        }

        fs::write(&path, content.as_bytes())
            .with_context(|| format!("failed to write Hermes hook file {}", path.display()))?;
        set_executable_if_handler(&path)?;
        managed_entries.push(marker);
        report.written_files.push(path);
    }

    if !managed_entries.is_empty() {
        let marker_path = report.hook_dir.join(MANAGED_MARKER_FILE);
        let marker = ManagedHooksMarker {
            version: 1,
            files: managed_entries,
        };
        let marker_json = serde_json::to_string_pretty(&marker)
            .context("failed to serialize Hermes hook marker")?;
        fs::write(&marker_path, marker_json).with_context(|| {
            format!(
                "failed to write Hermes hook marker {}",
                marker_path.display()
            )
        })?;
        report.written_files.push(marker_path);
    }

    Ok(report)
}

pub fn uninstall_hermes_hooks(
    hermes_home: impl AsRef<Path>,
    dry_run: bool,
) -> Result<HookUninstallReport> {
    let hook_dir = hermes_hook_dir(hermes_home.as_ref());
    let mut report = HookUninstallReport {
        hook_dir: hook_dir.clone(),
        planned_paths: vec![hook_dir.clone()],
        removed_paths: Vec::new(),
        dry_run,
    };

    if dry_run {
        return Ok(report);
    }

    if hook_dir.exists() {
        if !hook_dir.is_dir() {
            return Ok(report);
        }

        let marker_path = hook_dir.join(MANAGED_MARKER_FILE);
        let managed_entries = if marker_path.exists() {
            read_marker(&marker_path)?.files
        } else {
            Vec::new()
        };

        for managed in managed_entries {
            let Some(path) = managed_file_path(&hook_dir, &managed.path) else {
                continue;
            };
            if !path.exists() {
                continue;
            }

            let current = fs::read_to_string(&path)
                .with_context(|| format!("failed to read Hermes hook file {}", path.display()))?;
            if checksum(&current) == managed.checksum {
                fs::remove_file(&path).with_context(|| {
                    format!("failed to remove Hermes hook file {}", path.display())
                })?;
                report.removed_paths.push(path);
            }
        }

        if marker_path.exists() {
            let has_remaining_entries = fs::read_dir(&hook_dir)
                .with_context(|| {
                    format!(
                        "failed to read Hermes hook directory {}",
                        hook_dir.display()
                    )
                })?
                .filter_map(|entry| entry.ok())
                .any(|entry| entry.file_name() != MANAGED_MARKER_FILE);

            if !has_remaining_entries {
                fs::remove_file(&marker_path).with_context(|| {
                    format!(
                        "failed to remove Hermes hook marker {}",
                        marker_path.display()
                    )
                })?;
                report.removed_paths.push(marker_path);
                fs::remove_dir(&hook_dir).with_context(|| {
                    format!(
                        "failed to remove empty Hermes hook directory {}",
                        hook_dir.display()
                    )
                })?;
                report.removed_paths.push(hook_dir);
            }
        }
    }

    Ok(report)
}

fn hermes_hook_dir(hermes_home: impl AsRef<Path>) -> PathBuf {
    hermes_home.as_ref().join("hooks").join(HOOK_NAME)
}

fn managed_hook_files(
    hook_dir: &Path,
    hermeship_bin: Option<&Path>,
) -> Result<Vec<(PathBuf, String, ManagedHookFile)>> {
    let handler = render_hook_handler(hermeship_bin)?;
    Ok(vec![
        (
            hook_dir.join("HOOK.yaml"),
            HOOK_MANIFEST_TEMPLATE.to_string(),
            ManagedHookFile {
                path: "HOOK.yaml".to_string(),
                checksum: checksum(HOOK_MANIFEST_TEMPLATE),
            },
        ),
        (
            hook_dir.join("handler.py"),
            handler.clone(),
            ManagedHookFile {
                path: "handler.py".to_string(),
                checksum: checksum(&handler),
            },
        ),
    ])
}

fn render_hook_handler(hermeship_bin: Option<&Path>) -> Result<String> {
    let rendered_bin = match hermeship_bin {
        Some(path) => path.display().to_string(),
        None => default_hermeship_binary()?,
    };

    let literal = serde_json::to_string(&rendered_bin).context("failed to encode binary path")?;
    Ok(HOOK_HANDLER_TEMPLATE.replace(
        &format!("\"{DEFAULT_HERMESHIP_BIN_PLACEHOLDER}\""),
        &literal,
    ))
}

fn default_hermeship_binary() -> Result<String> {
    Ok(std::env::current_exe()
        .context("failed to resolve current hermeship binary")?
        .display()
        .to_string())
}

fn normalize_path(value: Option<String>) -> Option<PathBuf> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
    })
}

fn set_executable_if_handler(path: &Path) -> Result<()> {
    if path.file_name().and_then(|value| value.to_str()) != Some("handler.py") {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .with_context(|| format!("failed to stat Hermes hook file {}", path.display()))?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("failed to chmod Hermes hook file {}", path.display()))?;
    }

    Ok(())
}

fn read_marker(path: &Path) -> Result<ManagedHooksMarker> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read Hermes hook marker {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse Hermes hook marker {}", path.display()))
}

fn managed_file_path(hook_dir: &Path, relative_path: &str) -> Option<PathBuf> {
    let mut components = Path::new(relative_path).components();
    let Some(Component::Normal(file_name)) = components.next() else {
        return None;
    };
    if components.next().is_some() {
        return None;
    }
    Some(hook_dir.join(file_name))
}

fn checksum(content: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3_u64);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::{
        HOOK_HANDLER_TEMPLATE, HOOK_MANIFEST_TEMPLATE, HookInstallOptions, install_hermes_hooks,
        uninstall_hermes_hooks,
    };

    const REQUIRED_EVENTS: [&str; 6] = [
        "gateway:startup",
        "session:start",
        "session:end",
        "session:reset",
        "agent:start",
        "agent:end",
    ];

    #[test]
    fn hook_manifest_template_matches_hermes_gateway_contract() {
        assert!(HOOK_MANIFEST_TEMPLATE.contains("name: hermeship"));
        assert!(HOOK_MANIFEST_TEMPLATE.contains("events:"));

        let events = manifest_events(HOOK_MANIFEST_TEMPLATE);
        for event in REQUIRED_EVENTS {
            assert!(
                events.iter().any(|declared| declared == event),
                "HOOK.yaml is missing event {event}"
            );
        }
        assert!(
            !events.iter().any(|declared| declared == "agent:step"),
            "agent:step must stay opt-in because enable_agent_step defaults to false"
        );
        assert!(
            !events.iter().any(|declared| declared == "command:*"),
            "command:* must stay opt-in because enable_command_events defaults to false"
        );
    }

    #[test]
    fn hook_handler_template_is_stdlib_fail_open_bridge() {
        assert!(HOOK_HANDLER_TEMPLATE.contains("def handle(event_type, context):"));
        assert!(HOOK_HANDLER_TEMPLATE.contains("hermeship"));
        assert!(HOOK_HANDLER_TEMPLATE.contains("hermes"));
        assert!(HOOK_HANDLER_TEMPLATE.contains("hook"));
        assert!(HOOK_HANDLER_TEMPLATE.contains("--payload"));
        assert!(HOOK_HANDLER_TEMPLATE.contains("subprocess.run"));
        assert!(HOOK_HANDLER_TEMPLATE.contains("timeout="));
        assert!(HOOK_HANDLER_TEMPLATE.contains("except Exception"));
        assert!(HOOK_HANDLER_TEMPLATE.contains("separators=(\",\", \":\")"));

        for forbidden in [
            "import hermeship",
            "from hermeship",
            "DISCORD_TOKEN",
            "HERMESHIP_DISCORD_TOKEN",
            "authorization",
            "cookie",
            "secret",
        ] {
            assert!(
                !HOOK_HANDLER_TEMPLATE.contains(forbidden),
                "handler template contains forbidden text `{forbidden}`"
            );
        }

        for import in handler_imports(HOOK_HANDLER_TEMPLATE) {
            let module = import
                .strip_prefix("import ")
                .or_else(|| import.strip_prefix("from "))
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap();
            assert!(
                matches!(module, "json" | "os" | "subprocess" | "sys"),
                "handler imports non-stdlib or unexpected module `{module}`"
            );
        }
    }

    #[test]
    fn install_writes_hook_files_to_fake_hermes_home() {
        let home = temp_dir("install-writes");
        let report = install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: false,
        })
        .unwrap();

        let hook_dir = home.join("hooks/hermeship");
        let manifest = hook_dir.join("HOOK.yaml");
        let handler = hook_dir.join("handler.py");

        assert_eq!(report.hook_dir, hook_dir);
        assert!(report.written_files.contains(&manifest));
        assert!(report.written_files.contains(&handler));
        assert_eq!(
            fs::read_to_string(manifest).unwrap(),
            HOOK_MANIFEST_TEMPLATE
        );
        let installed_handler = fs::read_to_string(handler).unwrap();
        assert!(installed_handler.contains(r#"DEFAULT_HERMESHIP_BIN = "/tmp/hermeship-bin""#));
        assert!(!installed_handler.contains("__HERMESHIP_BIN__"));

        remove_temp_dir(&home);
    }

    #[test]
    fn install_does_not_overwrite_existing_files_without_force() {
        let home = temp_dir("install-no-overwrite");
        let hook_dir = home.join("hooks/hermeship");
        fs::create_dir_all(&hook_dir).unwrap();
        let handler = hook_dir.join("handler.py");
        fs::write(&handler, "local handler").unwrap();

        let report = install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: false,
        })
        .unwrap();

        assert_eq!(fs::read_to_string(&handler).unwrap(), "local handler");
        assert!(report.skipped_files.contains(&handler));
        assert!(report.written_files.contains(&hook_dir.join("HOOK.yaml")));

        remove_temp_dir(&home);
    }

    #[test]
    fn install_force_overwrites_existing_files() {
        let home = temp_dir("install-force");
        let hook_dir = home.join("hooks/hermeship");
        fs::create_dir_all(&hook_dir).unwrap();
        let handler = hook_dir.join("handler.py");
        fs::write(&handler, "old handler").unwrap();

        let report = install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: true,
            dry_run: false,
        })
        .unwrap();

        assert!(
            fs::read_to_string(&handler)
                .unwrap()
                .contains(r#"DEFAULT_HERMESHIP_BIN = "/tmp/hermeship-bin""#)
        );
        assert!(report.written_files.contains(&handler));
        assert!(!report.skipped_files.contains(&handler));

        remove_temp_dir(&home);
    }

    #[test]
    fn install_dry_run_reports_paths_without_writing() {
        let home = temp_dir("install-dry-run");
        let report = install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: true,
        })
        .unwrap();

        assert!(report.dry_run);
        assert_eq!(
            report.planned_files,
            vec![
                home.join("hooks/hermeship/HOOK.yaml"),
                home.join("hooks/hermeship/handler.py"),
                home.join("hooks/hermeship/.hermeship-managed.json"),
            ]
        );
        assert!(report.written_files.is_empty());
        assert!(!home.join("hooks/hermeship/HOOK.yaml").exists());
        assert!(!home.join("hooks/hermeship/handler.py").exists());
        assert!(
            !home
                .join("hooks/hermeship/.hermeship-managed.json")
                .exists()
        );

        remove_temp_dir(&home);
    }

    #[test]
    fn uninstall_removes_only_hermeship_hook_directory() {
        let home = temp_dir("uninstall-hooks");
        install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: false,
        })
        .unwrap();
        let other_hook = home.join("hooks/other/HOOK.yaml");
        fs::create_dir_all(other_hook.parent().unwrap()).unwrap();
        fs::write(&other_hook, "name: other\n").unwrap();

        let report = uninstall_hermes_hooks(&home, false).unwrap();

        assert!(!home.join("hooks/hermeship").exists());
        assert!(report.removed_paths.contains(&home.join("hooks/hermeship")));
        assert!(other_hook.exists());

        remove_temp_dir(&home);
    }

    #[test]
    fn uninstall_preserves_user_owned_files_and_modified_managed_files() {
        let home = temp_dir("uninstall-preserves-user-files");
        install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: false,
        })
        .unwrap();
        let hook_dir = home.join("hooks/hermeship");
        let manifest = hook_dir.join("HOOK.yaml");
        let handler = hook_dir.join("handler.py");
        let custom = hook_dir.join("custom.txt");
        fs::write(&manifest, "name: local-hermeship\n").unwrap();
        fs::write(&custom, "keep me").unwrap();

        let report = uninstall_hermes_hooks(&home, false).unwrap();

        assert!(
            !handler.exists(),
            "unchanged managed handler should be removed"
        );
        assert!(manifest.exists(), "modified manifest must be preserved");
        assert_eq!(
            fs::read_to_string(&manifest).unwrap(),
            "name: local-hermeship\n"
        );
        assert!(custom.exists(), "user-owned file must be preserved");
        assert!(hook_dir.exists(), "non-empty hook directory must remain");
        assert!(!report.removed_paths.contains(&manifest));
        assert!(!report.removed_paths.contains(&custom));

        remove_temp_dir(&home);
    }

    #[test]
    fn installed_handler_uses_rendered_binary_path_without_env_override() {
        let home = temp_dir("handler-default-bin");
        let bin_dir = home.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let fake_bin = bin_dir.join("hermeship");
        let stdin_path = home.join("payload-default-bin.json");
        write_executable(
            &fake_bin,
            &format!("#!/bin/sh\n/bin/cat > '{}'\n", stdin_path.display()),
        );
        install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(fake_bin),
            force: false,
            dry_run: false,
        })
        .unwrap();

        let output = run_handler(
            &home.join("hooks/hermeship/handler.py"),
            "session:start",
            r#"{"session_id":"synthetic-session-default-bin"}"#,
            &[("HERMESHIP_HOOK_TIMEOUT_SECS", "10")],
        );

        assert!(
            output.status.success(),
            "handler failed: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let payload: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(stdin_path).unwrap()).unwrap();
        assert_eq!(payload["event"], "session:start");
        assert_eq!(
            payload["context"]["session_id"],
            "synthetic-session-default-bin"
        );

        remove_temp_dir(&home);
    }

    #[test]
    fn env_override_still_wins_over_rendered_binary_path() {
        let home = temp_dir("handler-env-override");
        let default_bin = home.join("default-bin");
        write_executable(&default_bin, "#!/bin/sh\nexit 99\n");
        let override_bin = home.join("override-bin");
        let stdin_path = home.join("payload-env-override.json");
        write_executable(
            &override_bin,
            &format!("#!/bin/sh\n/bin/cat > '{}'\n", stdin_path.display()),
        );
        install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(default_bin),
            force: false,
            dry_run: false,
        })
        .unwrap();
        let override_bin_env = override_bin.display().to_string();

        let output = run_handler(
            &home.join("hooks/hermeship/handler.py"),
            "session:end",
            r#"{"session_id":"synthetic-session-override"}"#,
            &[
                ("HERMESHIP_BIN", &override_bin_env),
                ("HERMESHIP_HOOK_TIMEOUT_SECS", "10"),
            ],
        );

        assert!(
            output.status.success(),
            "handler failed: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let payload: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(stdin_path).unwrap()).unwrap();
        assert_eq!(payload["event"], "session:end");

        remove_temp_dir(&home);
    }

    #[test]
    fn install_writes_managed_marker_for_safe_uninstall() {
        let home = temp_dir("install-marker");
        let report = install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: false,
        })
        .unwrap();
        let marker = home.join("hooks/hermeship/.hermeship-managed.json");

        assert!(marker.exists());
        assert!(report.written_files.contains(&marker));
        let raw_marker = fs::read_to_string(marker).unwrap();
        assert!(raw_marker.contains("HOOK.yaml"));
        assert!(raw_marker.contains("handler.py"));

        remove_temp_dir(&home);
    }

    #[test]
    fn uninstall_without_marker_does_not_delete_unknown_hook_directory() {
        let home = temp_dir("uninstall-no-marker");
        let hook_dir = home.join("hooks/hermeship");
        fs::create_dir_all(&hook_dir).unwrap();
        let manifest = hook_dir.join("HOOK.yaml");
        fs::write(&manifest, HOOK_MANIFEST_TEMPLATE).unwrap();

        let report = uninstall_hermes_hooks(&home, false).unwrap();

        assert!(manifest.exists());
        assert!(report.removed_paths.is_empty());

        remove_temp_dir(&home);
    }

    #[test]
    fn uninstall_ignores_tampered_marker_paths_outside_hook_dir() {
        let home = temp_dir("uninstall-tampered-marker");
        let hook_dir = home.join("hooks/hermeship");
        fs::create_dir_all(&hook_dir).unwrap();
        let outside = home.join("outside.txt");
        fs::write(&outside, "outside").unwrap();
        fs::write(
            hook_dir.join(".hermeship-managed.json"),
            format!(
                r#"{{
  "version": 1,
  "files": [
    {{"path": "../outside.txt", "checksum": "{}"}}
  ]
}}"#,
                super::checksum("outside")
            ),
        )
        .unwrap();

        let report = uninstall_hermes_hooks(&home, false).unwrap();

        assert!(outside.exists());
        assert!(!report.removed_paths.contains(&outside));

        remove_temp_dir(&home);
    }

    #[test]
    fn rendered_handler_escapes_binary_path_for_python_string_literal() {
        let rendered =
            super::render_hook_handler(Some(Path::new(r#"/tmp/bin/"hermeship""#))).unwrap();

        assert!(rendered.contains(r#"DEFAULT_HERMESHIP_BIN = "/tmp/bin/\"hermeship\"""#));
    }

    #[test]
    fn handler_smoke_invokes_fake_hermeship_with_stdin_payload() {
        let home = temp_dir("handler-smoke");
        install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: false,
        })
        .unwrap();
        let bin_dir = home.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let fake_bin = bin_dir.join("hermeship");
        let stdin_path = home.join("payload.json");
        let args_path = home.join("args.txt");
        write_executable(
            &fake_bin,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" > '{}'\n/bin/cat > '{}'\n",
                args_path.display(),
                stdin_path.display()
            ),
        );
        let fake_bin_env = fake_bin.display().to_string();

        let output = run_handler(
            &home.join("hooks/hermeship/handler.py"),
            "agent:start",
            r#"{"session_id":"synthetic-session","agent_name":"demo-agent"}"#,
            &[
                ("HERMESHIP_BIN", &fake_bin_env),
                ("HERMESHIP_HOOK_TIMEOUT_SECS", "10"),
            ],
        );

        assert!(
            output.status.success(),
            "handler failed: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            fs::read_to_string(&args_path)
                .unwrap_or_else(|error| panic!(
                    "missing fake binary args at {}: {error}; stderr={}",
                    args_path.display(),
                    String::from_utf8_lossy(&output.stderr)
                ))
                .trim(),
            "hermes hook --payload -"
        );
        let payload: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(stdin_path).unwrap()).unwrap();
        assert_eq!(payload["provider"], "hermes");
        assert_eq!(payload["source"], "gateway");
        assert_eq!(payload["event"], "agent:start");
        assert_eq!(payload["context"]["session_id"], "synthetic-session");
        assert_eq!(payload["context"]["agent_name"], "demo-agent");

        remove_temp_dir(&home);
    }

    #[test]
    fn handler_smoke_fail_opens_for_missing_failure_and_timeout() {
        let home = temp_dir("handler-fail-open");
        install_hermes_hooks(&HookInstallOptions {
            hermes_home: home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: false,
        })
        .unwrap();
        let handler = home.join("hooks/hermeship/handler.py");
        let bin_dir = home.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();

        let missing = run_handler(
            &handler,
            "agent:start",
            r#"{"session_id":"missing"}"#,
            &[("HERMESHIP_BIN", &home.join("missing").display().to_string())],
        );
        assert!(missing.status.success());

        let failing_bin = bin_dir.join("hermeship-fails");
        write_executable(&failing_bin, "#!/bin/sh\nexit 42\n");
        let failing = run_handler(
            &handler,
            "agent:start",
            r#"{"session_id":"failure"}"#,
            &[("HERMESHIP_BIN", &failing_bin.display().to_string())],
        );
        assert!(failing.status.success());

        let slow_bin = bin_dir.join("hermeship-slow");
        write_executable(&slow_bin, "#!/bin/sh\n/bin/sleep 1\n");
        let timeout = run_handler(
            &handler,
            "agent:start",
            r#"{"session_id":"timeout"}"#,
            &[
                ("HERMESHIP_BIN", &slow_bin.display().to_string()),
                ("HERMESHIP_HOOK_TIMEOUT_SECS", "0.1"),
            ],
        );
        assert!(timeout.status.success());

        remove_temp_dir(&home);
    }

    fn manifest_events(raw: &str) -> Vec<String> {
        raw.lines()
            .map(str::trim)
            .filter_map(|line| line.strip_prefix("- "))
            .map(|event| event.trim_matches('"').to_string())
            .collect()
    }

    fn handler_imports(raw: &str) -> Vec<&str> {
        raw.lines()
            .map(str::trim)
            .filter(|line| line.starts_with("import ") || line.starts_with("from "))
            .collect()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "hermeship-{name}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn remove_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    fn write_executable(path: &Path, content: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = file.metadata().unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
    }

    fn run_handler(
        handler_path: &Path,
        event_type: &str,
        context_json: &str,
        envs: &[(&str, &str)],
    ) -> std::process::Output {
        let code = format!(
            r#"
import importlib.util
import json
import sys

spec = importlib.util.spec_from_file_location("hermeship_test_hook", {handler_path:?})
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
module.handle({event_type:?}, json.loads({context_json:?}))
"#,
            handler_path = handler_path.display().to_string(),
            event_type = event_type,
            context_json = context_json
        );
        let mut command = Command::new("python3");
        command.arg("-c").arg(code);
        for (key, value) in envs {
            command.env(key, value);
        }
        command.output().unwrap()
    }
}
