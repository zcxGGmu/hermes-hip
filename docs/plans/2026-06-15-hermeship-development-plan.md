# Hermeship Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build `hermeship`, a Hermes-to-clawhip adapter that forwards Hermes lifecycle events to clawhip without polluting Hermes gateway sessions.

**Architecture:** `hermeship` is a small Python package plus installable Hermes hook bundle. It listens to Hermes gateway hooks first, maps them into stable clawhip `agent.*` / `emit` events, and calls the local clawhip daemon through the CLI or HTTP client. A later observer-plugin mode can add higher-fidelity tool/API telemetry without changing the MVP contract.

**Tech Stack:** Python 3.11+, stdlib (`tomllib`, `dataclasses`, `pathlib`, `subprocess`, `json`, `logging`), `pytest`, `ruff`, packaging via `hatchling` or `setuptools`, Hermes hook directories under `~/.hermes/hooks/`, clawhip CLI/daemon API, optional `httpx` only if HTTP transport is added later.

---

## 0. Context And Constraints

### Repositories

- Hermeship target repo: `/home/zq/work-space/repo/ai-projs/posp/hermeship`
- clawhip reference repo: `/home/zq/work-space/repo/ai-projs/posp/clawhip`
- Hermes reference repo: `/home/zq/work-space/repo/ai-projs/agents/hermes-agent`

### Relevant clawhip Surfaces

Use these as implementation contracts:

- `clawhip/docs/event-contract-v1.md`
- `clawhip/docs/native-event-contract.md`
- `clawhip/src/cli.rs`
- `clawhip/src/events.rs`
- `clawhip/src/event/body.rs`
- `clawhip/src/native_hooks.rs`
- `clawhip/plugins/codex/bridge.sh`
- `clawhip/plugins/claude-code/bridge.sh`

Important clawhip facts:

- `clawhip` is daemon-first: events go to the daemon, then router, renderer, Discord sink.
- Stable v1 provider-native hook contract is intentionally limited to Codex/Claude shared events:
  - `SessionStart`
  - `PreToolUse`
  - `PostToolUse`
  - `UserPromptSubmit`
  - `Stop`
- The safer MVP path for Hermes is not to change that v1 contract. Use `clawhip agent ...` and `clawhip emit ...` first.
- `clawhip native hook --provider hermes` can be explored later only after contract discussion, because the documented v1 provider surface names Codex/Claude.

### Relevant Hermes Surfaces

Use these as implementation contracts:

- `hermes-agent/gateway/hooks.py`
- `hermes-agent/docs/observability/README.md`
- `hermes-agent/docs/middleware/README.md`
- `hermes-agent/gateway/stream_events.py`
- `hermes-agent/AGENTS.md`

Important Hermes facts:

- Hermes favors plugins and skills at the edge; do not grow Hermes core for this adapter.
- Gateway hooks live under `~/.hermes/hooks/<hook-name>/`.
- Gateway hook manifest shape:
  - `HOOK.yaml`
  - `handler.py`
- Gateway hook events include:
  - `gateway:startup`
  - `session:start`
  - `session:end`
  - `session:reset`
  - `agent:start`
  - `agent:step`
  - `agent:end`
  - `command:*`
- Hook callbacks are fail-open. Hermes catches exceptions and continues.
- Observer hooks exist for high-fidelity telemetry, but an external adapter should start with gateway hooks because they are easy to install and do not require Hermes core changes.

## 0.1 Engineering Principles

Hermeship should obey these rules throughout the build:

- **Fail open.** A forwarding failure must never break Hermes itself.
- **Keep dependencies boring.** Use stdlib first; add a third-party dependency only if it removes real complexity.
- **No feedback loops.** Events emitted by Hermeship must carry enough origin metadata to avoid re-ingestion or recursive notification loops.
- **Idempotent installation.** Re-running `install-hook` should preserve existing user edits unless `--force` is explicitly used.
- **Small payloads only.** Send summaries, not histories. Truncate aggressively.
- **Stable contracts first.** Keep event names, config keys, and CLI flags stable once the first live verification passes.
- **Observable failures.** When something is skipped, timed out, or dropped, log the reason and the key correlation IDs.

## 1. Product Boundary

### What Hermeship Is

Hermeship is a Hermes adapter for clawhip:

```text
Hermes gateway hook events
  -> hermeship hook handler
  -> hermeship event mapper
  -> clawhip CLI or daemon HTTP
  -> clawhip routing/rendering/sinks
  -> Discord notification channel
```

### What Hermeship Is Not

Hermeship is not:

- a replacement for clawhip
- a fork of clawhip
- a new Discord bot
- a Hermes runtime
- a multi-agent orchestrator
- a new Hermes model tool
- a change to clawhip's frozen Codex/Claude v1 provider-native contract

### MVP Scope

MVP only supports:

- installing a Hermes gateway hook bundle
- forwarding `agent:start`, `agent:end`, `session:start`, `session:end`, `session:reset`, and `gateway:startup`
- mapping events to clawhip `agent.started`, `agent.finished`, `agent.failed`, and `emit` events
- optional channel override from config
- fail-open behavior when clawhip is missing or down
- local dry-run output for tests and debugging

Out of scope for MVP:

- direct Hermes core patching
- observer plugin telemetry
- middleware
- bidirectional Discord control
- automatic Codex/LazyCodex worker launch
- GitHub PR creation
- native `clawhip` source changes

## 2. Naming And Event Model

### Project Name

Use `hermeship`:

```text
hermes + whip/ship = Hermes adapter that ships lifecycle events into clawhip.
```

The package and CLI should be named:

- Python package: `hermeship`
- CLI: `hermeship`
- Hermes hook name: `hermeship-clawhip`

### Hermeship Event

Internal normalized event:

```python
from dataclasses import dataclass, field
from typing import Any, Mapping

@dataclass(frozen=True)
class HermeshipEvent:
    event_type: str
    provider: str = "hermes"
    agent_name: str = "hermes"
    session_id: str | None = None
    platform: str | None = None
    user_id: str | None = None
    chat_id: str | None = None
    thread_id: str | None = None
    project: str | None = None
    summary: str | None = None
    error_message: str | None = None
    raw_context: Mapping[str, Any] = field(default_factory=dict)
```

### Event Mapping

MVP mapping:

| Hermes event | Hermeship event | clawhip ingress | clawhip event |
| --- | --- | --- | --- |
| `gateway:startup` | `gateway.started` | `clawhip emit` | `hermes.gateway.started` |
| `session:start` | `session.started` | `clawhip emit` | `hermes.session.started` |
| `session:end` | `session.finished` | `clawhip emit` | `hermes.session.finished` |
| `session:reset` | `session.reset` | `clawhip emit` | `hermes.session.reset` |
| `agent:start` | `agent.started` | `clawhip agent started` | `agent.started` |
| `agent:end` with response | `agent.finished` | `clawhip agent finished` | `agent.finished` |
| `agent:end` with error marker | `agent.failed` | `clawhip agent failed` | `agent.failed` |

Hermes `agent:end` context does not document a structured `error` field. MVP should treat it as finished unless `context` includes an explicit `error`, `exception`, or `status` field from a newer Hermes version.

### Payload Rules

- Always include `provider=hermes` in raw `emit` payloads.
- Use Hermes `session_id` as clawhip `--session`.
- Use Hermes `platform` as project fallback only when no configured project exists.
- Truncate user-visible message/response fields to avoid sending long content into Discord.
- Never send full conversation history.
- Never send raw credentials, API keys, tokens, or file contents.

## 3. Configuration

### User Config File

Create:

```text
~/.hermeship/config.toml
```

Example:

```toml
[clawhip]
mode = "cli"
binary = "clawhip"
daemon_base_url = "http://127.0.0.1:25294"
timeout_secs = 2.0

[defaults]
channel = ""
mention = ""
project = "hermes"
format = "compact"
dry_run = false

[events]
gateway_startup = true
session_start = true
session_end = true
session_reset = true
agent_start = true
agent_end = true
command_events = false

[privacy]
max_message_chars = 300
max_response_chars = 500
redact_keys = ["token", "api_key", "authorization", "password", "secret"]
dedupe_window_secs = 30
```

### Environment Overrides

Support these only for secrets/ops override, not primary behavioral settings:

```text
HERMESHIP_CONFIG
HERMESHIP_DRY_RUN
HERMESHIP_CLAWHIP_BIN
HERMESHIP_CLAWHIP_URL
```

Do not introduce `HERMES_*` user-facing settings for this adapter. Hermes contributor guidance explicitly rejects new `HERMES_*` env vars for non-secret config.

### Config Resolution Order

Resolve configuration in this order:

1. CLI flags
2. environment overrides
3. repo-local override file, if present
4. user config at `~/.hermeship/config.toml`
5. built-in defaults

The repo-local override should only be read when Hermeship can identify a repo root from the current execution context or from the Hermes hook context. The intent is to let one Hermes installation behave differently per repository without duplicating global setup.

### Validation Rules

- Unknown top-level config keys should be rejected in strict mode and warned about in permissive mode.
- Empty strings for `channel`, `mention`, and `project` should normalize to `None`.
- `dedupe_window_secs` should be clamped to a sensible minimum and maximum to avoid accidental unbounded caches.
- If config parsing fails, the adapter should disable forwarding and write a short diagnostic, not abort Hermes.

## 4. Repository Layout

Create this structure:

```text
hermeship/
  pyproject.toml
  README.md
  docs/
    plans/
      2026-06-15-hermeship-development-plan.md
    event-mapping.md
    operations.md
  src/
    hermeship/
      __init__.py
      __main__.py
      cli.py
      config.py
      events.py
      mapper.py
      privacy.py
      clawhip_client.py
      installer.py
      hook_handler.py
  templates/
    hermes-hook/
      HOOK.yaml
      handler.py
  tests/
    test_config.py
    test_mapper.py
    test_privacy.py
    test_clawhip_client.py
      test_hook_handler.py
      test_installer.py
```

### Task Dependency Graph

Implementation order should stay mostly linear:

```text
Task 1 -> Task 2 -> Task 4 -> Task 5 -> Task 6 -> Task 7 -> Task 8
Task 2 -> Task 3 -> Task 4
Task 3 -> Task 5
Task 4 -> Task 9
Task 6 -> Task 9
```

Rationale:

- `config.py` unblocks every other component.
- `privacy.py` should exist before the mapper and client so payload handling is consistent.
- `mapper.py` defines the canonical event contract used by the client and hook handler.
- `hook_handler.py` depends on the mapper and client.
- `installer.py` depends on the hook bundle layout and handler entrypoint.
- observer-plugin mode should come last because it extends the telemetry surface without changing the MVP data path.

## 5. Implementation Plan

### Task 1: Package Skeleton

**Files:**

- Create: `pyproject.toml`
- Create: `src/hermeship/__init__.py`
- Create: `src/hermeship/__main__.py`
- Create: `src/hermeship/cli.py`
- Modify: `README.md`

**Step 1: Write packaging metadata**

Use `hatchling` or setuptools. Keep dependencies minimal:

```toml
[project]
name = "hermeship"
version = "0.1.0"
description = "Hermes adapter for clawhip event notifications"
requires-python = ">=3.11"
dependencies = []

[project.optional-dependencies]
dev = [
  "build>=1.2",
  "pytest>=8",
  "ruff>=0.6",
]

[project.scripts]
hermeship = "hermeship.cli:main"

[tool.pytest.ini_options]
testpaths = ["tests"]
pythonpath = ["src"]
```

**Step 2: Add CLI placeholder**

Initial CLI commands:

```text
hermeship --help
hermeship config path
hermeship install-hook
hermeship emit-sample
hermeship doctor
```

**Step 3: Test**

Run:

```bash
python -m pip install -e ".[dev]"
ruff check .
pytest -q
python -m hermeship --help
```

Expected:

- lint passes
- tests pass
- `python -m hermeship --help` exits 0

**Step 4: Commit**

```bash
git add pyproject.toml README.md src/hermeship
git commit -m "chore: scaffold hermeship package"
```

### Task 2: Config Loader

**Files:**

- Create: `src/hermeship/config.py`
- Create: `tests/test_config.py`

**Step 1: Write failing tests**

Test cases:

```python
def test_default_config_has_safe_values():
    cfg = load_config(None)
    assert cfg.clawhip.mode == "cli"
    assert cfg.clawhip.binary == "clawhip"
    assert cfg.defaults.format == "compact"
    assert cfg.privacy.max_message_chars == 300

def test_config_file_overrides_defaults(tmp_path):
    path = tmp_path / "config.toml"
    path.write_text('[defaults]\nchannel = "123"\nproject = "hermes-dev"\n', encoding="utf-8")
    cfg = load_config(path)
    assert cfg.defaults.channel == "123"
    assert cfg.defaults.project == "hermes-dev"

def test_missing_config_uses_defaults(tmp_path):
    cfg = load_config(tmp_path / "missing.toml")
    assert cfg.defaults.project == "hermes"

def test_env_override_wins(monkeypatch, tmp_path):
    monkeypatch.setenv("HERMESHIP_DRY_RUN", "1")
    cfg = load_config(tmp_path / "missing.toml")
    assert cfg.defaults.dry_run is True
```

**Step 2: Implement dataclasses**

Required dataclasses:

```python
@dataclass(frozen=True)
class ClawhipConfig:
    mode: str = "cli"
    binary: str = "clawhip"
    daemon_base_url: str = "http://127.0.0.1:25294"
    timeout_secs: float = 2.0
```

Also implement `DefaultsConfig`, `EventsConfig`, `PrivacyConfig`, and `HermeshipConfig`.

Include a small `resolve_bool_env()` helper so environment overrides do not depend on ad hoc string comparisons scattered across the codebase.

**Step 3: Implement TOML parsing**

Use stdlib `tomllib` on Python 3.11+.

Support:

```python
def default_config_path() -> Path
def load_config(path: Path | None = None) -> HermeshipConfig
```

If both a repo-local config and the user config exist, merge them shallowly by section. Avoid deep merge semantics unless a concrete use case appears; shallow-by-section is easy to reason about and reduces surprising precedence bugs.

**Step 4: Test**

Run:

```bash
pytest tests/test_config.py -q
```

Expected: all tests pass.

**Step 5: Commit**

```bash
git add src/hermeship/config.py tests/test_config.py
git commit -m "feat: add hermeship config loader"
```

### Task 3: Privacy And Redaction

**Files:**

- Create: `src/hermeship/privacy.py`
- Create: `tests/test_privacy.py`

**Step 1: Write failing tests**

Test:

```python
def test_redacts_sensitive_keys_recursively():
    payload = {"token": "abc", "nested": {"api_key": "secret", "ok": "value"}}
    assert redact_payload(payload, ["token", "api_key"]) == {
        "token": "[redacted]",
        "nested": {"api_key": "[redacted]", "ok": "value"},
    }

def test_truncates_long_strings():
    assert truncate_text("abcdef", 3) == "abc..."
```

**Step 2: Implement**

Functions:

```python
def truncate_text(value: str | None, max_chars: int) -> str | None
def redact_payload(value: Any, redact_keys: Sequence[str]) -> Any
```

Rules:

- Redaction key match is case-insensitive.
- Preserve JSON-compatible containers.
- Truncation appends `...` only when text exceeded max length.

**Step 3: Test**

```bash
pytest tests/test_privacy.py -q
```

Expected: all tests pass.

**Step 4: Commit**

```bash
git add src/hermeship/privacy.py tests/test_privacy.py
git commit -m "feat: add payload redaction helpers"
```

### Task 4: Hermes Event Model And Mapper

**Files:**

- Create: `src/hermeship/events.py`
- Create: `src/hermeship/mapper.py`
- Create: `tests/test_mapper.py`
- Create: `docs/event-mapping.md`

**Step 1: Write failing tests**

Test gateway hook contexts:

```python
def test_agent_start_maps_to_clawhip_agent_started():
    event = map_hermes_event(
        "agent:start",
        {
            "platform": "discord",
            "user_id": "u1",
            "chat_id": "c1",
            "session_id": "s1",
            "message": "fix issue",
        },
        default_project="hermes",
    )
    assert event.kind == "agent.started"
    assert event.agent_name == "hermes"
    assert event.session_id == "s1"
    assert event.summary == "fix issue"

def test_agent_end_maps_to_finished():
    event = map_hermes_event(
        "agent:end",
        {"session_id": "s1", "response": "done"},
        default_project="hermes",
    )
    assert event.kind == "agent.finished"
    assert event.summary == "done"

def test_session_start_maps_to_custom_emit():
    event = map_hermes_event("session:start", {"session_id": "s1"}, default_project="hermes")
    assert event.kind == "hermes.session.started"

def test_agent_end_with_error_maps_to_failed():
    event = map_hermes_event(
        "agent:end",
        {"session_id": "s1", "error": "boom", "message": "ignored"},
        default_project="hermes",
    )
    assert event.kind == "agent.failed"
    assert event.error_message == "boom"
```

**Step 2: Implement model**

Implement:

```python
@dataclass(frozen=True)
class MappedEvent:
    kind: str
    agent_name: str = "hermes"
    session_id: str | None = None
    project: str | None = None
    summary: str | None = None
    error_message: str | None = None
    channel: str | None = None
    mention: str | None = None
    payload: dict[str, Any] = field(default_factory=dict)
```

**Step 3: Implement mapping**

Function:

```python
def map_hermes_event(
    event_type: str,
    context: Mapping[str, Any],
    *,
    default_project: str,
    channel: str | None = None,
    mention: str | None = None,
    privacy: PrivacyConfig | None = None,
) -> MappedEvent | None:
```

Return `None` for disabled/unknown events.

Recommended mapping rules:

- `gateway:startup` -> `hermes.gateway.started`
- `session:start` -> `hermes.session.started`
- `session:end` -> `hermes.session.finished`
- `session:reset` -> `hermes.session.reset`
- `agent:start` -> `agent.started`
- `agent:end` with `error`/`exception` -> `agent.failed`
- `agent:end` otherwise -> `agent.finished`

Payload construction should include only normalized, redacted, and truncated fields:

- `session_id`
- `platform`
- `chat_id`
- `thread_id`
- `user_id`
- `project`
- `summary`
- `error_message`
- `provider = hermes`
- `origin = hermeship`

If a context field is absent, omit it from the payload instead of storing empty strings.

**Step 4: Document mapping**

`docs/event-mapping.md` must contain:

- supported Hermes events
- clawhip event output
- payload fields
- privacy rules
- compatibility notes

**Step 5: Test**

```bash
pytest tests/test_mapper.py -q
```

Expected: all tests pass.

**Step 6: Commit**

```bash
git add src/hermeship/events.py src/hermeship/mapper.py tests/test_mapper.py docs/event-mapping.md
git commit -m "feat: map Hermes hooks to clawhip events"
```

### Task 5: clawhip Client

**Files:**

- Create: `src/hermeship/clawhip_client.py`
- Create: `tests/test_clawhip_client.py`

**Step 1: Write failing tests for CLI command generation**

Use a fake runner function:

```python
def test_agent_started_cli_command():
    calls = []
    client = ClawhipClient(binary="clawhip", runner=lambda cmd: calls.append(cmd))
    client.send(MappedEvent(kind="agent.started", session_id="s1", project="hermes", summary="started"))
    assert calls == [[
        "clawhip", "agent", "started",
        "--name", "hermes",
        "--session", "s1",
        "--project", "hermes",
        "--summary", "started",
    ]]
```

**Step 2: Write failing tests for emit command generation**

```python
def test_custom_emit_cli_command_includes_payload():
    calls = []
    client = ClawhipClient(binary="clawhip", runner=lambda cmd: calls.append(cmd))
    client.send(MappedEvent(kind="hermes.session.started", payload={"provider": "hermes"}))
    assert calls[0][:3] == ["clawhip", "emit", "hermes.session.started"]
```

**Step 3: Implement CLI mode**

Implement:

```python
class ClawhipClient:
    def __init__(self, binary: str = "clawhip", runner: Callable[[list[str]], None] | None = None): ...
    def send(self, event: MappedEvent) -> None: ...
```

Use `subprocess.run(..., check=False, timeout=timeout)` and fail open by default.

Transport semantics:

- Capture `stdout` and `stderr` for diagnostics.
- Never raise on non-zero exit status from clawhip; return a warning and keep Hermes alive.
- Distinguish `missing binary`, `timeout`, and `non-zero exit` because they imply different operator actions.
- In dry-run mode, print the exact command line plus the JSON payload that would have been sent.

Mapping:

- `agent.started` -> `clawhip agent started`
- `agent.finished` -> `clawhip agent finished`
- `agent.blocked` -> `clawhip agent blocked`
- `agent.failed` -> `clawhip agent failed --error ...`
- other kinds -> `clawhip emit <kind> --payload <json>`

If `channel` exists, append `--channel`.
If `mention` exists, append `--mention`.

For custom events, preserve the mapped kind as the clawhip event type and pass the payload through `--payload` or stdin only if the payload cannot be rendered as a finite shell-safe command line. Prefer JSON stdin for richer payloads to avoid shell quoting bugs.

**Step 4: HTTP mode later**

Do not implement HTTP in MVP unless CLI mode is complete. Add interface seam:

```python
class ClawhipTransport(Protocol):
    def send(self, event: MappedEvent) -> None: ...
```

**Step 5: Test**

```bash
pytest tests/test_clawhip_client.py -q
```

Expected: all tests pass.

**Step 6: Commit**

```bash
git add src/hermeship/clawhip_client.py tests/test_clawhip_client.py
git commit -m "feat: send mapped events to clawhip"
```

### Task 6: Hermes Hook Handler

**Files:**

- Create: `src/hermeship/hook_handler.py`
- Create: `templates/hermes-hook/HOOK.yaml`
- Create: `templates/hermes-hook/handler.py`
- Create: `tests/test_hook_handler.py`

**Step 1: Write hook manifest**

`templates/hermes-hook/HOOK.yaml`:

```yaml
name: hermeship-clawhip
description: Forward Hermes lifecycle events to clawhip.
events:
  - gateway:startup
  - session:start
  - session:end
  - session:reset
  - agent:start
  - agent:end
```

Do not enable `command:*` by default. It can be noisy and may leak intent.

The handler should be installed under a dedicated hook directory name so it can be cleanly removed or replaced without touching other Hermes hooks.

**Step 2: Write template handler**

`templates/hermes-hook/handler.py` should be thin:

```python
from hermeship.hook_handler import handle
```

This lets installed hooks use the package implementation.

**Step 3: Write failing tests**

```python
def test_handle_forwards_mapped_event(monkeypatch):
    sent = []
    monkeypatch.setattr("hermeship.hook_handler.send_event", lambda event, cfg: sent.append(event))
    handle("agent:start", {"session_id": "s1", "message": "hello"})
    assert sent[0].kind == "agent.started"
```

**Step 4: Implement handler**

Function:

```python
def handle(event_type: str, context: dict[str, Any]) -> None:
    cfg = load_config()
    if not event_enabled(cfg, event_type):
        return
    mapped = map_hermes_event(...)
    if mapped is None:
        return
    ClawhipClient.from_config(cfg).send(mapped)
```

Rules:

- Catch all exceptions.
- Print a short warning to stderr.
- Never raise into Hermes.
- Honor dry-run by printing JSON instead of calling clawhip.
- Ignore events that already carry `origin=hermeship` to prevent recursive loops.
- If the same lifecycle event is observed repeatedly within the dedupe window, suppress duplicates while still logging one compact note.

**Step 5: Test**

```bash
pytest tests/test_hook_handler.py -q
```

Expected: all tests pass.

**Step 6: Commit**

```bash
git add src/hermeship/hook_handler.py templates/hermes-hook tests/test_hook_handler.py
git commit -m "feat: add Hermes gateway hook handler"
```

### Task 7: Installer

**Files:**

- Create: `src/hermeship/installer.py`
- Extend: `src/hermeship/cli.py`
- Create: `tests/test_installer.py`
- Create: `docs/operations.md`

**Step 1: Write failing tests**

```python
def test_install_hook_writes_manifest_and_handler(tmp_path):
    home = tmp_path / ".hermes"
    install_hook(home=home, force=False)
    assert (home / "hooks/hermeship-clawhip/HOOK.yaml").is_file()
    assert (home / "hooks/hermeship-clawhip/handler.py").is_file()
```

Test no overwrite:

```python
def test_install_hook_refuses_existing_without_force(tmp_path):
    hook_dir = tmp_path / ".hermes/hooks/hermeship-clawhip"
    hook_dir.mkdir(parents=True)
    (hook_dir / "handler.py").write_text("# custom", encoding="utf-8")
    with pytest.raises(FileExistsError):
        install_hook(home=tmp_path / ".hermes", force=False)

def test_install_hook_is_idempotent_with_force(tmp_path):
    home = tmp_path / ".hermes"
    install_hook(home=home, force=True)
    install_hook(home=home, force=True)
    assert (home / "hooks/hermeship-clawhip/HOOK.yaml").is_file()
```

**Step 2: Implement installer**

CLI:

```bash
hermeship install-hook
hermeship install-hook --home ~/.hermes
hermeship install-hook --force
```

Implementation:

```python
def install_hook(home: Path, force: bool = False) -> Path:
    ...
```

Implementation details:

- Create parent directories with explicit permissions if the platform supports them.
- Copy the template hook files from `templates/hermes-hook/`.
- Refuse to overwrite existing files unless `force=True`.
- Preserve a small backup copy or write an install manifest when replacing an existing hook bundle.
- Return the final hook directory path for CLI reporting.

**Step 3: Implement doctor**

`hermeship doctor` checks:

- Python package import works.
- `~/.hermes/hooks/hermeship-clawhip/HOOK.yaml` exists.
- `clawhip` binary is on PATH or configured.
- `clawhip status` exits 0, or reports daemon unavailable clearly.
- config file path and effective mode.

**Step 4: Write operations docs**

`docs/operations.md` must cover:

```bash
pip install -e .
hermeship install-hook
hermeship doctor
clawhip start
hermeship emit-sample
```

Also include rollback:

```bash
rm -rf ~/.hermes/hooks/hermeship-clawhip
```

**Step 5: Test**

```bash
pytest tests/test_installer.py -q
python -m hermeship install-hook --home /tmp/hermeship-test-home --force
python -m hermeship doctor
```

Expected:

- tests pass
- install writes hook files
- doctor prints actionable status

**Step 6: Commit**

```bash
git add src/hermeship/installer.py src/hermeship/cli.py tests/test_installer.py docs/operations.md
git commit -m "feat: install Hermes hook bundle"
```

### Task 8: Sample Event And Live Verification

**Files:**

- Extend: `src/hermeship/cli.py`
- Create: `docs/live-verification.md`
- Extend: `README.md`

**Step 1: Implement `emit-sample`**

CLI:

```bash
hermeship emit-sample --event agent:start
hermeship emit-sample --event agent:end
hermeship emit-sample --dry-run
```

Sample context:

```json
{
  "platform": "discord",
  "user_id": "local-test",
  "chat_id": "dev-channel",
  "thread_id": "",
  "session_id": "hermeship-local-test",
  "message": "Hermeship live verification"
}
```

**Step 2: Document live verification**

`docs/live-verification.md`:

```bash
clawhip start
hermeship doctor
hermeship emit-sample --event agent:start
hermeship emit-sample --event agent:end
```

Expected Discord outputs:

- compact `agent started` message
- compact `agent finished` message

Also verify the `session:start` and `session:end` custom events in a debug channel so both the lifecycle and adapter-specific custom routes are exercised.

**Step 3: Test with dry-run**

```bash
python -m hermeship emit-sample --event agent:start --dry-run
```

Expected:

- JSON printed
- no clawhip process invoked

**Step 4: Commit**

```bash
git add src/hermeship/cli.py README.md docs/live-verification.md
git commit -m "docs: add Hermeship live verification flow"
```

### Task 9: Optional Observer Plugin Mode

Do not implement this before MVP is working.

**Files:**

- Create: `src/hermeship/observer_plugin.py`
- Create: `templates/hermes-plugin/hermeship_observer.py`
- Create: `tests/test_observer_plugin.py`
- Create: `docs/observer-plugin.md`

**Purpose**

Gateway hooks provide coarse lifecycle events. Observer hooks can provide:

- `on_session_start`
- `on_session_end`
- `pre_tool_call`
- `post_tool_call`
- `pre_api_request`
- `post_api_request`
- `api_request_error`
- `subagent_start`
- `subagent_stop`

**Design rule**

Observer mode must be read-only and fail-open. It must not use Hermes middleware.

**Mapping**

| Hermes observer hook | clawhip event |
| --- | --- |
| `on_session_start` | `hermes.session.started` |
| `on_session_end` | `hermes.session.finished` |
| `pre_tool_call` | `hermes.tool.started` |
| `post_tool_call` | `hermes.tool.finished` or `hermes.tool.failed` |
| `api_request_error` | `hermes.api.failed` |
| `subagent_start` | `agent.started` with `agent_name=hermes-subagent` |
| `subagent_stop` | `agent.finished` or `agent.failed` |

**Acceptance**

- Observer plugin can be enabled without gateway hook.
- No full prompt or conversation history is sent by default.
- API request/response bodies are dropped unless explicit debug mode is enabled.

## 6. Failure Handling

Hermeship must be fail-open:

- If `clawhip` binary is missing, log once and continue.
- If clawhip daemon is down, log a short warning and continue.
- If mapping fails, log event type and continue.
- If config is invalid, use defaults where safe; if parsing fully fails, disable forwarding and warn.

No Hermes hook handler should raise an exception back into Hermes.

Operationally useful failure categories:

| Failure | User-visible behavior | Internal action |
| --- | --- | --- |
| Missing config | use defaults or disable forwarding | emit one warning with config path |
| Invalid config | disable forwarding | keep Hermes alive |
| Missing clawhip binary | skip send | log once per session |
| clawhip timeout | skip send | record command, timeout, session_id |
| clawhip non-zero exit | skip send | capture stderr tail for operator |
| serialization failure | send truncated fallback or skip | log mapper failure with event type |
| duplicate hook event | suppress duplicate | record dedupe hit |
| recursion detected | drop event | mark origin as hermeship |

## 7. Security And Privacy Requirements

Mandatory:

- Do not send `conversation_history`.
- Do not send full provider request/response payloads.
- Redact recursive keys matching:
  - `token`
  - `api_key`
  - `authorization`
  - `password`
  - `secret`
  - `cookie`
- Truncate `message` and `response`.
- Disable `command:*` by default.
- Do not introduce a new Hermes core model tool.
- Do not ask users to reuse Hermes/Discord bot tokens for clawhip notification bot.

Recommended:

- Use a dedicated clawhip Discord bot token configured in clawhip.
- Use path-based routing in clawhip when repo/worktree metadata exists.
- Use raw format only during local debugging.

Performance expectations:

- Hook handler local mapping should complete in under 50 ms.
- `clawhip` subprocess delivery should use a short timeout, defaulting to 2 seconds.
- Deduplication state should be in-memory and bounded.
- Do not perform network calls from the Hermes hook handler except through the configured clawhip transport.
- Do not block Hermes shutdown on pending notification sends.

## 8. clawhip Route Examples

Basic route:

```toml
[[routes]]
event = "agent.*"
filter = { project = "hermes" }
channel = "DISCORD_CHANNEL_ID"
format = "compact"
```

Hermes-specific custom events:

```toml
[[routes]]
event = "hermes.*"
filter = { provider = "hermes" }
channel = "DISCORD_CHANNEL_ID"
format = "compact"
```

Debug route:

```toml
[[routes]]
event = "hermes.*"
filter = { provider = "hermes" }
channel = "DISCORD_DEBUG_CHANNEL_ID"
format = "raw"
```

## 9. End-To-End Verification

### Local Unit Verification

Run:

```bash
pytest -q
python -m hermeship --help
python -m hermeship emit-sample --event agent:start --dry-run
```

Expected:

- all tests pass
- CLI help exits 0
- dry-run prints mapped event JSON

Add a CI job that runs these tests against a fake `clawhip` binary on `PATH` so the command-generation logic is exercised without needing a live daemon.

### clawhip Integration Verification

Precondition:

```bash
clawhip start
clawhip status
```

Run:

```bash
hermeship emit-sample --event agent:start
hermeship emit-sample --event agent:end
```

Expected:

- `clawhip status` reports daemon healthy
- Discord receives start/end messages

Use a short-lived test channel or a private verification channel, and keep the verification payload intentionally small so the live check is safe to repeat.

### Hermes Hook Verification

Use isolated Hermes home:

```bash
export HERMES_HOME=/tmp/hermeship-hermes-home
mkdir -p "$HERMES_HOME"
hermeship install-hook --home "$HERMES_HOME" --force
find "$HERMES_HOME/hooks/hermeship-clawhip" -maxdepth 1 -type f -print
```

Expected:

```text
HOOK.yaml
handler.py
```

Then run a minimal Hermes gateway or CLI session using the same `HERMES_HOME` and confirm:

- hook loads
- `agent:start` fires
- `agent:end` fires
- clawhip receives both

If a full Hermes gateway session is too heavy for CI, document it as a manual live verification step and keep unit tests around the handler.

For automated coverage, prefer a hermes hook smoke test that imports `hermeship.hook_handler.handle()` directly with synthetic event contexts instead of trying to drive a full Hermes GUI or chat loop in CI.

### Test Matrix

| Layer | Tests | Required before merge |
| --- | --- | --- |
| Config | defaults, env overrides, malformed TOML, repo-local override | yes |
| Privacy | recursive redaction, truncation, non-string payloads, large payloads | yes |
| Mapper | every supported Hermes event, missing fields, error fields, disabled event | yes |
| Client | CLI command generation, dry-run, timeout, missing binary, non-zero exit | yes |
| Hook handler | fail-open behavior, duplicate suppression, recursion suppression | yes |
| Installer | first install, no-overwrite, force install, rollback path | yes |
| Live clawhip | sample event reaches daemon | manual or integration |
| Live Discord | message appears in verification channel | manual |
| Hermes gateway | hook loads in isolated `HERMES_HOME` | manual or smoke |

### CI Shape

Recommended CI commands:

```bash
python -m pip install -e ".[dev]"
ruff check .
pytest -q
python -m hermeship --help
python -m hermeship emit-sample --event agent:start --dry-run
```

If `ruff` is not adopted immediately, keep the command documented and add it before the first tagged release.

## 10. Compatibility Policy

Hermeship owns:

- Hermes hook installation
- Hermes event mapping
- clawhip client call shape
- privacy and redaction

clawhip owns:

- daemon
- routing
- rendering
- Discord delivery
- Git/GitHub/tmux sources
- Codex/Claude native hook v1 contract

Hermes owns:

- gateway lifecycle
- hook discovery
- observer hooks
- session semantics
- plugin/middleware runtime

Do not patch Hermes or clawhip for MVP. If a missing capability is found, first implement around existing public surfaces. Only propose upstream changes after a live MVP proves the need.

## 11. Milestones

### Milestone 1: Local Dry-Run

Complete Tasks 1-4.

Acceptance:

- `pytest -q` passes.
- `hermeship emit-sample --dry-run` prints correct mapped events.
- `docs/event-mapping.md` exists.
- config defaults, redaction, and mapper behavior are locked by tests.

### Milestone 2: clawhip CLI Delivery

Complete Task 5.

Acceptance:

- `hermeship emit-sample --event agent:start` calls `clawhip agent started`.
- Missing clawhip binary does not crash the process.
- non-zero clawhip exit is reported as a warning, not a hard failure.

### Milestone 3: Hermes Hook Install

Complete Tasks 6-7.

Acceptance:

- `hermeship install-hook` writes a valid Hermes hook bundle.
- Handler can be imported by Hermes.
- Handler forwards synthetic `agent:start` and `agent:end` events.
- rerunning install with `--force` preserves correctness and does not corrupt the hook bundle.

### Milestone 4: Live Discord Verification

Complete Task 8.

Acceptance:

- clawhip daemon receives Hermeship sample events.
- Discord channel shows the messages through clawhip.
- README documents install, verify, and rollback.
- verification uses a dedicated channel and clearly scoped payloads.

### Milestone 5: Observer Plugin Research

Complete Task 9 only if gateway hook MVP is stable.

Acceptance:

- Observer plugin maps tool and API telemetry without changing Hermes behavior.
- Privacy tests prove prompt/request bodies are not forwarded by default.

## 12. Open Decisions

Defer these until after Milestone 1:

- Whether to support HTTP mode in MVP or keep CLI-only.
- Whether to expose `command:*` events.
- Whether `session:start` should be routed as custom `hermes.session.started` or as `agent.started`.
- Whether to upstream a `hermes` provider into clawhip's native hook contract.
- Whether Hermeship should include a clawhip plugin manifest later.

Recommended defaults:

- CLI-only for MVP.
- `command:*` disabled.
- Session events use `hermes.session.*`.
- Agent events use clawhip `agent.*`.
- No upstream clawhip changes until Hermeship proves useful.

## 13. Release And Rollback Strategy

### Versioning

Use semantic versioning:

- `0.1.0`: CLI transport, hook install, dry-run, sample events.
- `0.2.0`: live clawhip verification docs and improved diagnostics.
- `0.3.0`: optional observer plugin research, if MVP is stable.
- `1.0.0`: stable config schema, stable event mapping, documented rollback, live verification complete.

Do not tag `1.0.0` until the gateway hook path has been verified against a real Hermes installation and a real clawhip daemon.

### Release Checklist

Before tagging:

```bash
ruff check .
pytest -q
python -m hermeship --help
python -m hermeship emit-sample --event agent:start --dry-run
python -m build
```

Check:

- README install commands match the current CLI.
- `docs/operations.md` includes setup, verify, upgrade, and rollback.
- `docs/event-mapping.md` reflects the code.
- no dependency was added without a clear reason.
- no secret-like test fixture appears in git.

### Upgrade Path

For an existing user:

```bash
python -m pip install -U hermeship
hermeship install-hook --force
hermeship doctor
hermeship emit-sample --event agent:start --dry-run
```

If the installed hook files change format, `install-hook --force` should either preserve a backup or print the overwritten file paths.

### Rollback Path

Rollback should not require touching clawhip:

```bash
rm -rf ~/.hermes/hooks/hermeship-clawhip
python -m pip uninstall hermeship
```

Then restart Hermes if it keeps hooks loaded for the process lifetime.

### Upstream Contribution Policy

Only propose changes to `clawhip` after the MVP proves a missing generic capability. Candidate upstream changes must be small and generic:

- a better documented `emit --payload` path
- a provider-agnostic event ingress endpoint
- renderer support for `hermes.*` custom events

Avoid upstreaming Hermes-specific assumptions into clawhip's Codex/Claude native hook v1 contract.

## 14. Final Review Checklist

Before claiming implementation complete:

- [ ] `pytest -q` passes.
- [ ] `ruff check .` passes or is documented as not yet adopted.
- [ ] `python -m hermeship --help` exits 0.
- [ ] `python -m hermeship emit-sample --event agent:start --dry-run` prints mapped JSON.
- [ ] `hermeship install-hook --home /tmp/hermeship-test-home --force` writes hook files.
- [ ] `clawhip status` tested against a running daemon.
- [ ] Live Discord delivery verified or explicitly documented as not run.
- [ ] No full message history or secrets appear in captured logs.
- [ ] README and operations docs match the actual commands.
- [ ] rollback path has been tested in a disposable `HERMES_HOME`.
