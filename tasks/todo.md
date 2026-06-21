# Task: 2026-06-21 README 双语入口与 Claude Official 架构图

更新时间：2026-06-21

本轮任务从最新提交 `04fb880 docs: 记录 2026-06-20 Hermeship 本地验证续接` 继续，目标是深入复核 Hermeship 当前能力和 `template/clawhip` 参考形态，重写/补强项目根 `README.md` 为中英文双语 operational spec，并使用 `fireworks-tech-graph` 生成一组 Style 6（Claude Official）架构、流程和框架图。范围限定为文档和图表资产；不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，`git status --short --branch` 只显示分支行。
- 启动时 HEAD：`04fb880 docs: 记录 2026-06-20 Hermeship 本地验证续接`。
- 最近 5 个提交：`04fb880`、`b76a007`、`95a53d5`、`608704e`、`c226514`。
- 已复习：`tasks/lessons.md`。
- 已读取绘图 skill：`fireworks-tech-graph`，并选用 Style 6 Claude Official。
- 已确认 `template/clawhip` 只作为架构、文档结构和运行形态参考，不作为运行时依赖。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此不执行真实 Discord/Hermes live check。
- `release preflight` 的 `live verification` ok 只证明 `docs/live-verification.md` 记录字段存在，不断言真实 live pass。

## 本轮执行计划

- [x] 复习 lessons、确认 Git 状态和最近提交。
  - 已读：`tasks/lessons.md`。
  - 已运行：`git status --short --branch`。
  - 已运行：`git log -5 --oneline`。

- [x] 读取 Hermeship 当前项目文档和边界。
  - 已读：`README.md`。
  - 已读：`ARCHITECTURE.md`。
  - 已读：`docs/development-status.md`。
  - 已读：`tasks/development-checklist.md`。
  - 已读：`tasks/todo.md`。
  - 已读：`docs/live-verification.md`。
  - 已读：`docs/operations.md`。
  - 已读：`docs/hermes-event-contract.md`。
  - 已读：`docs/plans/2026-06-15-hermeship-development-plan.md`。
  - 已读：`src/release_preflight.rs`。
  - 已读：`tests/fixtures/README.md`。

- [x] 读取 `clawhip` 参考。
  - 路径：`/Users/zq/Desktop/ai-projs/posp/template/clawhip`。
  - 已读：`README.md`。
  - 已读：`ARCHITECTURE.md`。
  - 已读：`docs/event-contract-v1.md`。
  - 目标：参考其 README 的 operational spec 结构、daemon-first 描述、input -> behavior -> verification 写法和运行边界表达。

- [x] 生成 Style 6 图表资产。
  - 输出目录：`docs/assets/diagrams/`。
  - 图 1：`hermeship-architecture`，展示 Hermes hooks/plugin/CLI/source -> daemon ingress -> typed queue -> dispatcher/router/renderer/sink -> Discord。
  - 图 2：`hermeship-event-flow`，展示 `/event`、`/api/hermes/hook`、privacy sanitizer、typed envelope、route/render/deliver 和 live verification 边界。
  - 图 3：`hermeship-observer-framework`，展示 optional Hermes observer plugin、safe-field forwarding、typed Rust observer body、route aliases 和 fail-open/operator opt-in。
  - 每张图保留 `.json` 数据源、`.svg` 和 `.png`。

- [x] 重写根 README 为中英文双语入口。
  - 文件：`README.md`。
  - 结构：中文主体 + English mirror。
  - 必须包含：项目定位、当前能力、未完成边界、图表、安装配置、运行、Hermes hooks、observer plugin、source commands、route/render/privacy、live verification、release preflight、开发验证。
  - 必须明确：不是 clawhip thin adapter；不依赖 clawhip daemon；observer plugin 可选且需要 operator 手动启用；真实 Discord/Hermes live pass 尚未完成；默认不实现 Slack sink。

- [x] 更新状态文档和开发清单运行日志。
  - 文件：`docs/development-status.md`。
  - 文件：`tasks/development-checklist.md`。
  - 文件：`tasks/todo.md`。
  - 目标：记录本轮 README/图表文档阶段、验证结果、未执行真实 live check 的原因和剩余边界。

- [x] 运行图表验证。
  - 命令：`python3 -m json.tool docs/assets/diagrams/*.json`。
  - 命令：`python3 -c "import xml.etree.ElementTree as ET; ..."` 逐个解析 SVG。
  - 命令：使用 `cairosvg` 将每个 SVG 导出 PNG。
  - 命令：必要时读取 PNG 做基本视觉检查。

- [x] 运行项目验证。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。
  - 命令：`git diff --check`。

- [ ] 阶段提交。
  - 提交前检查：`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - commit 信息：中文说明 README 双语入口、Claude Official 图表、验证结果和影响。

## Review

- 已从 `04fb880 docs: 记录 2026-06-20 Hermeship 本地验证续接` 继续，复习 lessons、确认分支和最近提交，并读取 Hermeship 当前文档、状态入口、release preflight、fixture policy 和 `template/clawhip` 参考 README/ARCHITECTURE。
- 已重写 `README.md` 为中英文双语 operational spec：明确 Hermeship 是 Hermes-native daemon-first event router，不是 `clawhip` thin adapter；覆盖当前能力、未完成边界、安装配置、daemon、Hermes hooks、observer plugin、source commands、路由/渲染/隐私、live verification 和 release preflight。
- 已新增 `docs/assets/diagrams/` 下 3 组 Style 6（Claude Official）图表资产：`hermeship-architecture`、`hermeship-event-flow`、`hermeship-observer-framework`；每组包含 `.json` 数据源、`.svg` 和 `.png`。
- 已验证图表资产：3 个 JSON 解析通过、3 个 SVG XML 解析通过、3 个 PNG 均为 1280x760，并完成视觉抽查；`cairosvg` 因本机缺少 cairo 动态库不可用，实际使用 macOS `sips` 导出 PNG。
- 已更新 `docs/development-status.md`：记录 2026-06-21 README 双语入口与 Claude Official 图表阶段，明确不改变功能代码、不新增真实 live pass、不实现 Slack sink、不自动启用 Hermes observer plugin。
- 已更新 `tasks/development-checklist.md`：追加 2026-06-21 “README 双语入口与 Claude Official 图表”运行状态日志。
- 本轮未提供 Discord credentials、测试频道、Hermes gateway 测试环境或明确执行确认，因此没有执行真实 Discord/Hermes live check，也没有新增 `docs/live-verification.md` 真实 pass 结果。
- 本轮默认不实现 Slack sink，不自动启用 Hermes observer plugin，不修改功能代码。
- 已运行本轮验证：`python3 -m py_compile templates/hermes-plugin/__init__.py`、`cargo test observer_plugin`（13 passed）、`cargo test release_preflight`（16 passed）、`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在）、`cargo fmt --all -- --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（221 lib tests + 15 bin tests + doctests passed）。
- 阶段提交前仍需运行 `git diff --check`、确认 diff 范围并提交。
