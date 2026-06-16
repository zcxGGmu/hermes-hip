# Task: Milestone 8 - clawhip 功能 Parity 扩展

更新时间：2026-06-16 Milestone 7 已完成并提交，Milestone 8 待启动

本文件是下次开发启动工作台。下次会话开始后先复习项目交接文档，再从 Milestone 8 的本地 deterministic parity 路径继续。Hermeship 仍然是 Hermes-native daemon-first event router，不调用 clawhip runtime，不依赖运行中的 clawhip daemon。

本次边界：只更新开发状态交接文档，不进入 Milestone 8 实现，不执行真实 live verification、Slack sink 或 Hermes plugin/observer。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 更新前工作树：`git status --short --branch` 只显示分支行，工作树干净。
- 最新功能阶段提交：`162efcd feat: 增加安装生命周期与发布预检`。
- 最近提交包含：`162efcd feat: 增加安装生命周期与发布预检`、`64e8641 docs: 更新 Hermeship 最新开发状态`、`f6f98a3 feat: 支持 Hermes hook bridge 安装`。
- Milestone 0 到 Milestone 7 已完成并提交。
- Milestone 8 到 Milestone 10 未完成。

## 已完成能力

- 已实现 Rust CLI/config/event/privacy/daemon/router/renderer/dispatcher/sinks/lifecycle/preflight 主路径。
- 已实现 daemon `/health`、`/event`、`/api/hermes/hook`、bounded queue、privacy sanitizer、DaemonClient health/event/hook POST。
- 已实现 Router、DefaultRenderer、Dispatcher、Sink trait、FakeSink、Discord sink、sink 失败语义、本地 daemon -> fake sink smoke。
- 已实现 Hermes hook bridge 模板、install-hooks/uninstall-hooks、安全卸载 marker、handler fail-open smoke。
- 已实现 `hermeship install`、`setup`、`uninstall` 和 `release preflight <version>` 的本地 deterministic 路径。
- 已新增 `deploy/hermeship.service` 与 `docs/operations.md`；本阶段不真实执行 `systemctl` 或 `launchctl`。

## 未完成范围

- Milestone 8：clawhip 功能 parity 扩展，包括 git、GitHub、tmux、cron 和 memory scaffold。
- Milestone 9：README/architecture/event contract/live verification runbook 与首次 live check。
- Milestone 10：Hermes plugin / observer 研究与可选 MVP。
- 真实 live verification 尚未执行。
- Slack sink 尚未实现。
- Hermes plugin/observer 尚未启动。
- 真实 systemd/launchd 安装自动化尚未实现。

## 下次执行计划

- [ ] 复习启动文档。
  - `tasks/lessons.md`
  - `docs/development-status.md`
  - `docs/plans/2026-06-15-hermeship-development-plan.md`
  - `tasks/development-checklist.md`
  - `tasks/todo.md`

- [ ] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 预期：分支为 `codex/milestone-1-cli`；最新功能阶段提交仍可追溯到 `162efcd feat: 增加安装生命周期与发布预检`；未提交变更必须先判断是否为预期文档/代码变更。

- [ ] 确认 Milestone 8 计划。
  - 文件：`tasks/development-checklist.md`
  - 入口：`## Milestone 8：clawhip 功能 Parity 扩展`
  - 第一项：任务 8.1 Git Source。

- [ ] 阅读 Milestone 8 相关代码和 fixture 规则。
  - `src/cli.rs`
  - `src/main.rs`
  - `src/config.rs`
  - `src/events.rs`
  - `src/event/mod.rs`
  - `src/event/body.rs`
  - `src/event/compat.rs`
  - `src/router.rs`
  - `src/render/mod.rs`
  - `src/render/default.rs`
  - `src/dispatch.rs`
  - `src/lifecycle.rs`
  - `src/release_preflight.rs`
  - `tests/fixtures/README.md`
  - 方案文档中 CLI、source/parity、测试矩阵和发布章节。

- [ ] 任务 8.1：Git Source，先写失败测试。
  - 计划新建：`src/source/git.rs`
  - 计划修改：`src/cli.rs`、`src/main.rs`、`src/lib.rs`
  - 计划 fixture：本地 deterministic git payload/metadata，不使用真实远程仓库或外网。
  - 覆盖：repo path、branch、commit sha、commit summary、author metadata、route metadata。
  - CLI 预期：`hermeship git commit`、`hermeship git branch-changed`。
  - 窄验证：`cargo test git`

- [ ] 任务 8.1：实现最小本地 deterministic Git Source。
  - 不调用 clawhip runtime。
  - 不依赖运行中的 clawhip daemon。
  - 默认只构造并投递 Hermeship 自己的 `IncomingEvent`/`EventEnvelope`。
  - 保持隐私默认值，不发送完整 diff、完整 commit body 或 secret。

- [ ] 运行 Milestone 8 验证。
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
  - 按子任务补充更窄测试，例如 `cargo test git`。

- [ ] 更新开发状态并提交。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 本次只做文档交接更新，未修改功能代码。
- 已将状态入口更新到 Milestone 7 已完成并提交、Milestone 8 待执行。
- 已将过期 Milestone 7 提交引用修正为实际 HEAD `162efcd feat: 增加安装生命周期与发布预检`。
- 已明确下一入口是 Milestone 8 的 clawhip 功能 parity 扩展，首选任务为 8.1 Git Source。
- 已保留边界：默认不进入真实 live verification、Slack sink 或 Hermes plugin/observer。
- 本次文档验证：`git diff --check`、过期 Milestone 7 提交号搜索、`git status --short --branch`。
