# Task: Milestone 1.3 - 质量门禁与仓库基础

启动时间：2026-06-15

本阶段目标：补齐仓库基础质量门禁，不进入 daemon、event、privacy、router、sink、install 或 release preflight 实现。

- [x] 复习 `tasks/lessons.md`，确认阶段完成后必须验证并提交。
- [x] 复习阶段上下文。
  - 阅读：`docs/development-status.md`
  - 阅读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 阅读：`tasks/development-checklist.md`
  - 阅读：`tasks/todo.md`
- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 结果：当前分支为 `codex/milestone-1-cli`；最新提交为 `267efba docs: 更新 Hermeship 最新开发状态`；最新功能阶段提交为 `50723af feat: 实现 hermeship 配置模型与 config CLI`；启动时工作树干净。
- [x] 明确本阶段边界。
  - 只处理仓库基础、质量门禁文档和 fixture 目录。
  - 不实现 daemon、events、privacy、router、sink、install 或 release preflight。
  - 不修改方案文档的架构边界，执行进度只写入 `tasks/development-checklist.md` 和本文。
- [x] 检查现有仓库基础。
  - 查看：`.gitignore`
  - 查看：`README.md`
  - 查看：`tests/fixtures/`
  - 完成标准：确认当前 `.gitignore` 只有 `/target/`，fixture 目前只有 `tests/fixtures/cli/public_commands.txt`。
- [x] 更新 `.gitignore`。
  - 保留：`/target/`
  - 增加：常见本地临时文件、日志、测试输出目录。
  - 完成标准：不会忽略源码、文档、fixture 或 Cargo lockfile。
- [x] 增加 rustfmt/clippy 约束说明。
  - 文件：`README.md` 或 `docs/development.md`
  - 内容：本地验证命令、默认测试不得依赖外部凭据或网络、阶段提交前必须运行的基础门禁。
- [x] 增加测试 fixture 目录。
  - 新建：`tests/fixtures/hermes/`
  - 新建：`tests/fixtures/privacy/`
  - 新建：`tests/fixtures/routes/`
  - 新建：`tests/fixtures/discord/`
  - 保留：`tests/fixtures/cli/`
  - 完成标准：目录可被 git 跟踪，且不包含真实 token、cookie、prompt、完整对话或 provider request/response body。
- [x] 修复 clippy 门禁暴露的既有 lint。
  - 首次 `cargo clippy --all-targets -- -D warnings` 失败于 `clippy::derivable_impls` 和 `clippy::useless_conversion`。
  - 修复：`AppConfig`、`MessageFormat` 使用 derive/default；CLI fixture 测试去掉多余 `.into_iter()`。
- [x] 运行任务 1.3 验证命令。
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- [x] 更新 `tasks/development-checklist.md`。
  - 勾选任务 1.3 已完成项。
  - 在运行状态日志顶部记录本阶段实现、验证和提交状态。
- [x] 更新 `tasks/todo.md` Review。
  - 记录实现、验证、边界和剩余风险。
- [x] 提交任务 1.3。
  - commit：`chore: 增加 Rust 质量门禁与仓库基础`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 实现：扩展 `.gitignore`，保留 `/target/` 并新增本地编辑器临时文件、日志、临时目录、测试输出和覆盖率输出规则；未忽略源码、文档、fixture 或 `Cargo.lock`。
- 实现：在 `README.md` 新增 Development Quality Gates，明确阶段提交前运行 `cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`，并重申默认测试不得依赖外部凭据或真实 Hermes/Discord/GitHub/tmux/外网状态。
- 实现：新增 `tests/fixtures/hermes/`、`tests/fixtures/privacy/`、`tests/fixtures/routes/`、`tests/fixtures/discord/`，保留 `tests/fixtures/cli/`，并新增 `tests/fixtures/README.md` 记录 fixture 脱敏规则。
- 实现：修复 clippy 首次门禁发现的既有 lint，未改变配置默认值或 CLI 解析语义。
- 验证：`cargo fmt --all -- --check` 通过。
- 验证：`cargo clippy --all-targets -- -D warnings` 通过。
- 验证：`cargo test` 通过，14 个测试全部通过。
- 边界：未实现 daemon、events、privacy、router、sink、install 或 release preflight；未修改方案文档。
- 剩余风险：本阶段只建立 fixture 目录和质量门禁说明，尚未添加 Milestone 2+ 的具体 Hermes/privacy/routes/discord fixture payload。
