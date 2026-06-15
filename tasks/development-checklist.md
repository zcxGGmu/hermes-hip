# Hermeship Development Checklist

This checklist tracks iterative development for `hermeship`, the Hermes adapter for `clawhip`.

Source plan: `docs/plans/2026-06-15-hermeship-development-plan.md`

## Tracking Rules

- [ ] Keep one active milestone at a time.
- [ ] Update this checklist after every implementation session.
- [ ] Mark a task complete only after its verification command has run.
- [ ] Commit at the end of each task group, not only at the end of a milestone.
- [ ] Do not start observer-plugin work until the gateway-hook MVP is live-verified.
- [ ] Do not patch Hermes or clawhip during MVP unless this checklist is explicitly revised.
- [ ] Record skipped live checks with the exact reason and what remains unverified.

## Global Definition Of Done

- [ ] `pytest -q` passes.
- [ ] `ruff check .` passes, or the reason ruff is not yet adopted is documented.
- [ ] `python -m hermeship --help` exits 0.
- [ ] `python -m hermeship emit-sample --event agent:start --dry-run` prints mapped JSON.
- [ ] `hermeship install-hook --home /tmp/hermeship-test-home --force` writes hook files.
- [ ] `clawhip status` has been tested against a running daemon.
- [ ] Live Discord delivery is verified or explicitly documented as not run.
- [ ] Rollback path is tested in a disposable `HERMES_HOME`.
- [ ] No full conversation history, prompt body, provider request/response, token, cookie, or secret appears in logs or fixtures.
- [ ] README and docs match the actual CLI.

## Milestone 0: Repository Hygiene

Goal: make the repository ready for test-driven implementation.

- [ ] Confirm current branch and remote.
  - Run: `git status --short --branch`
  - Done when: branch is known and unrelated local changes are documented.
- [ ] Confirm existing files.
  - Run: `find . -maxdepth 3 -type f | sort`
  - Done when: implementation will not overwrite unrelated files.
- [ ] Review implementation plan.
  - Read: `docs/plans/2026-06-15-hermeship-development-plan.md`
  - Done when: current task sequence matches this checklist.
- [ ] Create or update project README purpose section.
  - File: `README.md`
  - Done when: README states Hermeship is a Hermes-to-clawhip adapter, not a clawhip fork.
- [ ] Commit hygiene baseline if needed.
  - Run: `git status --short`
  - Done when: doc-only planning state is clear before implementation starts.

## Milestone 1: Local Dry-Run Foundation

Goal: package, config, privacy, and mapping work locally without a clawhip daemon.

### Task 1.1: Package Skeleton

- [ ] Write failing CLI import/help smoke test.
  - Create: `tests/test_cli.py`
  - Verify fail: `pytest tests/test_cli.py -q`
- [ ] Create packaging metadata.
  - Create: `pyproject.toml`
  - Include: package metadata, console script, dev extras, pytest config.
- [ ] Create Python package entrypoints.
  - Create: `src/hermeship/__init__.py`
  - Create: `src/hermeship/__main__.py`
  - Create: `src/hermeship/cli.py`
- [ ] Implement minimal `main()` and `--help`.
  - Done when: `python -m hermeship --help` exits 0.
- [ ] Update README quickstart placeholder.
  - Modify: `README.md`
- [ ] Verify Task 1.1.
  - Run: `python -m pip install -e ".[dev]"`
  - Run: `pytest tests/test_cli.py -q`
  - Run: `python -m hermeship --help`
- [ ] Commit Task 1.1.
  - Commit: `chore: scaffold hermeship package`

### Task 1.2: Config Loader

- [ ] Write config default tests.
  - Create: `tests/test_config.py`
  - Cases: missing file, default values, empty strings normalize to `None`.
- [ ] Write config override tests.
  - Cases: user config, env override, malformed TOML, unknown keys warning/strict mode.
- [ ] Implement config dataclasses.
  - Create: `src/hermeship/config.py`
  - Include: `ClawhipConfig`, `DefaultsConfig`, `EventsConfig`, `PrivacyConfig`, `HermeshipConfig`.
- [ ] Implement config path resolution.
  - Include: `default_config_path()`, optional `repo_config_path()`, `load_config()`.
- [ ] Implement env override helpers.
  - Include: boolean parsing for `HERMESHIP_DRY_RUN`.
- [ ] Implement validation.
  - Validate: `mode`, `format`, `timeout_secs`, `dedupe_window_secs`, unknown keys.
- [ ] Verify Task 1.2.
  - Run: `pytest tests/test_config.py -q`
- [ ] Commit Task 1.2.
  - Commit: `feat: add hermeship config loader`

### Task 1.3: Privacy And Redaction

- [ ] Write redaction tests.
  - Create: `tests/test_privacy.py`
  - Cases: nested dicts, lists, case-insensitive keys, non-string values.
- [ ] Write truncation tests.
  - Cases: no truncation, exact length, long text, `None`.
- [ ] Implement privacy helpers.
  - Create: `src/hermeship/privacy.py`
  - Functions: `truncate_text()`, `redact_payload()`, optional `sanitize_context()`.
- [ ] Verify no mutation of input payload.
  - Test: original input remains unchanged after redaction.
- [ ] Verify Task 1.3.
  - Run: `pytest tests/test_privacy.py -q`
- [ ] Commit Task 1.3.
  - Commit: `feat: add payload privacy helpers`

### Task 1.4: Event Model And Mapper

- [ ] Write mapper tests for supported lifecycle events.
  - Create: `tests/test_mapper.py`
  - Cover: `gateway:startup`, `session:start`, `session:end`, `session:reset`, `agent:start`, `agent:end`.
- [ ] Write mapper tests for error handling.
  - Cover: `agent:end` with `error`, `exception`, `status=failed`.
- [ ] Write mapper tests for disabled/unknown events.
  - Cover: event disabled by config, unknown event returns `None`.
- [ ] Write mapper tests for privacy.
  - Cover: message/response truncation, secret redaction, no `conversation_history`.
- [ ] Implement event dataclasses.
  - Create: `src/hermeship/events.py`
  - Include: `MappedEvent`.
- [ ] Implement mapper.
  - Create: `src/hermeship/mapper.py`
  - Include: `map_hermes_event()`, event-enabled helper.
- [ ] Add event mapping docs.
  - Create: `docs/event-mapping.md`
  - Include: supported events, output events, payload fields, privacy policy.
- [ ] Verify Task 1.4.
  - Run: `pytest tests/test_mapper.py -q`
- [ ] Verify Milestone 1.
  - Run: `pytest -q`
  - Run: `python -m hermeship --help`
- [ ] Commit Task 1.4.
  - Commit: `feat: map Hermes lifecycle events`

## Milestone 2: clawhip CLI Delivery

Goal: mapped events can be sent to clawhip through stable existing clawhip CLI surfaces.

### Task 2.1: Client Contract And Command Generation

- [ ] Write client tests with fake runner.
  - Create: `tests/test_clawhip_client.py`
  - Cover: `agent.started`, `agent.finished`, `agent.blocked`, `agent.failed`.
- [ ] Write custom emit command tests.
  - Cover: `hermes.session.started`, payload JSON, channel, mention.
- [ ] Implement client class.
  - Create: `src/hermeship/clawhip_client.py`
  - Include: `ClawhipClient`, runner seam, timeout, dry-run.
- [ ] Implement command builder without shell.
  - Use: `subprocess.run(list[str], shell=False, ...)`
- [ ] Verify Task 2.1.
  - Run: `pytest tests/test_clawhip_client.py -q`
- [ ] Commit Task 2.1.
  - Commit: `feat: build clawhip delivery commands`

### Task 2.2: Failure Semantics

- [ ] Test missing binary.
  - Expected: warning result, no exception.
- [ ] Test timeout.
  - Expected: warning result, no exception.
- [ ] Test non-zero exit.
  - Expected: stderr tail captured, no exception.
- [ ] Test serialization failure fallback.
  - Expected: event skipped with diagnostic.
- [ ] Implement structured send result.
  - Include: `sent`, `skipped`, `reason`, `stderr_tail`, `command`.
- [ ] Verify Task 2.2.
  - Run: `pytest tests/test_clawhip_client.py -q`
- [ ] Commit Task 2.2.
  - Commit: `feat: make clawhip delivery fail open`

### Task 2.3: Dry-Run And Sample Event

- [ ] Add CLI `emit-sample`.
  - Modify: `src/hermeship/cli.py`
  - Flags: `--event`, `--dry-run`, `--channel`, `--project`.
- [ ] Test dry-run emits JSON.
  - Cover: no subprocess call.
- [ ] Test sample event mapping.
  - Cover: `agent:start`, `agent:end`, `session:start`, `session:end`.
- [ ] Verify Task 2.3.
  - Run: `pytest tests/test_cli.py tests/test_clawhip_client.py -q`
  - Run: `python -m hermeship emit-sample --event agent:start --dry-run`
- [ ] Verify Milestone 2.
  - Run: `pytest -q`
  - Run: `python -m hermeship emit-sample --event agent:start --dry-run`
- [ ] Commit Task 2.3.
  - Commit: `feat: add dry-run sample events`

## Milestone 3: Hermes Hook Installation

Goal: Hermeship installs a fail-open Hermes gateway hook bundle.

### Task 3.1: Hook Template

- [ ] Create hook manifest template.
  - Create: `templates/hermes-hook/HOOK.yaml`
  - Events: `gateway:startup`, `session:start`, `session:end`, `session:reset`, `agent:start`, `agent:end`.
- [ ] Create hook handler template.
  - Create: `templates/hermes-hook/handler.py`
  - Content: import and expose `hermeship.hook_handler.handle`.
- [ ] Write template integrity tests.
  - Create: `tests/test_hook_template.py`
  - Cover: manifest valid YAML shape if YAML parser is available, or basic content checks.
- [ ] Verify Task 3.1.
  - Run: `pytest tests/test_hook_template.py -q`
- [ ] Commit Task 3.1.
  - Commit: `feat: add Hermes hook template`

### Task 3.2: Hook Runtime Handler

- [ ] Write handler forwarding tests.
  - Create: `tests/test_hook_handler.py`
  - Cover: `agent:start` forwards mapped event.
- [ ] Write fail-open tests.
  - Cover: config parse failure, mapper failure, client failure.
- [ ] Write recursion suppression tests.
  - Cover: context with `origin=hermeship` is ignored.
- [ ] Write dedupe tests.
  - Cover: same event/session within `dedupe_window_secs` suppresses duplicate.
- [ ] Implement runtime handler.
  - Create: `src/hermeship/hook_handler.py`
  - Include: `handle(event_type, context)`, `send_event()`, dedupe state.
- [ ] Verify Task 3.2.
  - Run: `pytest tests/test_hook_handler.py -q`
- [ ] Commit Task 3.2.
  - Commit: `feat: add fail-open Hermes hook handler`

### Task 3.3: Installer And Doctor

- [ ] Write installer tests.
  - Create: `tests/test_installer.py`
  - Cover: first install, no-overwrite, force install, returned path.
- [ ] Implement installer.
  - Create: `src/hermeship/installer.py`
  - Function: `install_hook(home: Path, force: bool = False) -> Path`.
- [ ] Add CLI `install-hook`.
  - Modify: `src/hermeship/cli.py`
  - Flags: `--home`, `--force`.
- [ ] Add CLI `doctor`.
  - Checks: package import, hook installed, clawhip binary found, clawhip status.
- [ ] Document operations.
  - Create: `docs/operations.md`
  - Include: install, verify, update, rollback.
- [ ] Verify Task 3.3.
  - Run: `pytest tests/test_installer.py -q`
  - Run: `python -m hermeship install-hook --home /tmp/hermeship-test-home --force`
  - Run: `find /tmp/hermeship-test-home/hooks/hermeship-clawhip -maxdepth 1 -type f -print`
- [ ] Verify Milestone 3.
  - Run: `pytest -q`
  - Run: `python -m hermeship doctor`
- [ ] Commit Task 3.3.
  - Commit: `feat: install Hermes hook bundle`

## Milestone 4: Documentation And Live Verification

Goal: prove Hermeship works with clawhip and document how operators run it safely.

### Task 4.1: README And Operator Docs

- [ ] Rewrite README quickstart.
  - Include: what Hermeship is, install, configure clawhip, install hook, dry-run, live check.
- [ ] Add rollback section.
  - Include: remove hook directory, uninstall package, restart Hermes if needed.
- [ ] Add privacy section.
  - Include: what is never forwarded.
- [ ] Link to plan and event mapping.
  - Files: `README.md`, `docs/event-mapping.md`, `docs/operations.md`.
- [ ] Verify Task 4.1.
  - Run: `rg -n "install-hook|emit-sample|rollback|privacy|clawhip" README.md docs`
- [ ] Commit Task 4.1.
  - Commit: `docs: add Hermeship operator guide`

### Task 4.2: Live Verification Runbook

- [ ] Create live verification doc.
  - Create: `docs/live-verification.md`
- [ ] Document preconditions.
  - Include: running clawhip daemon, configured Discord test channel, dedicated clawhip bot token.
- [ ] Document dry-run verification.
  - Command: `python -m hermeship emit-sample --event agent:start --dry-run`.
- [ ] Document clawhip daemon verification.
  - Commands: `clawhip start`, `clawhip status`, sample events.
- [ ] Document Hermes hook smoke verification.
  - Use isolated `HERMES_HOME`.
- [ ] Document expected Discord messages.
  - Include: start, finish, session started, session finished.
- [ ] Verify Task 4.2.
  - Run: `rg -n "clawhip status|HERMES_HOME|Discord|emit-sample" docs/live-verification.md`
- [ ] Commit Task 4.2.
  - Commit: `docs: add live verification runbook`

### Task 4.3: First Live Check

- [ ] Start or confirm clawhip daemon.
  - Run: `clawhip status`
  - If unavailable: record reason in this checklist.
- [ ] Send dry-run sample.
  - Run: `python -m hermeship emit-sample --event agent:start --dry-run`
- [ ] Send live start sample.
  - Run: `python -m hermeship emit-sample --event agent:start`
- [ ] Send live finish sample.
  - Run: `python -m hermeship emit-sample --event agent:end`
- [ ] Confirm Discord delivery.
  - Record: channel, timestamp, observed message shape.
- [ ] Test rollback in disposable home.
  - Run: `rm -rf /tmp/hermeship-test-home/hooks/hermeship-clawhip`
- [ ] Verify Milestone 4.
  - Run: `pytest -q`
  - Run: `python -m hermeship --help`
- [ ] Commit live-verification notes if docs changed.
  - Commit: `docs: record Hermeship live verification`

## Milestone 5: CI And Release Readiness

Goal: make the MVP repeatable and releasable.

### Task 5.1: Lint And Test Automation

- [ ] Add ruff config.
  - Modify: `pyproject.toml`
- [ ] Ensure all tests pass under editable install.
  - Run: `python -m pip install -e ".[dev]"`
  - Run: `ruff check .`
  - Run: `pytest -q`
- [ ] Add fake clawhip binary test fixture.
  - File: `tests/conftest.py`
  - Purpose: exercise command integration without daemon.
- [ ] Verify Task 5.1.
  - Run: `ruff check .`
  - Run: `pytest -q`
- [ ] Commit Task 5.1.
  - Commit: `test: add lint and fake clawhip coverage`

### Task 5.2: Build And Packaging

- [ ] Add package build verification.
  - Run: `python -m build`
- [ ] Inspect distribution contents.
  - Confirm: hook templates and docs are included.
- [ ] Add packaging tests if templates are missing.
  - Cover: installed package can locate template files.
- [ ] Verify Task 5.2.
  - Run: `python -m build`
  - Run: `python -m pip install dist/*.whl --force-reinstall`
  - Run: `python -m hermeship --help`
- [ ] Commit Task 5.2.
  - Commit: `chore: verify package build artifacts`

### Task 5.3: Release Gate

- [ ] Review config schema stability.
  - Done when: no pending rename of user-facing keys.
- [ ] Review event mapping stability.
  - Done when: event names match `docs/event-mapping.md`.
- [ ] Review privacy defaults.
  - Done when: no high-risk fields forwarded by default.
- [ ] Review rollback.
  - Done when: rollback path works in disposable `HERMES_HOME`.
- [ ] Run full release verification.
  - Run: `ruff check .`
  - Run: `pytest -q`
  - Run: `python -m hermeship emit-sample --event agent:start --dry-run`
  - Run: `python -m build`
- [ ] Commit release readiness update.
  - Commit: `chore: prepare hermeship v0.1.0`

## Milestone 6: Optional Observer Plugin Research

Goal: add higher-fidelity telemetry only after the gateway-hook MVP is stable.

Gate: do not begin until Milestones 1-5 are complete or explicitly waived.

### Task 6.1: Research Observer Hook Runtime

- [ ] Re-read Hermes observer docs.
  - File: `/home/zq/work-space/repo/ai-projs/agents/hermes-agent/docs/observability/README.md`
- [ ] Identify plugin install mechanism.
  - Done when: exact expected plugin directory and enable command are documented.
- [ ] Draft observer event mapping.
  - Create or update: `docs/observer-plugin.md`
- [ ] Decide whether observer mode belongs in v0.2 or later.
  - Record decision in this checklist.

### Task 6.2: Observer Plugin MVP

- [ ] Write observer plugin tests.
  - Create: `tests/test_observer_plugin.py`
- [ ] Implement read-only observer callbacks.
  - Create: `src/hermeship/observer_plugin.py`
  - Create: `templates/hermes-plugin/hermeship_observer.py`
- [ ] Verify privacy defaults.
  - Test: request/response bodies dropped by default.
- [ ] Verify observer mode does not require gateway hook.
  - Run: targeted tests.
- [ ] Commit observer plugin.
  - Commit: `feat: add optional Hermes observer forwarding`

## Running Status Log

Use this section during implementation. Keep newest entries at the top.

### 2026-06-15

- [x] Created initial implementation plan.
- [x] Created iterative development checklist.
- [ ] Implementation not started.

## Blockers

- [ ] Confirm whether live Discord verification credentials are available.
- [ ] Confirm whether `clawhip` binary is installed in the development environment.
- [ ] Confirm whether a real Hermes gateway run is required for v0.1.0 or can be manual post-release verification.

## Decisions

- [x] MVP uses Hermes gateway hooks, not Hermes core patches.
- [x] MVP uses `clawhip agent` and `clawhip emit`, not a new clawhip native provider contract.
- [x] MVP is stdlib-first; HTTP transport is deferred.
- [x] `command:*` forwarding is disabled by default.
