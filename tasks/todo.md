# Task: Milestone 10.2 Observer Plugin MVP Scaffold

更新时间：2026-06-19

本轮任务从当前 `codex/milestone-1-cli` 分支继续 Hermeship 开发，进入 Milestone 10.2 Observer Plugin MVP scaffold。范围限定为可选 Hermes observer plugin 模板、fail-open safe-field forwarding、本地 Python compile/smoke 测试、必要的 release preflight 与状态文档更新。

默认不执行真实 Discord/Hermes live check；只有提供 Discord credentials、测试频道、Hermes gateway 测试环境和明确执行确认时，才补做 Milestone 9.3 live check。默认不实现 Slack sink。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，`git status --short --branch` 只显示分支行。
- 当前 HEAD：`eb64408 docs: 更新 Hermeship 最新开发状态`。
- 最近 5 个提交：`eb64408`、`93aa9ec`、`0d0d354`、`92790ef`、`589c9e2`。
- 最新状态文档提交：`eb64408 docs: 更新 Hermeship 最新开发状态`。
- 最新 Milestone 10.1 契约研究提交：`93aa9ec docs: 完成 Hermes observer plugin 契约研究`。
- Milestone 0 到 8.4、9.1、9.2 已完成并提交。
- Milestone 9.3 已完成 `blocked`/`not_run` 记录；真实 Discord/Hermes live verification 仍未获得 `pass`。
- Milestone 10.1 已完成并提交；Milestone 10.2 本轮启动。

## 本轮执行计划

- [x] 复习 lessons、确认 Git 状态和最近提交。
  - 已读：`tasks/lessons.md`。
  - 命令：`git status --short --branch`。
  - 命令：`git log -5 --oneline`。

- [x] 阅读当前状态与契约入口。
  - 已读：`docs/development-status.md`。
  - 已读：`docs/plans/2026-06-15-hermeship-development-plan.md`。
  - 已读：`tasks/development-checklist.md` 的 Milestone 10 与最新运行日志。
  - 已读：`tasks/todo.md`。
  - 已读：`docs/observer-plugin.md`。
  - 已读：`docs/live-verification.md`。
  - 已读：`README.md`。
  - 已读：`ARCHITECTURE.md`。
  - 已读：`docs/operations.md`。
  - 已读：`docs/hermes-event-contract.md`。
  - 已读：`src/release_preflight.rs`。

- [x] 写 Red 测试锁定 10.2 交付边界。
  - 修改：`src/release_preflight.rs`。
  - 目标：preflight 要求 `templates/hermes-plugin/plugin.yaml` 与 `templates/hermes-plugin/__init__.py` 存在，并包含 observer plugin 契约关键字。
  - 目标：测试 fixture 能在缺少 observer plugin 模板时失败。
  - 目标：保持 live verification check 只验证文档字段，不代表真实 live pass。
  - Red 命令：`cargo test release_preflight::tests::preflight_fails_when_observer_plugin_template_is_missing`。
  - Red 结果：实现前失败于 `assertion failed: !report.ok()`。
  - Green 结果：`cargo test observer_plugin` 通过。

- [x] 创建可选 Hermes observer plugin 模板。
  - 新建：`templates/hermes-plugin/plugin.yaml`。
  - 新建：`templates/hermes-plugin/__init__.py`。
  - 目标：目录 plugin 满足 Hermes `plugin.yaml` + `__init__.py` + `register(ctx)` 结构。
  - 目标：注册 observer hooks：`pre_tool_call`、`post_tool_call`、`post_llm_call`、`api_request_error`、`subagent_start`、`subagent_stop`。
  - 目标：hook callback 只返回 `None`，不注册 middleware，不返回 block/action 指令。

- [x] 实现 fail-open safe-field forwarding。
  - 修改：`templates/hermes-plugin/__init__.py`。
  - 目标：使用标准库 `urllib.request` 直接 POST 到 `HERMESHIP_DAEMON_URL` 的 `/event`。
  - 目标：默认 `HERMESHIP_DAEMON_URL=http://127.0.0.1:25295`。
  - 目标：默认 `HERMESHIP_OBSERVER_TIMEOUT_SECS=2`。
  - 目标：`HERMESHIP_OBSERVER_DISABLED` truthy 时跳过发送。
  - 目标：HTTP、序列化、字段访问和类型错误全部 fail-open，不向 Hermes 抛异常。
  - 目标：只转发 safe fields、长度、计数、状态、错误摘要，不转发 raw args、command、tool result body、request/response body、message history、full child goal 或 summary。

- [x] 增加 Python compile/smoke 测试。
  - 修改：`src/release_preflight.rs` 或新增本地测试辅助。
  - 目标：`python3 -m py_compile templates/hermes-plugin/__init__.py` 可通过。
  - 目标：本地 smoke 通过 fake HTTP client 或 monkeypatch 触发 hooks，不依赖真实 Hermes、Discord、network 或凭据。
  - 目标：smoke 断言 POST `/event` payload 为 `hermes.observer.*`，包含 `provider=hermes`、`source=plugin`、`observer_schema_version=1`，且不包含 forbidden raw fields。
  - 结果：`cargo test observer_plugin` 覆盖 compile、fake ctx、fake HTTP、disabled env、fail-open 和 forbidden raw field 断言。

- [x] 更新状态文档与开发清单。
  - 更新：`docs/development-status.md`。
  - 更新：`tasks/development-checklist.md`。
  - 更新：本文件 Review。
  - 已更新：`README.md`、`docs/operations.md`、`ARCHITECTURE.md`、`docs/hermes-event-contract.md`、`docs/observer-plugin.md`。

- [x] 运行验证。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：plugin smoke test。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。
  - 已验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 已验证：`cargo test observer_plugin`（3 passed）。
  - 已验证：`cargo test release_preflight`（15 passed）。
  - 已验证：`cargo run -- release preflight 0.1.0`（9 checks ok，新增 `observer plugin template` ok；`live verification` ok 仍只证明文档字段存在）。
  - 已验证：`cargo fmt --all -- --check`。
  - 已验证：`cargo clippy --all-targets -- -D warnings`。
  - 已验证：`cargo test`（197 lib tests + 15 bin tests passed）。

- [x] 阶段提交。
  - 提交前检查：`git diff --check`、`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - commit 信息：`feat: 增加可选 Hermes observer plugin scaffold`。
  - 提交后检查：`git status --short --branch`、`git log -5 --oneline`。

## Review

- 已新增可选 Hermes observer plugin scaffold：`templates/hermes-plugin/plugin.yaml` 与 `templates/hermes-plugin/__init__.py`。
- Plugin 通过 `register(ctx)` 注册 observer hooks，callback 返回 `None`，使用 `urllib.request` POST `HERMESHIP_DAEMON_URL` 的 `/event`，支持 `HERMESHIP_OBSERVER_TIMEOUT_SECS` 和 `HERMESHIP_OBSERVER_DISABLED`。
- Plugin 只转发 safe id、状态、计数、长度、safe token usage 和 bounded error summary；不转发 raw prompt、conversation history、request/response body、shell command、tool output、tool result JSON、child goal 或 child summary。
- 已按代码审查修复阻塞隐私问题：不再直接复制 `error_message`，只通过 `error_summary` 填充 bounded error summary；smoke 测试新增 `error_message` sentinel，确认原始错误文本不进入 payload。
- 已按复审 warning 加固：`pattern_keys` 增加数量和单项长度边界，observer timeout 增加 5 秒上限，`.gitignore` 忽略 Python bytecode 验证副产物。
- 已扩展 release preflight，新增 `observer plugin template` 检查；实际 `cargo run -- release preflight 0.1.0` 输出该检查为 ok。
- 已更新 README、ARCHITECTURE、operations、Hermes event contract、observer contract、development status 和 development checklist。
- 本轮未执行真实 Discord/Hermes live check，未新增 `docs/live-verification.md` 真实 pass 结果，未实现 Slack sink，未新增 observer install/enable CLI，未新增 typed Rust observer body。
