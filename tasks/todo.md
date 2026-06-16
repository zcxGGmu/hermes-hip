# Task: Milestone 4.2 - Renderer

更新时间：2026-06-16 Milestone 4.2 已完成

本阶段目标：在已完成 typed event、privacy sanitizer、daemon ingress、Hermes hook ingress、Router 与 `hermeship explain` 的基础上，实现 Hermeship 的第一版 renderer。Renderer 负责把 `EventEnvelope` 和 `ResolvedDelivery` 转为可投递文本，支持 `compact`、`inline`、`alert`、`raw` 四种格式，并支持 route/template 中的 `{session_id}`、`{platform}`、`{project}`、`{event}` 等安全 token。

本阶段边界：只实现 Renderer；不实现 dispatcher、sink、fake sink、Discord HTTP payload、hook bridge install、install/uninstall lifecycle 或 release preflight。默认测试仍只使用本地 deterministic fixture，不依赖真实 Hermes gateway、真实 Discord 或外网状态。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 最新功能阶段提交：本阶段提交 `feat: 增加 Hermes 默认渲染器`（具体 hash 见 `git log -1 --oneline`）。
- 启动时应先确认工作树状态：`git status --short --branch`。
- 已完成 Milestone 3.1：daemon `/health`、typed `HealthResponse`、`QueueHealth`、daemon listener、`hermeship start`、`hermeship status`。
- 已完成 Milestone 3.2：daemon 通用 `POST /event`、`EventAcceptedResponse`、bounded `tokio::mpsc` queue、入队前 privacy sanitizer、`DaemonClient::post_event()`、`hermeship emit`、`hermeship send`。
- 已完成 Milestone 3.3：daemon `POST /api/hermes/hook`、`HermesHookEnvelope` normalization、`DaemonClient::post_hermes_hook()`、`hermeship hermes hook --payload` inline/stdin。
- 已完成 Milestone 4.1：`src/router.rs`、`Router`、`ResolvedDelivery`、`SinkTarget`、`DeliveryExplanation`、event glob、metadata filter、0..N delivery、`hermeship explain` 本地 route explain、Discord webhook 诊断脱敏。
- 当前 daemon 队列仍只入队不消费，达到容量后 `/event` 和 `/api/hermes/hook` 会返回 503；dispatcher/consumer 在 Milestone 4.3 实现。
- 当前 install、release、Hermes hook bridge install 仍保持后续 milestone placeholder。

## 已完成

- [x] Milestone 0：契约与仓库基线。
- [x] Milestone 1.1：Cargo 项目与 CLI 入口。
- [x] Milestone 1.2：配置模型。
- [x] Milestone 1.3：质量门禁与仓库基础。
- [x] Milestone 2.1：IncomingEvent 与格式。
- [x] Milestone 2.2：Typed EventEnvelope。
- [x] Milestone 2.3：隐私与 payload 清洗。
- [x] Milestone 3.1：Daemon health 与 client。
- [x] Milestone 3.2：Event ingress 与队列。
- [x] Milestone 3.3：Hermes hook ingress。
- [x] Milestone 4.1：Router。
- [x] Milestone 4.2：Renderer。

## 后续未完成

- [ ] Milestone 4.3：Dispatcher 与 fake sink。
- [ ] Milestone 5：Discord Sink 与基础 Live Path。
- [ ] Milestone 6：Hermes Hook Bridge 安装。
- [ ] Milestone 7：安装、生命周期与运维 CLI。
- [ ] Milestone 8：clawhip 功能 Parity 扩展。
- [ ] Milestone 9：文档与 Live Verification。
- [ ] Milestone 10：Hermes Plugin / Observer 研究。

## 执行计划

- [x] 复习项目规则与状态入口。
  - 阅读：`tasks/lessons.md`
  - 阅读：`docs/development-status.md`
  - 阅读：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 阅读：`tasks/development-checklist.md`
  - 阅读：`tasks/todo.md`

- [x] 确认当前分支、最新提交和未提交变更。
  - 命令：`git status --short --branch`
  - 命令：`git log -3 --oneline`
  - 预期：当前分支为 `codex/milestone-1-cli`；最新功能阶段提交为 `864e7f4 feat: 实现多投递路由`；启动时不要混入无关改动。

- [x] 检查现有代码边界。
  - 查看：`src/cli.rs`
  - 查看：`src/main.rs`
  - 查看：`src/config.rs`
  - 查看：`src/events.rs`
  - 查看：`src/event/mod.rs`
  - 查看：`src/event/body.rs`
  - 查看：`src/event/compat.rs`
  - 查看：`src/privacy.rs`
  - 查看：`src/router.rs`
  - 查看：`tests/fixtures/README.md`
  - 必要时参考：`/Users/zq/Desktop/ai-projs/posp/template/clawhip/src/render/default.rs`
  - 完成标准：确认本阶段只实现 renderer，不进入 dispatcher/sink。

- [x] 先写失败测试。
  - 新建：`src/render/mod.rs`
  - 新建：`src/render/default.rs`
  - 必要时修改：`src/lib.rs`
  - 覆盖：`compact`、`inline`、`alert`、`raw` 格式，Hermes gateway/session/agent/custom 事件，缺字段降级，template token 渲染，敏感正文不泄漏。
  - 命令：`cargo test render`
  - 预期：实现前测试失败于缺少 render 模块、默认 renderer、template token 和格式渲染行为。

- [x] 实现 renderer 类型与默认 renderer。
  - 新建：`src/render/mod.rs`
  - 新建：`src/render/default.rs`
  - 类型：`RenderedMessage` 或等价结果类型、`DefaultRenderer`。
  - 输入：`EventEnvelope`、`ResolvedDelivery`。
  - 输出：可投递文本，不包含真实 token、cookie、secret、完整 prompt、完整对话或 provider request/response body。

- [x] 实现四种格式。
  - `compact`：短摘要，适合普通通知。
  - `inline`：单行可扫描文本，包含 event、session/platform/project 等关键字段。
  - `alert`：突出失败或高优先级事件。
  - `raw`：输出 sanitizer 后的安全 JSON 摘要，不输出完整高风险正文。

- [x] 实现 Hermes 事件渲染。
  - gateway/session/agent/custom。
  - 缺失 `session_id`、`platform`、`project`、`agent_name` 时应降级为可读占位或省略，不 panic。
  - `hermes.agent.failed` 应能展示安全错误摘要。

- [x] 实现 template token 渲染。
  - 支持 `{event}`、`{session_id}`、`{platform}`、`{project}`、`{agent_name}`、`{source}`、`{provider}`、`{channel}` 等当前 metadata 可提供字段。
  - 未知 token 保持原样或明确降级，不能 panic。
  - token value 来自 typed metadata 和 canonical event，不从未清洗原始正文取值。

- [x] 编写 renderer 测试。
  - 覆盖：所有格式。
  - 覆盖：缺字段降级。
  - 覆盖：raw JSON 不泄漏敏感字段。
  - 覆盖：template token。
  - 覆盖：route-level `format`、`template`、`mention` 与 renderer 输出的组合边界。
  - 要求：使用本地 config 和 fixture，不依赖 daemon、Discord 或外网。

- [x] 运行任务 4.2 验证命令。
  - `cargo test render`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

- [x] 更新开发状态文档。
  - 更新：`tasks/development-checklist.md`
  - 更新：`tasks/todo.md`
  - 必要时更新：`docs/development-status.md`
  - 完成标准：记录实现、验证、边界和剩余风险。

- [x] 提交任务 4.2。
  - commit：`feat: 增加 Hermes 默认渲染器`
  - commit 信息使用中文，说明变更、验证和影响。

## Review

- 已新增 `src/render/mod.rs`、`src/render/default.rs`，并在 `src/lib.rs` 导出 `hermeship::render`。
- 已实现 `Renderer` trait、`DefaultRenderer`、`RenderedMessage`，输入为 `EventEnvelope` 和 `ResolvedDelivery`，输出 deterministic 可投递文本。
- 已支持 `compact`、`inline`、`alert`、`raw` 四种格式，并覆盖 Hermes gateway/session/agent/custom 事件。
- 已实现 template token：`{event}`、`{canonical_kind}`、`{source}`、`{provider}`、`{platform}`、`{session_id}`、`{agent_name}`、`{project}`、`{channel}`；未批准 token 保持原样。
- 已按代码审查修复 raw 隐私边界：`MessageFormat::Raw` 永远输出 JSON，忽略 template/mention，不直接序列化 typed 自由文本，只保留长度/存在性摘要并清洗 nested payload。
- 已覆盖测试：所有格式、缺字段降级、raw JSON、template token、route-level `format`/`template`/`mention`、raw+template、direct typed free-text raw 泄漏回归和未批准 token。
- 已运行验证：`cargo test render`（10 passed）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（74 lib tests + 6 bin tests passed）。
- 本阶段未实现 dispatcher、sink、fake sink、Discord HTTP payload、hook bridge install、install/uninstall lifecycle 或 release preflight；Milestone 4.3 继续实现 dispatcher 与 fake sink。
- 上一阶段 Milestone 4.1 已完成并提交：`864e7f4 feat: 实现多投递路由`。
