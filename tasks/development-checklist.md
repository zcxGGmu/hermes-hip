# Hermeship 开发进度清单

本文用于跟踪 `hermeship` 的后续迭代开发进度。

方案文档：`docs/plans/2026-06-15-hermeship-development-plan.md`

## 跟踪规则

- [ ] 同一时间只推进一个 milestone。
- [ ] 每次实现会话结束前更新本清单。
- [ ] 只有运行过对应验证命令后，才能勾选任务完成。
- [ ] 每完成一个阶段任务就提交一次。
- [ ] commit 信息使用中文，说明完成内容、验证结果和影响。
- [ ] MVP 阶段不修改 Hermes 核心。
- [ ] MVP 阶段不修改 clawhip，除非本清单被显式修订。
- [ ] Observer plugin 必须等 gateway hook MVP 完成 live verification 后再启动。
- [ ] 未执行的 live check 必须记录原因和剩余风险。

## 全局完成定义

- [ ] `pytest -q` 通过。
- [ ] `ruff check .` 通过，或记录暂未启用 ruff 的原因。
- [ ] `python -m hermeship --help` 退出码为 0。
- [ ] `python -m hermeship emit-sample --event agent:start --dry-run` 输出映射后的 JSON。
- [ ] `hermeship install-hook --home /tmp/hermeship-test-home --force` 写入 hook 文件。
- [ ] `clawhip status` 已在运行中的 daemon 上验证。
- [ ] Discord live delivery 已验证，或明确记录未验证原因。
- [ ] 在一次性 `HERMES_HOME` 中测试过回滚路径。
- [ ] 日志和测试 fixture 中没有完整对话、prompt、provider 请求/响应、token、cookie 或 secret。
- [ ] README、operations 文档和实际 CLI 一致。

## Milestone 0：仓库卫生

目标：确保仓库处于可迭代开发状态。

- [ ] 确认当前分支和远程。
  - 命令：`git status --short --branch`
  - 完成标准：分支、远程和未提交变更清楚。
- [ ] 确认当前文件结构。
  - 命令：`find . -maxdepth 3 -type f | sort`
  - 完成标准：实现不会覆盖无关文件。
- [ ] 复习 lessons。
  - 文件：`tasks/lessons.md`
  - 完成标准：确认阶段完成后验证并提交。
- [ ] 复习方案文档。
  - 文件：`docs/plans/2026-06-15-hermeship-development-plan.md`
  - 完成标准：当前实现顺序和本清单一致。
- [ ] 更新 README 项目定位。
  - 文件：`README.md`
  - 完成标准：说明 Hermeship 是 Hermes 到 clawhip 的适配层，不是 clawhip fork。
- [ ] 提交仓库卫生阶段。
  - 验证：`git status --short`
  - commit：`docs: 明确 hermeship 项目定位`

## Milestone 1：本地 Dry-run 基础

目标：在不依赖 clawhip daemon 的情况下完成 package、config、privacy、mapper。

### 任务 1.1：包骨架与 CLI 入口

- [ ] 编写 CLI import/help 失败测试。
  - 新建：`tests/test_cli.py`
  - 验证失败：`pytest tests/test_cli.py -q`
- [ ] 新建 package metadata。
  - 新建：`pyproject.toml`
  - 包含：project metadata、console script、dev extras、pytest 配置。
- [ ] 新建 Python package 入口。
  - 新建：`src/hermeship/__init__.py`
  - 新建：`src/hermeship/__main__.py`
  - 新建：`src/hermeship/cli.py`
- [ ] 实现最小 `main()` 和 `--help`。
  - 完成标准：`python -m hermeship --help` 退出码为 0。
- [ ] 更新 README quickstart 占位内容。
  - 修改：`README.md`
- [ ] 验证任务 1.1。
  - 命令：`python -m pip install -e ".[dev]"`
  - 命令：`pytest tests/test_cli.py -q`
  - 命令：`python -m hermeship --help`
- [ ] 提交任务 1.1。
  - commit：`chore: 搭建 hermeship Python 包骨架`

### 任务 1.2：配置加载器

- [ ] 编写默认配置测试。
  - 新建：`tests/test_config.py`
  - 覆盖：缺失配置文件、默认值、空字符串归一化。
- [ ] 编写配置覆盖测试。
  - 覆盖：用户配置、环境变量覆盖、非法 TOML、未知 key。
- [ ] 实现配置 dataclass。
  - 新建：`src/hermeship/config.py`
  - 包含：`ClawhipConfig`、`DefaultsConfig`、`EventsConfig`、`PrivacyConfig`、`HermeshipConfig`。
- [ ] 实现配置路径解析。
  - 包含：`default_config_path()`、可选 `repo_config_path()`、`load_config()`。
- [ ] 实现环境变量解析。
  - 覆盖：`HERMESHIP_DRY_RUN` 布尔值解析。
- [ ] 实现配置校验。
  - 校验：`mode`、`format`、`timeout_secs`、`dedupe_window_secs`、未知 key。
- [ ] 验证任务 1.2。
  - 命令：`pytest tests/test_config.py -q`
- [ ] 提交任务 1.2。
  - commit：`feat: 实现 hermeship 配置加载器`

### 任务 1.3：隐私与脱敏

- [ ] 编写脱敏测试。
  - 新建：`tests/test_privacy.py`
  - 覆盖：嵌套 dict、list、大小写不敏感 key、非字符串值。
- [ ] 编写截断测试。
  - 覆盖：不截断、刚好等长、超长文本、`None`。
- [ ] 实现隐私 helper。
  - 新建：`src/hermeship/privacy.py`
  - 函数：`truncate_text()`、`redact_payload()`、可选 `sanitize_context()`。
- [ ] 验证不会原地修改输入 payload。
  - 完成标准：redaction 后原始对象不变。
- [ ] 验证任务 1.3。
  - 命令：`pytest tests/test_privacy.py -q`
- [ ] 提交任务 1.3。
  - commit：`feat: 增加 payload 脱敏与截断工具`

### 任务 1.4：事件模型与映射器

- [ ] 编写生命周期映射测试。
  - 新建：`tests/test_mapper.py`
  - 覆盖：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:end`。
- [ ] 编写错误映射测试。
  - 覆盖：`agent:end` with `error`、`exception`、`status=failed`。
- [ ] 编写禁用/未知事件测试。
  - 覆盖：配置禁用事件、未知事件返回 `None`。
- [ ] 编写隐私映射测试。
  - 覆盖：message/response 截断、secret 脱敏、不携带 `conversation_history`。
- [ ] 实现事件 dataclass。
  - 新建：`src/hermeship/events.py`
  - 包含：`MappedEvent`。
- [ ] 实现 mapper。
  - 新建：`src/hermeship/mapper.py`
  - 包含：`map_hermes_event()`、event enabled helper。
- [ ] 新增事件映射文档。
  - 新建：`docs/event-mapping.md`
  - 内容：支持事件、输出事件、payload 字段、隐私规则。
- [ ] 验证任务 1.4。
  - 命令：`pytest tests/test_mapper.py -q`
- [ ] 验证 Milestone 1。
  - 命令：`pytest -q`
  - 命令：`python -m hermeship --help`
- [ ] 提交任务 1.4。
  - commit：`feat: 实现 Hermes 生命周期事件映射`

## Milestone 2：clawhip CLI 投递

目标：通过 clawhip 已有 CLI 入口投递 Hermeship 映射事件。

### 任务 2.1：客户端契约与命令生成

- [ ] 编写 fake runner 测试。
  - 新建：`tests/test_clawhip_client.py`
  - 覆盖：`agent.started`、`agent.finished`、`agent.blocked`、`agent.failed`。
- [ ] 编写 custom emit 命令测试。
  - 覆盖：`hermes.session.started`、JSON payload、channel、mention。
- [ ] 实现 client class。
  - 新建：`src/hermeship/clawhip_client.py`
  - 包含：`ClawhipClient`、runner seam、timeout、dry-run。
- [ ] 使用无 shell 的命令构造。
  - 要求：`subprocess.run(list[str], shell=False, ...)`。
- [ ] 验证任务 2.1。
  - 命令：`pytest tests/test_clawhip_client.py -q`
- [ ] 提交任务 2.1。
  - commit：`feat: 生成 clawhip 投递命令`

### 任务 2.2：失败语义

- [ ] 测试 clawhip binary 缺失。
  - 预期：返回 warning，不抛异常。
- [ ] 测试 timeout。
  - 预期：返回 warning，不抛异常。
- [ ] 测试非零退出码。
  - 预期：捕获 stderr tail，不抛异常。
- [ ] 测试序列化失败 fallback。
  - 预期：跳过事件并输出诊断。
- [ ] 实现结构化 send result。
  - 字段：`sent`、`skipped`、`reason`、`stderr_tail`、`command`。
- [ ] 验证任务 2.2。
  - 命令：`pytest tests/test_clawhip_client.py -q`
- [ ] 提交任务 2.2。
  - commit：`feat: 让 clawhip 投递失败保持 fail-open`

### 任务 2.3：Dry-run 与样例事件

- [ ] 新增 CLI `emit-sample`。
  - 修改：`src/hermeship/cli.py`
  - flags：`--event`、`--dry-run`、`--channel`、`--project`。
- [ ] 测试 dry-run 输出 JSON。
  - 覆盖：不调用 subprocess。
- [ ] 测试样例事件映射。
  - 覆盖：`agent:start`、`agent:end`、`session:start`、`session:end`。
- [ ] 验证任务 2.3。
  - 命令：`pytest tests/test_cli.py tests/test_clawhip_client.py -q`
  - 命令：`python -m hermeship emit-sample --event agent:start --dry-run`
- [ ] 验证 Milestone 2。
  - 命令：`pytest -q`
  - 命令：`python -m hermeship emit-sample --event agent:start --dry-run`
- [ ] 提交任务 2.3。
  - commit：`feat: 增加 dry-run 样例事件`

## Milestone 3：Hermes Hook 安装

目标：Hermeship 能安装 fail-open 的 Hermes gateway hook bundle。

### 任务 3.1：Hook 模板

- [ ] 创建 hook manifest 模板。
  - 新建：`templates/hermes-hook/HOOK.yaml`
  - 事件：`gateway:startup`、`session:start`、`session:end`、`session:reset`、`agent:start`、`agent:end`。
- [ ] 创建 hook handler 模板。
  - 新建：`templates/hermes-hook/handler.py`
  - 内容：导入并暴露 `hermeship.hook_handler.handle`。
- [ ] 编写模板完整性测试。
  - 新建：`tests/test_hook_template.py`
  - 覆盖：manifest 结构或基本内容。
- [ ] 验证任务 3.1。
  - 命令：`pytest tests/test_hook_template.py -q`
- [ ] 提交任务 3.1。
  - commit：`feat: 增加 Hermes hook 模板`

### 任务 3.2：Hook 运行时 handler

- [ ] 编写 handler 转发测试。
  - 新建：`tests/test_hook_handler.py`
  - 覆盖：`agent:start` 转发 mapped event。
- [ ] 编写 fail-open 测试。
  - 覆盖：配置解析失败、mapper 失败、client 失败。
- [ ] 编写防递归测试。
  - 覆盖：`origin=hermeship` 的 context 被忽略。
- [ ] 编写去重测试。
  - 覆盖：同一事件/session 在 dedupe window 内只转发一次。
- [ ] 实现 runtime handler。
  - 新建：`src/hermeship/hook_handler.py`
  - 包含：`handle(event_type, context)`、`send_event()`、dedupe state。
- [ ] 验证任务 3.2。
  - 命令：`pytest tests/test_hook_handler.py -q`
- [ ] 提交任务 3.2。
  - commit：`feat: 实现 fail-open 的 Hermes hook handler`

### 任务 3.3：Installer 与 Doctor

- [ ] 编写 installer 测试。
  - 新建：`tests/test_installer.py`
  - 覆盖：首次安装、不覆盖、force install、返回路径。
- [ ] 实现 installer。
  - 新建：`src/hermeship/installer.py`
  - 函数：`install_hook(home: Path, force: bool = False) -> Path`。
- [ ] 新增 CLI `install-hook`。
  - 修改：`src/hermeship/cli.py`
  - flags：`--home`、`--force`。
- [ ] 新增 CLI `doctor`。
  - 检查：package import、hook 是否安装、clawhip binary、clawhip status。
- [ ] 新增运维文档。
  - 新建：`docs/operations.md`
  - 内容：安装、验证、更新、回滚。
- [ ] 验证任务 3.3。
  - 命令：`pytest tests/test_installer.py -q`
  - 命令：`python -m hermeship install-hook --home /tmp/hermeship-test-home --force`
  - 命令：`find /tmp/hermeship-test-home/hooks/hermeship-clawhip -maxdepth 1 -type f -print`
- [ ] 验证 Milestone 3。
  - 命令：`pytest -q`
  - 命令：`python -m hermeship doctor`
- [ ] 提交任务 3.3。
  - commit：`feat: 支持安装 Hermes hook bundle`

## Milestone 4：文档与 Live Verification

目标：证明 Hermeship 能与 clawhip 联通，并给 operator 清晰操作路径。

### 任务 4.1：README 与运维文档

- [ ] 重写 README quickstart。
  - 内容：Hermeship 是什么、安装、配置 clawhip、安装 hook、dry-run、live check。
- [ ] 增加 rollback 章节。
  - 内容：删除 hook 目录、卸载 package、必要时重启 Hermes。
- [ ] 增加隐私章节。
  - 内容：默认不会转发哪些内容。
- [ ] 链接方案与事件映射文档。
  - 文件：`README.md`、`docs/event-mapping.md`、`docs/operations.md`。
- [ ] 验证任务 4.1。
  - 命令：`rg -n "install-hook|emit-sample|rollback|privacy|clawhip" README.md docs`
- [ ] 提交任务 4.1。
  - commit：`docs: 增加 Hermeship 运维说明`

### 任务 4.2：Live Verification Runbook

- [ ] 创建 live verification 文档。
  - 新建：`docs/live-verification.md`
- [ ] 记录前置条件。
  - 包含：运行中的 clawhip daemon、Discord 测试频道、独立 clawhip bot token。
- [ ] 记录 dry-run 验证。
  - 命令：`python -m hermeship emit-sample --event agent:start --dry-run`。
- [ ] 记录 clawhip daemon 验证。
  - 命令：`clawhip start`、`clawhip status`、sample events。
- [ ] 记录 Hermes hook smoke 验证。
  - 使用隔离 `HERMES_HOME`。
- [ ] 记录预期 Discord 消息。
  - 包含：start、finish、session started、session finished。
- [ ] 验证任务 4.2。
  - 命令：`rg -n "clawhip status|HERMES_HOME|Discord|emit-sample" docs/live-verification.md`
- [ ] 提交任务 4.2。
  - commit：`docs: 增加 live verification runbook`

### 任务 4.3：首次 Live Check

- [ ] 启动或确认 clawhip daemon。
  - 命令：`clawhip status`
  - 如果不可用：在本清单记录原因。
- [ ] 发送 dry-run 样例。
  - 命令：`python -m hermeship emit-sample --event agent:start --dry-run`
- [ ] 发送 live start 样例。
  - 命令：`python -m hermeship emit-sample --event agent:start`
- [ ] 发送 live finish 样例。
  - 命令：`python -m hermeship emit-sample --event agent:end`
- [ ] 确认 Discord 投递。
  - 记录：频道、时间、消息形态。
- [ ] 在一次性 home 测试回滚。
  - 命令：`rm -rf /tmp/hermeship-test-home/hooks/hermeship-clawhip`
- [ ] 验证 Milestone 4。
  - 命令：`pytest -q`
  - 命令：`python -m hermeship --help`
- [ ] 如文档变化则提交 live verification 记录。
  - commit：`docs: 记录 Hermeship live verification 结果`

## Milestone 5：CI 与发布准备

目标：让 MVP 可重复验证、可发布、可回滚。

### 任务 5.1：Lint 与测试自动化

- [ ] 增加 ruff 配置。
  - 修改：`pyproject.toml`
- [ ] 确认 editable install 下测试通过。
  - 命令：`python -m pip install -e ".[dev]"`
  - 命令：`ruff check .`
  - 命令：`pytest -q`
- [ ] 增加 fake clawhip binary 测试 fixture。
  - 文件：`tests/conftest.py`
  - 目的：不依赖 daemon 验证 command integration。
- [ ] 验证任务 5.1。
  - 命令：`ruff check .`
  - 命令：`pytest -q`
- [ ] 提交任务 5.1。
  - commit：`test: 增加 lint 与 fake clawhip 覆盖`

### 任务 5.2：构建与打包

- [ ] 增加 package build 验证。
  - 命令：`python -m build`
- [ ] 检查 distribution 内容。
  - 确认：hook templates 和 docs 被包含。
- [ ] 如果模板缺失，增加 packaging 测试。
  - 覆盖：安装后的 package 能定位 template 文件。
- [ ] 验证任务 5.2。
  - 命令：`python -m build`
  - 命令：`python -m pip install dist/*.whl --force-reinstall`
  - 命令：`python -m hermeship --help`
- [ ] 提交任务 5.2。
  - commit：`chore: 验证 hermeship 打包产物`

### 任务 5.3：发布门禁

- [ ] 复核配置 schema 稳定性。
  - 完成标准：没有待定的用户可见 key 重命名。
- [ ] 复核事件映射稳定性。
  - 完成标准：事件名与 `docs/event-mapping.md` 一致。
- [ ] 复核隐私默认值。
  - 完成标准：默认不转发高风险字段。
- [ ] 复核回滚路径。
  - 完成标准：在一次性 `HERMES_HOME` 中回滚成功。
- [ ] 运行完整发布验证。
  - 命令：`ruff check .`
  - 命令：`pytest -q`
  - 命令：`python -m hermeship emit-sample --event agent:start --dry-run`
  - 命令：`python -m build`
- [ ] 提交发布准备更新。
  - commit：`chore: 准备 hermeship v0.1.0`

## Milestone 6：可选 Observer Plugin 研究

目标：在 gateway hook MVP 稳定后，增加更高保真 telemetry。

门禁：Milestone 1-5 完成或被明确豁免前，不启动本阶段。

### 任务 6.1：Observer Hook 运行时研究

- [ ] 复读 Hermes observer 文档。
  - 文件：`/home/zq/work-space/repo/ai-projs/agents/hermes-agent/docs/observability/README.md`
- [ ] 确认 plugin 安装机制。
  - 完成标准：记录准确 plugin 目录和启用命令。
- [ ] 起草 observer event mapping。
  - 新建或更新：`docs/observer-plugin.md`
- [ ] 决定 observer mode 进入 v0.2 还是更晚。
  - 记录决策到本清单。

### 任务 6.2：Observer Plugin MVP

- [ ] 编写 observer plugin 测试。
  - 新建：`tests/test_observer_plugin.py`
- [ ] 实现只读 observer callback。
  - 新建：`src/hermeship/observer_plugin.py`
  - 新建：`templates/hermes-plugin/hermeship_observer.py`
- [ ] 验证隐私默认值。
  - 测试：默认丢弃 request/response body。
- [ ] 验证 observer mode 不依赖 gateway hook。
  - 命令：运行 targeted tests。
- [ ] 提交 observer plugin。
  - commit：`feat: 增加可选 Hermes observer 转发`

## 运行状态日志

最新记录放在最上方。

### 2026-06-15

- [x] 创建初版开发方案。
- [x] 创建迭代开发清单。
- [x] 将方案和清单拆分为中文文档。
- [ ] 实现尚未开始。

## 阻塞项

- [ ] 确认 live Discord verification 凭据是否可用。
- [ ] 确认开发环境是否已安装 `clawhip` binary。
- [ ] 确认 v0.1.0 是否必须真实运行 Hermes gateway，还是允许先做手动 post-release verification。

## 决策记录

- [x] MVP 使用 Hermes gateway hooks，不 patch Hermes core。
- [x] MVP 使用 `clawhip agent` 和 `clawhip emit`，不新增 clawhip native provider contract。
- [x] MVP 优先标准库，HTTP transport 延后。
- [x] 默认不转发 `command:*`。
- [x] 方案文档和进度清单分离维护。
