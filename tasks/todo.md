# Task: Milestone 8.2 - GitHub Source 本地 deterministic parity

更新时间：2026-06-17 Milestone 8.1 已完成并提交，Milestone 8.2 待执行

本文件是当前开发工作台。下次启动应从 Milestone 8.2 GitHub Source 继续。Hermeship 仍然是 Hermes-native daemon-first event router，不调用 clawhip runtime，不依赖运行中的 clawhip daemon。

本次边界：下一阶段只实现 GitHub Source 的本地 deterministic parity 路径；默认不访问真实 GitHub API，不依赖外网，不执行 live verification，不实现 Slack sink，不启动 Hermes plugin/observer。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 当前工作树：最近一次交接检查时 `git status --short --branch` 只显示分支行，工作树干净。
- 文档交接提交：本文件当前更新提交；下次启动以 `git log -3 --oneline` 的最新 docs 提交为准。上一文档交接提交为 `a6bd734 docs: 更新 Hermeship Milestone 8.1 交接状态`。
- 最新功能阶段提交：`1536b6a feat: 增加 Git Source 本地事件路径`。
- 最近提交基线：`a6bd734 docs: 更新 Hermeship Milestone 8.1 交接状态`、`1536b6a feat: 增加 Git Source 本地事件路径`、`475f2a3 docs: 更新 Hermeship Milestone 8 开发入口`。
- Milestone 0 到 Milestone 8.1 已完成并提交。
- Milestone 8.2 到 Milestone 10 未完成。
- 下一入口：Milestone 8.2 GitHub Source。

## 已完成能力

- 已实现 Rust CLI/config/event/privacy/daemon/router/renderer/dispatcher/sinks/hooks/lifecycle/preflight 主路径。
- 已实现 daemon `/health`、`/event`、`/api/hermes/hook`、bounded queue、privacy sanitizer、DaemonClient health/event/hook POST。
- 已实现 Router、DefaultRenderer、Dispatcher、Sink trait、FakeSink、Discord sink、sink 失败语义、本地 daemon -> fake sink smoke。
- 已实现 Hermes hook bridge 模板、install-hooks/uninstall-hooks、安全卸载 marker、handler fail-open smoke。
- 已实现 `hermeship install`、`setup`、`uninstall` 和 `release preflight <version>` 的本地 deterministic 路径。
- 已新增 `deploy/hermeship.service` 与 `docs/operations.md`；当前不真实执行 `systemctl` 或 `launchctl`。
- 已实现 Git Source 本地 deterministic parity：`hermeship git commit`、`hermeship git branch-changed`、`git.commit` / `git.branch-changed` typed event、route metadata 和默认安全渲染。

## 未完成范围

- Milestone 8.2：GitHub Source。
- Milestone 8.3：tmux Source。
- Milestone 8.4：cron 与 memory scaffold。
- Milestone 9：README/architecture/event contract/live verification runbook 与首次 live check。
- Milestone 10：Hermes plugin / observer 研究与可选 MVP。
- 真实 live verification 尚未执行。
- Slack sink 尚未实现。
- Hermes plugin/observer 尚未启动。
- 真实 GitHub API source、真实 tmux watch、真实 systemd/launchd 安装自动化尚未实现。

## 当前执行计划

- [ ] 复习启动文档。
  - `tasks/lessons.md`
  - `docs/development-status.md`
  - `docs/plans/2026-06-15-hermeship-development-plan.md`
  - `tasks/development-checklist.md`
  - `tasks/todo.md`

- [ ] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 完成标准：确认仍在 `codex/milestone-1-cli`，并识别是否存在未提交变更。

- [ ] 确认 Milestone 8.2 计划。
  - 文件：`tasks/development-checklist.md`
  - 入口：`## Milestone 8：clawhip 功能 Parity 扩展`
  - 下一项：任务 8.2 GitHub Source。

- [ ] 阅读 Milestone 8.2 相关代码和 fixture 规则。
  - `src/cli.rs`
  - `src/main.rs`
  - `src/config.rs`
  - `src/events.rs`
  - `src/event/mod.rs`
  - `src/event/body.rs`
  - `src/event/compat.rs`
  - `src/source/git.rs`
  - `src/router.rs`
  - `src/render/mod.rs`
  - `src/render/default.rs`
  - `src/dispatch.rs`
  - `src/lifecycle.rs`
  - `src/release_preflight.rs`
  - `tests/fixtures/README.md`
  - 方案文档中 CLI、source/parity、测试矩阵和发布章节。

- [ ] 写失败测试：GitHub source 事件构造。
  - 新建：`src/source/github.rs`
  - 覆盖：issue、pull request、CI/check、release 事件的本地 deterministic `IncomingEvent` 构造。
  - 隐私边界：不包含完整 issue/PR body、token、cookie、secret 或外部 API 响应正文。

- [ ] 写失败测试：typed conversion 与 route metadata。
  - 修改：`src/event/body.rs`
  - 修改：`src/event/mod.rs`
  - 修改：`src/event/compat.rs`
  - 覆盖：GitHub 事件进入 typed `EventEnvelope`，metadata 可用于 repo、owner、branch、number、status 等 route filter。

- [ ] 写失败测试：CLI 解析与 daemon submit。
  - 修改：`src/cli.rs`
  - 修改：`src/main.rs`
  - 修改：`tests/fixtures/cli/public_commands.txt`
  - 覆盖：`hermeship github ...` 子命令能 parse，并通过 `DaemonClient::post_event()` 投递 `/event`。

- [ ] 写失败测试：默认 renderer。
  - 修改：`src/render/default.rs`
  - 覆盖：compact/inline/alert/raw 输出安全 GitHub 摘要，不泄漏完整 body 或 secret。

- [ ] 运行 Red 验证。
  - 命令：`cargo test github`
  - 预期：实现前失败于缺少 `source::github` API、CLI 子命令和 GitHub typed event variants。

- [ ] 实现最小本地 deterministic GitHub Source。
  - 只构造 Hermeship 自己的 `IncomingEvent`/`EventEnvelope`。
  - 不调用 clawhip runtime。
  - 不访问真实 GitHub API。
  - 不依赖外网、真实 GitHub token 或 webhook secret。

- [ ] 运行 Milestone 8.2 验证。
  - `cargo test github`
  - `cargo test release_preflight`
  - `cargo run -- release preflight 0.1.0`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [ ] 更新开发状态并提交。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 更新：`docs/development-status.md`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- Milestone 8.1 Git Source 本地 deterministic parity 已完成并提交：`1536b6a feat: 增加 Git Source 本地事件路径`。
- 已完成 `src/source/git.rs`，提供 `git.commit` 和 `git.branch-changed` 的 `IncomingEvent` 构造；本阶段不执行真实 `git`、不轮询 repo、不访问远端。
- 已新增 typed Git body，并让 `git.commit` / `git.branch-changed` 进入现有 `IncomingEvent -> EventEnvelope -> Router -> Renderer -> Sink` 管线。
- 已接入 CLI：`hermeship git commit` 与 `hermeship git branch-changed`，并更新公开命令 fixture 与 release preflight 检查。
- 默认 renderer 输出 Git 安全摘要；raw JSON 不展开完整 diff、commit body、repo path、worktree path 或 author email。
- 已验证：`cargo test git`、`cargo test release_preflight`、`cargo run -- release preflight 0.1.0`、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`。
- 当前交接已将下一入口切到 Milestone 8.2 GitHub Source；继续默认使用本地 deterministic fixture，不依赖真实 GitHub 或外网。
- 未进入范围：真实 GitHub API source、真实 git polling source、tmux source、cron、memory、live verification、Slack sink、Hermes plugin/observer。
