# Task: 2026-06-21 README 独立叙述优化

更新时间：2026-06-21

用户反馈：README 不应出现和 clawhip 相关的内容，Hermeship 对外应呈现为完全独立的项目。本轮范围限定为 README 文档定位优化：移除 `README.md` 和 `README.en.md` 中的 clawhip、template、adapter 等关联表述，保留 Hermeship 自身能力、边界、图表和操作说明。默认不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，当前 HEAD 为 `7da47ce docs: 拆分 README 中英文入口并添加语言切换`。
- 已复习：`tasks/lessons.md`。
- 已确认 README 中的 clawhip 关联集中在项目定位开头：中文 README 和英文 README 均出现 `clawhip`、`template/clawhip` 或 adapter 说明。

## 本轮执行计划

- [x] 确认 Git 状态和最近提交。
  - 已运行：`git status --short --branch`。
  - 已运行：`git log -5 --oneline`。

- [x] 复习项目 lessons 并记录用户纠正。
  - 已读：`tasks/lessons.md`。
  - 已修改：新增“公开 README 必须是 Hermeship 独立叙述”的规则。

- [x] 优化 README 独立叙述。
  - 修改：`README.md`。
  - 修改：`README.en.md`。
  - 要求：不得出现 `clawhip`、`template/clawhip`、runtime adapter、thin adapter 等相关表述。
  - 要求：保留 Hermeship 的独立项目定位、Hermes 集成边界、Discord sink、observer plugin 手动启用、deterministic source、live verification 未完成等能力边界。

- [x] 更新状态记录。
  - 修改：`docs/development-status.md`。
  - 修改：`tasks/development-checklist.md`。
  - 修改：`tasks/todo.md` Review。

- [x] 运行验证。
  - 命令：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md`，预期无匹配。
  - 命令：`rg -n -i "claw" README.md README.en.md`，预期无匹配。
  - 命令：`rg -n "README.en.md|img.shields.io|真实 Discord/Hermes live verification pass 尚未获得|Real Discord/Hermes live verification has not passed yet|Slack sink|observer plugin|deterministic source" README.md README.en.md`。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`git diff --check`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。

- [x] 阶段提交。
  - 提交前检查：`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - commit 信息：中文说明 README 独立叙述优化、验证和影响。

## Review

- 已按用户反馈优化 README 公开定位：`README.md` 和 `README.en.md` 不再出现 `clawhip`、`template/clawhip`、thin adapter 或 runtime adapter 相关表述。
- 已将 README 开头改成正向独立叙述：Hermeship 是独立 Hermes-native daemon-first 事件通知路由器，拥有自己的事件契约、daemon、路由、渲染、投递和发布验证流程。
- 已保留关键能力边界声明：真实 Discord/Hermes live pass 未完成、`release preflight` 不证明真实 live pass、Slack sink 不在默认范围、observer plugin 需要手动启用、source 命令仍是 deterministic-only。
- 已更新 `tasks/lessons.md`，记录“公开 README 必须是 Hermeship 独立叙述”的规则。
- 已更新 `docs/development-status.md` 和 `tasks/development-checklist.md`，记录本轮 README 独立叙述优化。
- 本轮只修改 README 与状态记录，不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。
- 已运行验证：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md` 无匹配，`rg -n -i "claw" README.md README.en.md` 无匹配，关键边界声明检查通过，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed），`git diff --check`。
- 阶段提交前已确认 diff 范围仅包含 README 独立叙述优化和状态记录。
