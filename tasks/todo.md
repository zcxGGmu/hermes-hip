# Task: Milestone 7 - 安装、生命周期与运维 CLI

更新时间：2026-06-16 Milestone 7 已完成，下一入口为 Milestone 8

本阶段目标：补齐 Hermeship 的本地可运维表面：`install`、`setup`、`uninstall`、service 模板和 `release preflight`。实现必须是 Hermes-native、daemon-first，不调用 clawhip runtime，不依赖运行中的 clawhip daemon。

本阶段边界：只做本地 deterministic lifecycle 与 preflight 路径；默认测试只使用临时 HOME、fake Hermes home、本地 fixture 和仓库文件。不实现真实 live verification、Slack sink、Hermes plugin/observer，也不真实执行 systemd/launchd 安装或启动。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动确认：`git status --short --branch` 显示工作树干净。
- 最新功能阶段提交：`4fe5c14 feat: 增加安装生命周期与发布预检`。
- 最近提交包含：`4fe5c14 feat: 增加安装生命周期与发布预检`、`64e8641 docs: 更新 Hermeship 最新开发状态`、`f6f98a3 feat: 支持 Hermes hook bridge 安装`。
- Milestone 0 到 Milestone 7 已完成并提交。
- 已实现 daemon `/health`、`/event`、`/api/hermes/hook`、bounded queue、privacy sanitizer、DaemonClient health/event/hook POST。
- 已实现 Router、DefaultRenderer、Dispatcher、Sink trait、FakeSink、Discord sink、sink 失败语义、本地 daemon -> fake sink smoke。
- 已实现 Hermes hook bridge 模板、install-hooks/uninstall-hooks、安全卸载 marker、handler fail-open smoke。

## 执行计划

- [x] 复习项目 rules、lessons、状态入口和 Milestone 7 范围。
  - 已读：`tasks/lessons.md`
  - 已读：`docs/development-status.md`
  - 已读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 已读：`tasks/development-checklist.md`
  - 已读：`tasks/todo.md`

- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 记录：当前分支为 `codex/milestone-1-cli`，工作树干净，最新提交为 `64e8641 docs: 更新 Hermeship 最新开发状态`。

- [x] 阅读 Milestone 7 相关源码和方案章节。
  - 已读：`src/cli.rs`
  - 已读：`src/main.rs`
  - 已读：`src/hooks.rs`
  - 已读：`src/config.rs`
  - 已读：`src/daemon.rs`
  - 已读：`src/client.rs`
  - 已读：`tests/fixtures/README.md`
  - 已读：方案文档安装与回滚章节、测试矩阵、版本与发布章节。
  - 参考：`template/clawhip/src/lifecycle.rs`、`template/clawhip/src/release_preflight.rs`、`template/clawhip/src/cli.rs`、`template/clawhip/src/main.rs`。

- [x] 任务 7.1：先写 install/setup 失败测试。
  - 新增或修改：`src/lifecycle.rs`、`src/cli.rs`、`src/main.rs`、`src/lib.rs`。
  - 覆盖：`hermeship install --home <tmp> --config <tmp>/config.toml --dry-run` 只报告，不写磁盘。
  - 覆盖：`hermeship install --home <tmp> --config <tmp>/config.toml` 创建 `.hermeship/`、`state/`、`hooks/`、`logs/`，scaffold `config.toml`，不覆盖已有配置。
  - 覆盖：`hermeship setup --discord-token-stdin --default-channel <id> --daemon-url <url>` 更新配置但报告不打印 secret。
  - 命令：`cargo test lifecycle`
  - 预期：实现前失败于缺少 lifecycle 模块、setup 命令和 install 参数。

- [x] 任务 7.1：实现 install/setup 最小路径。
  - `install` 只创建本地 Hermeship 目录和默认配置，不安装真实 service，不启动 daemon。
  - `setup` 只更新 Hermeship TOML 配置中的 Discord token、default channel 和 daemon base URL；输出 redacted summary。
  - 完成标准：`cargo test lifecycle` 通过。

- [x] 任务 7.2：先写 service/uninstall 失败测试。
  - 新增：`deploy/hermeship.service`。
  - 覆盖：service 模板包含 `ExecStart`、`HERMESHIP_CONFIG`、`hermeship start`。
  - 覆盖：`uninstall --home <tmp> --config <tmp>/config.toml --remove-config --remove-hooks --dry-run` 只报告。
  - 覆盖：默认 uninstall 不删除 config，`--remove-config` 才删除配置，`--remove-hooks` 调用 Hermes hook 安全卸载并不误删用户 hook。
  - 命令：`cargo test lifecycle`

- [x] 任务 7.2：实现 service 模板与 uninstall。
  - CLI：`hermeship uninstall --home <path> --config <path> --remove-config --remove-hooks --dry-run`。
  - 本阶段不运行 `systemctl` 或 `launchctl`，只提供模板和报告。
  - 完成标准：`cargo test lifecycle` 通过。

- [x] 任务 7.3：先写 release preflight 失败测试。
  - 新增：`src/release_preflight.rs`。
  - 覆盖：版本规范化、`Cargo.toml`/`Cargo.lock` 一致性、公开 CLI fixture、hook 模板包含、fixture README policy、live verification 文档缺失时给出 pending 诊断但不阻塞默认本地 preflight。
  - 命令：`cargo test release_preflight`
  - 预期：实现前失败于缺少模块和 CLI 接入。

- [x] 任务 7.3：实现 release preflight 与 CLI。
  - CLI：`hermeship release preflight <version>`。
  - 检查项目一致性，输出每项 `ok`/`pending`/`fail`，只对本地发布一致性 hard fail。
  - 完成标准：`cargo test release_preflight` 和 `cargo run -- release preflight 0.1.0` 有确定性输出。

- [x] 运行 Milestone 7 验证命令。
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
  - 窄测试：`cargo test lifecycle`
  - 窄测试：`cargo test release_preflight`
  - CLI smoke：`cargo run -- --config /tmp/hermeship-lifecycle-home/config.toml install --home /tmp/hermeship-lifecycle-home --dry-run`
  - CLI smoke：`cargo run -- release preflight 0.1.0`

- [x] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险，并把下一入口切到 Milestone 8。

- [x] 提交 Milestone 7。
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- Milestone 7 已实现本地 deterministic lifecycle：新增 `src/lifecycle.rs`，支持 `install`、`setup`、`uninstall` 纯文件系统路径和结构化报告。
- `hermeship install` 支持 `--home`、`--force`、`--dry-run`，创建 Hermeship home、`state/`、`hooks/`、`logs/` 和默认 `config.toml`；不安装真实 service，不启动 daemon。
- `hermeship setup` 支持 `--discord-token-stdin`、`--discord-token-env`、`--default-channel`、`--daemon-url`、`--dry-run`；报告输出将 Discord token 显示为 `<redacted>`。
- `hermeship uninstall` 默认不删除用户配置或状态；只有显式 `--remove-config`、`--remove-state`、`--remove-hooks` 才删除对应路径；destructive removal 必须验证 Hermeship home marker，Hermes gateway hook 删除复用 Milestone 6 marker-based safe uninstall。
- 新增 `deploy/hermeship.service` systemd user service 模板和 `docs/operations.md`，记录 launchd 手动示例；本阶段没有执行 `systemctl` 或 `launchctl`。
- 新增 `src/release_preflight.rs`，检查 Cargo 版本一致性、公开 CLI fixture、文档命令、hook 模板、fixture policy、service 模板和 live verification；缺失 live verification 记录为 pending，不阻塞默认本地 preflight。
- 已完成 Red 验证：实现前 `cargo test lifecycle` 失败于缺少 lifecycle API 和 service 模板，`cargo test release_preflight` 失败于缺少 preflight API。
- 已完成本地 CLI smoke：`install --dry-run`、临时目录 `install`、`setup` 脱敏输出、`uninstall --dry-run`、`release preflight 0.1.0`。
- 已根据代码审查修复安全边界：`setup` 不再接受明文 token argv，改用 stdin/env；`config show` 默认脱敏；写配置时使用私有权限；`install` 写入 home marker；destructive `uninstall` 必须验证 marker；`--remove-hooks` 默认使用 Hermes home；release preflight 纳入 `docs/operations.md`。
- 已完成验证：`cargo test lifecycle`（10 passed）、`cargo test release_preflight`（6 passed）、`cargo test cli`（17 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（139 lib tests + 8 bin tests passed）。
- 本阶段没有实现真实 live verification、Slack sink、Hermes plugin/observer、真实 systemd/launchd 安装或外部网络发布自动化。
- 下一入口：从 `tasks/development-checklist.md` 的 Milestone 8 clawhip 功能 parity 扩展继续。
