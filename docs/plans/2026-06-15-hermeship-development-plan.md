# Hermeship 开发方案

日期：2026-06-15

## 1. 文档定位

本文是 `hermeship` 的中文开发方案，描述系统目标、边界、架构、事件契约、安全策略、验证策略和发布策略。

具体迭代任务、进度勾选、每阶段验证命令和提交边界不在本文重复维护，统一放在：

- `tasks/development-checklist.md`

这样做的原因是：方案文档保持稳定，用来回答“为什么这样设计”和“边界在哪里”；任务清单保持可变，用来跟踪“当前做到哪一步”和“下一步做什么”。

## 2. 背景

`clawhip` 是一个 daemon-first 的事件到频道通知路由器。它接收 GitHub、git、tmux、自定义事件和 provider-native hook payload，经过标准化、路由、渲染后发送到 Discord 等下游。

Hermes 是一个跨 CLI、网关、多聊天平台、插件、技能和子代理的个人 AI agent。Hermes 的设计原则是核心窄、能力在边缘扩展；因此 `hermeship` 不应修改 Hermes 核心，也不应把通知逻辑塞回 Hermes 对话上下文。

`hermeship` 的目标是把 Hermes 生命周期事件转发给 `clawhip`，让通知进入独立事件管道，避免污染 Hermes gateway session。

## 3. 目标

`hermeship` 第一阶段要做到：

- 作为 Hermes 到 clawhip 的适配层。
- 安装一个 Hermes gateway hook bundle。
- 监听 Hermes 的粗粒度生命周期事件。
- 将事件映射成 clawhip 已有的 `agent.*` 和 `emit` 事件。
- 支持 dry-run，便于本地调试和 CI。
- 在 clawhip 不存在、daemon 不可用或配置错误时 fail-open，不影响 Hermes 正常运行。
- 默认不发送完整对话、完整 prompt、provider 请求/响应、token、cookie 或 secrets。

## 4. 非目标

MVP 不做以下事情：

- 不 fork `clawhip`。
- 不替代 `clawhip` daemon、router、renderer 或 Discord sink。
- 不修改 Hermes 核心代码。
- 不新增 Hermes model tool。
- 不实现多 agent 调度器。
- 不自动启动 Codex/LazyCodex worker。
- 不创建 GitHub PR。
- 不把 Hermes 加进 clawhip 已冻结的 Codex/Claude native hook v1 契约。
- 不默认转发 `command:*` 事件。
- 不默认实现 HTTP transport；第一版优先使用 clawhip CLI。

## 5. 设计原则

- **失败开放**：Hermeship 失败不能中断 Hermes。
- **边界清晰**：Hermeship 只做适配，clawhip 继续负责路由和投递。
- **依赖最小**：MVP 使用 Python 标准库优先，第三方依赖只进入 dev extra。
- **事件小而稳**：发送摘要和结构化元数据，不发送大段内容。
- **隐私默认安全**：默认丢弃高风险字段，递归脱敏敏感 key。
- **安装可回滚**：hook 安装必须幂等，回滚只需删除 hook 目录。
- **不重复状态机**：不在 Hermeship 里重建 Hermes 或 clawhip 的内部状态。
- **先验证再提交**：每个阶段完成后必须运行验证命令并提交。

## 6. 总体架构

```text
Hermes gateway hooks
  -> hermeship hook handler
  -> hermeship mapper
  -> hermeship clawhip client
  -> clawhip CLI / daemon
  -> clawhip router / renderer / sink
  -> Discord notification channel
```

组件职责：

| 组件 | 职责 |
| --- | --- |
| Hermes | 运行 agent、网关、session、hook 触发 |
| Hermeship hook handler | 接收 Hermes hook context，fail-open 转发 |
| Hermeship mapper | 将 Hermes event/context 标准化为 Hermeship event |
| Hermeship privacy layer | 截断、脱敏、丢弃高风险字段 |
| Hermeship clawhip client | 调用 `clawhip agent` 或 `clawhip emit` |
| clawhip | 路由、渲染、投递通知 |
| Discord | 接收通知，不承载 Hermes 思考上下文 |

## 7. 与 clawhip 的关系

Hermeship 复用 clawhip 现有公共入口：

- `clawhip agent started`
- `clawhip agent finished`
- `clawhip agent blocked`
- `clawhip agent failed`
- `clawhip emit <event>`

Hermeship 不改变 clawhip 以下能力：

- daemon 生命周期
- Discord 配置
- route/filter 配置
- renderer/sink
- Git/GitHub/tmux source
- Codex/Claude native hook v1 contract

如果 MVP 证明需要 clawhip 上游支持，只提交小而通用的上游改动，例如：

- 更明确的 `emit --payload` 文档。
- provider-agnostic event ingress。
- `hermes.*` custom event 的渲染支持。

不要把 Hermes 特定假设写入 clawhip 的 Codex/Claude native hook v1 契约。

## 8. 与 Hermes 的关系

Hermeship 首选接入点是 Hermes gateway hook：

- `gateway:startup`
- `session:start`
- `session:end`
- `session:reset`
- `agent:start`
- `agent:end`

Hermeship 不要求 Hermes 核心 patch，也不新增核心工具。后续如果需要更高保真 telemetry，再研究 Hermes observer plugin：

- `on_session_start`
- `on_session_end`
- `pre_tool_call`
- `post_tool_call`
- `api_request_error`
- `subagent_start`
- `subagent_stop`

Observer plugin 是后续阶段，不是 MVP。

## 9. MVP 事件映射

| Hermes event | Hermeship event | clawhip 入口 | clawhip event |
| --- | --- | --- | --- |
| `gateway:startup` | `gateway.started` | `clawhip emit` | `hermes.gateway.started` |
| `session:start` | `session.started` | `clawhip emit` | `hermes.session.started` |
| `session:end` | `session.finished` | `clawhip emit` | `hermes.session.finished` |
| `session:reset` | `session.reset` | `clawhip emit` | `hermes.session.reset` |
| `agent:start` | `agent.started` | `clawhip agent started` | `agent.started` |
| `agent:end` | `agent.finished` | `clawhip agent finished` | `agent.finished` |
| `agent:end` with error | `agent.failed` | `clawhip agent failed` | `agent.failed` |

`agent:end` 默认映射为完成。只有当 context 中存在明确的 `error`、`exception` 或 `status=failed` 时才映射为失败。

## 10. Hermeship 事件模型

内部事件建议使用不可变 dataclass：

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

payload 默认只允许以下字段：

- `provider`
- `origin`
- `session_id`
- `platform`
- `chat_id`
- `thread_id`
- `user_id`
- `project`
- `summary`
- `error_message`

必须设置：

```text
provider = hermes
origin = hermeship
```

`origin=hermeship` 用于避免递归通知。

## 11. 配置设计

用户配置文件：

```text
~/.hermeship/config.toml
```

示例：

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
dedupe_window_secs = 30
redact_keys = ["token", "api_key", "authorization", "password", "secret", "cookie"]
```

配置优先级：

1. CLI flags
2. 环境变量覆盖
3. repo-local override
4. `~/.hermeship/config.toml`
5. 内置默认值

允许的环境变量：

- `HERMESHIP_CONFIG`
- `HERMESHIP_DRY_RUN`
- `HERMESHIP_CLAWHIP_BIN`
- `HERMESHIP_CLAWHIP_URL`

不要新增用户可见的 `HERMES_*` 非 secret 配置。

## 12. 配置校验

配置加载必须满足：

- `mode` 第一版只允许 `cli`。
- `format` 只允许 clawhip 支持的格式。
- 空字符串的 `channel`、`mention`、`project` 归一化为 `None`。
- `timeout_secs` 必须有上下限。
- `dedupe_window_secs` 必须有上下限。
- 解析失败时禁用转发并输出短诊断，不抛异常到 Hermes。
- 未知 key 在 strict mode 下失败，在 permissive mode 下告警。

## 13. 隐私与安全

默认禁止发送：

- `conversation_history`
- 完整 prompt
- 完整 user message
- 完整 assistant response
- provider request
- provider response
- tool result body
- token
- cookie
- secret
- API key
- password
- authorization header

处理规则：

- 递归脱敏敏感 key。
- message 和 response 只保留截断摘要。
- 默认不转发 `command:*`。
- 不读取或上传文件内容。
- 不复用 Hermes bot token 作为 clawhip 通知 bot token。
- live verification 使用专用测试频道。

## 14. 失败处理

Hermeship 必须 fail-open。

| 失败类型 | 行为 | 记录 |
| --- | --- | --- |
| 配置不存在 | 使用默认值或禁用转发 | config path |
| 配置非法 | 禁用转发 | 解析错误摘要 |
| clawhip binary 缺失 | 跳过发送 | session_id、binary |
| clawhip daemon 不可用 | 跳过发送 | stderr tail |
| clawhip 超时 | 跳过发送 | command、timeout |
| clawhip 非零退出 | 跳过发送 | exit code、stderr tail |
| payload 序列化失败 | 跳过事件 | event kind |
| 重复事件 | dedupe suppress | dedupe key |
| 递归事件 | drop | origin |

任何 handler 异常都不能逃逸到 Hermes。

## 15. 去重与防递归

防递归：

- 如果 context 或 payload 中已有 `origin=hermeship`，直接忽略。

去重：

- 使用内存级短窗口去重。
- dedupe key 建议为：

```text
event_type + session_id + summary_hash
```

去重状态必须有界，不持久化。

## 16. 性能要求

- 本地 mapping 应在 50 ms 内完成。
- clawhip CLI 调用默认 2 秒超时。
- dry-run 不调用 subprocess。
- 不在 Hermes hook handler 内执行不必要网络请求。
- 不阻塞 Hermes shutdown。

## 17. 安装与回滚

安装：

```bash
python -m pip install -e ".[dev]"
hermeship install-hook
hermeship doctor
```

hook 目录：

```text
~/.hermes/hooks/hermeship-clawhip/
  HOOK.yaml
  handler.py
```

回滚：

```bash
rm -rf ~/.hermes/hooks/hermeship-clawhip
python -m pip uninstall hermeship
```

如果 Hermes 进程缓存 hook，需要重启 Hermes。

## 18. 测试策略

测试矩阵：

| 层 | 必测内容 |
| --- | --- |
| config | 默认值、env override、非法 TOML、未知 key |
| privacy | 递归脱敏、截断、非字符串、大 payload |
| mapper | 所有支持事件、缺失字段、错误字段、禁用事件 |
| client | CLI 命令生成、dry-run、timeout、binary 缺失、非零退出 |
| hook handler | fail-open、去重、防递归 |
| installer | 首次安装、不覆盖、force、回滚路径 |
| live clawhip | sample event 到达 daemon |
| live Discord | 测试频道收到消息 |
| Hermes gateway | hook 在隔离 `HERMES_HOME` 加载 |

CI 建议：

```bash
python -m pip install -e ".[dev]"
ruff check .
pytest -q
python -m hermeship --help
python -m hermeship emit-sample --event agent:start --dry-run
```

## 19. Live Verification

前置条件：

- clawhip 已安装。
- clawhip daemon 可运行。
- clawhip 已配置专用 Discord 通知 bot。
- 有测试频道。

命令：

```bash
clawhip status
hermeship doctor
hermeship emit-sample --event agent:start --dry-run
hermeship emit-sample --event agent:start
hermeship emit-sample --event agent:end
```

预期：

- dry-run 输出 JSON。
- live start 事件进入 clawhip。
- live end 事件进入 clawhip。
- Discord 测试频道收到 compact 通知。

## 20. 版本与发布

版本策略：

- `0.1.0`：CLI transport、hook install、dry-run、sample events。
- `0.2.0`：live verification 文档与诊断增强。
- `0.3.0`：可选 observer plugin 研究。
- `1.0.0`：配置 schema、事件映射、回滚和 live verification 稳定。

发布前必须运行：

```bash
ruff check .
pytest -q
python -m hermeship --help
python -m hermeship emit-sample --event agent:start --dry-run
python -m build
```

## 21. 后续演进

MVP 稳定后再考虑：

- HTTP transport。
- Hermes observer plugin。
- 更细粒度 tool/API telemetry。
- 更丰富的 clawhip renderer。
- 与 repo-local `.hermeship/config.toml` 的更完整合并策略。

不建议过早做：

- 通用 AgentPort。
- 多 agent orchestration。
- 自动修复/自动 PR。
- 修改 clawhip native hook v1 contract。

## 22. 开放问题

- live Discord verification 凭据是否可用？
- v0.1.0 是否要求真实 Hermes gateway run，还是允许先用 hook smoke test？
- `repo-local override` 是否第一版就实现？
- `ruff` 是否立即纳入 dev extra？
- 是否需要为 `clawhip emit --payload` 补充上游文档？

## 23. 参考

- `../clawhip/docs/event-contract-v1.md`
- `../clawhip/docs/native-event-contract.md`
- `../clawhip/src/cli.rs`
- `/home/zq/work-space/repo/ai-projs/agents/hermes-agent/gateway/hooks.py`
- `/home/zq/work-space/repo/ai-projs/agents/hermes-agent/docs/observability/README.md`
- `/home/zq/work-space/repo/ai-projs/agents/hermes-agent/docs/middleware/README.md`
- `tasks/development-checklist.md`
