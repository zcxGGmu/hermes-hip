use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::AppConfig;
use crate::hooks::{HookUninstallReport, uninstall_hermes_hooks};

pub const SERVICE_TEMPLATE: &str = include_str!("../deploy/hermeship.service");
const HOME_MANAGED_MARKER_FILE: &str = ".hermeship-managed.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOptions {
    pub home: PathBuf,
    pub config_path: PathBuf,
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallReport {
    pub home: PathBuf,
    pub config_path: PathBuf,
    pub planned_dirs: Vec<PathBuf>,
    pub planned_files: Vec<PathBuf>,
    pub created_dirs: Vec<PathBuf>,
    pub written_files: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
    pub dry_run: bool,
}

impl InstallReport {
    pub fn render(&self) -> String {
        let mut output = if self.dry_run {
            format!(
                "hermeship install dry-run: would create {}\n",
                self.home.display()
            )
        } else {
            format!("hermeship install complete: {}\n", self.home.display())
        };

        for dir in &self.created_dirs {
            output.push_str(&format!("  created dir {}\n", dir.display()));
        }
        for file in &self.written_files {
            output.push_str(&format!("  wrote {}\n", file.display()));
        }
        for file in &self.skipped_files {
            output.push_str(&format!("  skipped existing {}\n", file.display()));
        }
        if self.dry_run {
            for dir in &self.planned_dirs {
                output.push_str(&format!("  would create dir {}\n", dir.display()));
            }
            for file in &self.planned_files {
                output.push_str(&format!("  would write {}\n", file.display()));
            }
        }
        output.push_str("next steps:\n");
        output.push_str("  hermeship setup --default-channel <channel>\n");
        output.push_str("  hermeship hermes install-hooks --scope global --force\n");
        output.push_str("  hermeship start\n");
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupOptions {
    pub config_path: PathBuf,
    pub discord_token: Option<String>,
    pub default_channel: Option<String>,
    pub daemon_url: Option<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupReport {
    pub config_path: PathBuf,
    pub changed_fields: Vec<String>,
    pub dry_run: bool,
}

impl SetupReport {
    pub fn render(&self) -> String {
        let prefix = if self.dry_run {
            "hermeship setup dry-run"
        } else {
            "hermeship setup complete"
        };
        let mut output = format!("{prefix}: {}\n", self.config_path.display());
        for field in &self.changed_fields {
            if field == "providers.discord.token" {
                output.push_str("  providers.discord.token=<redacted>\n");
            } else {
                output.push_str(&format!("  updated {field}\n"));
            }
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallOptions {
    pub home: PathBuf,
    pub config_path: PathBuf,
    pub hermes_home: Option<PathBuf>,
    pub remove_config: bool,
    pub remove_state: bool,
    pub remove_hooks: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallReport {
    pub planned_paths: Vec<PathBuf>,
    pub removed_paths: Vec<PathBuf>,
    pub skipped_paths: Vec<PathBuf>,
    pub hook_report: Option<HookUninstallReport>,
    pub dry_run: bool,
}

impl UninstallReport {
    pub fn render(&self) -> String {
        let mut output = if self.dry_run {
            "hermeship uninstall dry-run\n".to_string()
        } else {
            "hermeship uninstall complete\n".to_string()
        };
        for path in &self.planned_paths {
            if self.dry_run {
                output.push_str(&format!("  would remove {}\n", path.display()));
            }
        }
        for path in &self.removed_paths {
            output.push_str(&format!("  removed {}\n", path.display()));
        }
        for path in &self.skipped_paths {
            output.push_str(&format!("  preserved {}\n", path.display()));
        }
        output
    }
}

pub fn default_home() -> PathBuf {
    if let Some(home) = normalize_env_path("HERMESHIP_HOME") {
        return home;
    }

    normalize_env_path("HOME")
        .or_else(|| normalize_env_path("USERPROFILE"))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hermeship")
}

pub fn install(options: &InstallOptions) -> Result<InstallReport> {
    let planned_dirs = vec![
        options.home.clone(),
        options.home.join("state"),
        options.home.join("hooks"),
        options.home.join("logs"),
    ];
    let planned_files = vec![options.config_path.clone(), home_marker_path(&options.home)];
    let mut report = InstallReport {
        home: options.home.clone(),
        config_path: options.config_path.clone(),
        planned_dirs,
        planned_files,
        created_dirs: Vec::new(),
        written_files: Vec::new(),
        skipped_files: Vec::new(),
        dry_run: options.dry_run,
    };

    if options.dry_run {
        return Ok(report);
    }

    for dir in &report.planned_dirs {
        let existed = dir.exists();
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory {}", dir.display()))?;
        if !existed {
            report.created_dirs.push(dir.clone());
        }
    }

    if let Some(parent) = options.config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }

    if options.config_path.exists() && !options.force {
        report.skipped_files.push(options.config_path.clone());
    } else {
        let config = AppConfig::default();
        let raw = config.to_pretty_toml()?;
        write_config_file(&options.config_path, &raw)?;
        report.written_files.push(options.config_path.clone());
    }
    write_home_marker(&options.home)?;
    report.written_files.push(home_marker_path(&options.home));

    Ok(report)
}

pub fn setup(options: &SetupOptions) -> Result<SetupReport> {
    let mut config = AppConfig::load_or_default_with_env(&options.config_path, |_| None)?;
    let mut changed_fields = Vec::new();

    if let Some(token) = normalize_text(options.discord_token.as_deref()) {
        config.providers.discord.token = Some(token);
        changed_fields.push("providers.discord.token".to_string());
    }
    if let Some(channel) = normalize_text(options.default_channel.as_deref()) {
        config.defaults.channel = Some(channel);
        changed_fields.push("defaults.channel".to_string());
    }
    if let Some(url) = normalize_text(options.daemon_url.as_deref()) {
        config.daemon.base_url = Some(url);
        changed_fields.push("daemon.base_url".to_string());
    }

    if changed_fields.is_empty() {
        anyhow::bail!(
            "setup requires at least one of --discord-token-stdin, --discord-token-env, --default-channel, or --daemon-url"
        );
    }

    let report = SetupReport {
        config_path: options.config_path.clone(),
        changed_fields,
        dry_run: options.dry_run,
    };

    if options.dry_run {
        return Ok(report);
    }

    if let Some(parent) = options.config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }
    write_config_file(&options.config_path, &config.to_pretty_toml()?)?;

    Ok(report)
}

pub fn uninstall(options: &UninstallOptions) -> Result<UninstallReport> {
    let mut report = UninstallReport {
        planned_paths: planned_uninstall_paths(options),
        removed_paths: Vec::new(),
        skipped_paths: Vec::new(),
        hook_report: None,
        dry_run: options.dry_run,
    };

    if options.dry_run {
        return Ok(report);
    }

    if has_destructive_uninstall_flags(options) {
        ensure_managed_home(&options.home)?;
    }

    if options.remove_config {
        remove_file_if_exists(&options.config_path, &mut report.removed_paths)?;
    } else if options.config_path.exists() {
        report.skipped_paths.push(options.config_path.clone());
    }

    if options.remove_state {
        remove_dir_if_exists(&options.home.join("state"), &mut report.removed_paths)?;
        remove_dir_if_exists(&options.home.join("logs"), &mut report.removed_paths)?;
    }

    if options.remove_hooks {
        remove_empty_dir_if_exists(&options.home.join("hooks"), &mut report.removed_paths)?;
        if let Some(hermes_home) = &options.hermes_home {
            let hook_report = uninstall_hermes_hooks(hermes_home, false)?;
            report
                .removed_paths
                .extend(hook_report.removed_paths.iter().cloned());
            report.hook_report = Some(hook_report);
        }
    }

    Ok(report)
}

fn planned_uninstall_paths(options: &UninstallOptions) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if options.remove_config {
        paths.push(options.config_path.clone());
    }
    if options.remove_state {
        paths.push(options.home.join("state"));
        paths.push(options.home.join("logs"));
    }
    if options.remove_hooks {
        paths.push(options.home.join("hooks"));
        if let Some(hermes_home) = &options.hermes_home {
            paths.push(hermes_home.join("hooks").join("hermeship"));
        }
    }
    paths
}

fn remove_file_if_exists(path: &Path, removed: &mut Vec<PathBuf>) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("failed to remove file {}", path.display()))?;
        removed.push(path.to_path_buf());
    }
    Ok(())
}

fn remove_dir_if_exists(path: &Path, removed: &mut Vec<PathBuf>) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to remove directory {}", path.display()))?;
        removed.push(path.to_path_buf());
    }
    Ok(())
}

fn remove_empty_dir_if_exists(path: &Path, removed: &mut Vec<PathBuf>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if !path.is_dir() {
        anyhow::bail!("refusing to remove non-directory {}", path.display());
    }
    if fs::read_dir(path)
        .with_context(|| format!("failed to read directory {}", path.display()))?
        .next()
        .is_some()
    {
        anyhow::bail!(
            "refusing to remove non-empty unmanaged directory {}",
            path.display()
        );
    }

    fs::remove_dir(path)
        .with_context(|| format!("failed to remove directory {}", path.display()))?;
    removed.push(path.to_path_buf());
    Ok(())
}

fn has_destructive_uninstall_flags(options: &UninstallOptions) -> bool {
    options.remove_config || options.remove_state || options.remove_hooks
}

fn home_marker_path(home: &Path) -> PathBuf {
    home.join(HOME_MANAGED_MARKER_FILE)
}

fn ensure_managed_home(home: &Path) -> Result<()> {
    let marker = home_marker_path(home);
    if marker.is_file() {
        return Ok(());
    }

    anyhow::bail!(
        "refusing to uninstall unmanaged Hermeship home: {}",
        home.display()
    )
}

fn write_home_marker(home: &Path) -> Result<()> {
    let marker = home_marker_path(home);
    fs::write(&marker, r#"{"version":1,"kind":"hermeship-home"}"#)
        .with_context(|| format!("failed to write Hermeship home marker {}", marker.display()))
}

fn write_config_file(path: &Path, raw: &str) -> Result<()> {
    let mut file = open_private_config(path)?;
    file.write_all(raw.as_bytes())
        .with_context(|| format!("failed to write Hermeship config {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to flush Hermeship config {}", path.display()))?;
    set_private_config_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn open_private_config(path: &Path) -> Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("failed to open Hermeship config {}", path.display()))
}

#[cfg(not(unix))]
fn open_private_config(path: &Path) -> Result<fs::File> {
    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .with_context(|| format!("failed to open Hermeship config {}", path.display()))
}

#[cfg(unix)]
fn set_private_config_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .with_context(|| format!("failed to stat Hermeship config {}", path.display()))?
        .permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to chmod Hermeship config {}", path.display()))
}

#[cfg(not(unix))]
fn set_private_config_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

fn normalize_env_path(name: &str) -> Option<PathBuf> {
    env::var(name).ok().and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
    })
}

fn normalize_text(value: Option<&str>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::hooks::{HookInstallOptions, install_hermes_hooks};

    #[test]
    fn install_dry_run_reports_paths_without_writing() {
        let home = temp_dir("install-dry-run");
        let config_path = home.join("config.toml");

        let report = install(&InstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            force: false,
            dry_run: true,
        })
        .unwrap();

        assert!(report.dry_run);
        assert!(report.planned_dirs.contains(&home));
        assert!(report.planned_dirs.contains(&home.join("state")));
        assert!(report.planned_dirs.contains(&home.join("hooks")));
        assert!(report.planned_dirs.contains(&home.join("logs")));
        assert!(report.planned_files.contains(&config_path));
        assert!(
            report
                .planned_files
                .contains(&home.join(".hermeship-managed.json"))
        );
        assert!(!home.exists());
        assert!(report.render().contains("would create"));

        remove_temp_dir(&home);
    }

    #[test]
    fn install_scaffolds_config_and_dirs_without_overwriting_existing_config() {
        let home = temp_dir("install-scaffold");
        let config_path = home.join("config.toml");
        fs::create_dir_all(&home).unwrap();
        fs::write(&config_path, "# local config\n").unwrap();

        let report = install(&InstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();

        assert!(home.join("state").is_dir());
        assert!(home.join("hooks").is_dir());
        assert!(home.join("logs").is_dir());
        assert!(home.join(".hermeship-managed.json").is_file());
        assert_eq!(
            fs::read_to_string(&config_path).unwrap(),
            "# local config\n"
        );
        assert!(report.skipped_files.contains(&config_path));
        assert!(report.render().contains("next steps"));

        remove_temp_dir(&home);
    }

    #[test]
    fn install_force_replaces_scaffold_config() {
        let home = temp_dir("install-force");
        let config_path = home.join("config.toml");
        fs::create_dir_all(&home).unwrap();
        fs::write(&config_path, "# local config\n").unwrap();

        let report = install(&InstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            force: true,
            dry_run: false,
        })
        .unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        assert!(raw.contains("[daemon]"));
        assert!(raw.contains("[privacy]"));
        assert!(report.written_files.contains(&config_path));
        assert_private_config_permissions(&config_path);

        remove_temp_dir(&home);
    }

    #[test]
    fn setup_updates_config_without_rendering_secret() {
        let home = temp_dir("setup-config");
        let config_path = home.join("config.toml");
        install(&InstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();

        let report = setup(&SetupOptions {
            config_path: config_path.clone(),
            discord_token: Some("synthetic-token".to_string()),
            default_channel: Some("ops-channel".to_string()),
            daemon_url: Some("http://127.0.0.1:25296".to_string()),
            dry_run: false,
        })
        .unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        assert!(raw.contains("synthetic-token"));
        assert!(raw.contains("ops-channel"));
        assert!(raw.contains("http://127.0.0.1:25296"));
        assert!(
            report
                .changed_fields
                .contains(&"providers.discord.token".to_string())
        );
        assert!(!report.render().contains("synthetic-token"));
        assert!(
            report
                .render()
                .contains("providers.discord.token=<redacted>")
        );
        assert_private_config_permissions(&config_path);

        remove_temp_dir(&home);
    }

    #[test]
    fn setup_dry_run_reports_without_writing_config() {
        let home = temp_dir("setup-dry-run");
        let config_path = home.join("config.toml");

        let report = setup(&SetupOptions {
            config_path: config_path.clone(),
            discord_token: None,
            default_channel: Some("ops-channel".to_string()),
            daemon_url: None,
            dry_run: true,
        })
        .unwrap();

        assert!(report.dry_run);
        assert!(
            report
                .changed_fields
                .contains(&"defaults.channel".to_string())
        );
        assert!(!config_path.exists());

        remove_temp_dir(&home);
    }

    #[test]
    fn uninstall_refuses_destructive_removal_without_home_marker() {
        let home = temp_dir("uninstall-unmanaged");
        let config_path = home.join("config.toml");
        fs::create_dir_all(home.join("state")).unwrap();
        fs::write(&config_path, "local").unwrap();

        let error = uninstall(&UninstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            hermes_home: None,
            remove_config: true,
            remove_state: true,
            remove_hooks: false,
            dry_run: false,
        })
        .unwrap_err()
        .to_string();

        assert!(
            error.contains("refusing to uninstall unmanaged Hermeship home"),
            "{error}"
        );
        assert!(config_path.exists());
        assert!(home.join("state").exists());

        remove_temp_dir(&home);
    }

    #[test]
    fn uninstall_refuses_to_remove_non_empty_local_hooks_dir() {
        let home = temp_dir("uninstall-non-empty-hooks");
        let config_path = home.join("config.toml");
        install(&InstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();
        fs::write(home.join("hooks/user-file"), "keep").unwrap();

        let error = uninstall(&UninstallOptions {
            home: home.clone(),
            config_path,
            hermes_home: None,
            remove_config: false,
            remove_state: false,
            remove_hooks: true,
            dry_run: false,
        })
        .unwrap_err()
        .to_string();

        assert!(
            error.contains("refusing to remove non-empty unmanaged directory"),
            "{error}"
        );
        assert!(home.join("hooks/user-file").exists());

        remove_temp_dir(&home);
    }

    #[test]
    fn uninstall_preserves_config_by_default_and_removes_opt_in_paths() {
        let home = temp_dir("uninstall-local");
        let config_path = home.join("config.toml");
        install(&InstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();
        fs::write(home.join("state/queue.json"), "{}").unwrap();
        fs::write(home.join("logs/hermeship.log"), "log").unwrap();

        let default_report = uninstall(&UninstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            hermes_home: None,
            remove_config: false,
            remove_state: false,
            remove_hooks: false,
            dry_run: false,
        })
        .unwrap();

        assert!(default_report.removed_paths.is_empty());
        assert!(config_path.exists());
        assert!(home.join("state/queue.json").exists());

        let remove_report = uninstall(&UninstallOptions {
            home: home.clone(),
            config_path: config_path.clone(),
            hermes_home: None,
            remove_config: true,
            remove_state: true,
            remove_hooks: false,
            dry_run: false,
        })
        .unwrap();

        assert!(!config_path.exists());
        assert!(!home.join("state").exists());
        assert!(!home.join("logs").exists());
        assert!(remove_report.removed_paths.contains(&config_path));

        remove_temp_dir(&home);
    }

    #[test]
    fn uninstall_remove_hooks_uses_safe_hermes_hook_uninstall() {
        let home = temp_dir("uninstall-home");
        let config_path = home.join("config.toml");
        let hermes_home = temp_dir("uninstall-hermes-home");
        install(&InstallOptions {
            home: home.clone(),
            config_path,
            force: false,
            dry_run: false,
        })
        .unwrap();
        install_hermes_hooks(&HookInstallOptions {
            hermes_home: hermes_home.clone(),
            hermeship_bin: Some(PathBuf::from("/tmp/hermeship-bin")),
            force: false,
            dry_run: false,
        })
        .unwrap();
        let other_hook = hermes_home.join("hooks/other/HOOK.yaml");
        fs::create_dir_all(other_hook.parent().unwrap()).unwrap();
        fs::write(&other_hook, "name: other\n").unwrap();

        let report = uninstall(&UninstallOptions {
            home: home.clone(),
            config_path: home.join("config.toml"),
            hermes_home: Some(hermes_home.clone()),
            remove_config: false,
            remove_state: false,
            remove_hooks: true,
            dry_run: false,
        })
        .unwrap();

        assert!(!hermes_home.join("hooks/hermeship").exists());
        assert!(other_hook.exists());
        assert!(
            report
                .removed_paths
                .contains(&hermes_home.join("hooks/hermeship"))
        );

        remove_temp_dir(&home);
        remove_temp_dir(&hermes_home);
    }

    #[test]
    fn service_template_documents_user_service_without_real_install() {
        assert!(SERVICE_TEMPLATE.contains("[Unit]"));
        assert!(SERVICE_TEMPLATE.contains("Description=Hermeship"));
        assert!(SERVICE_TEMPLATE.contains("Environment=HERMESHIP_CONFIG="));
        assert!(SERVICE_TEMPLATE.contains("ExecStart="));
        assert!(SERVICE_TEMPLATE.contains("hermeship start"));
    }

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "hermeship-lifecycle-{name}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn remove_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(unix)]
    fn assert_private_config_permissions(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let mode = fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "config permissions should be private");
    }

    #[cfg(not(unix))]
    fn assert_private_config_permissions(_path: &Path) {}
}
