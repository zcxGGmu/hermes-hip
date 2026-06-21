# Task: 2026-06-21 README 顶部品牌区视觉优化

更新时间：2026-06-21

用户反馈当前 README 顶部图像和布局问题较大，并要求优化。本轮范围限定为公开 README 顶部品牌区、语言切换呈现、必要状态记录和静态资产检查；不修改 Rust 功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。

## 当前基线

- 当前分支：`codex/milestone-1-cli`。
- 启动时工作树：干净，当前 HEAD 为 `d8cf03c docs: 增加 HERMES-HIP README 艺术字`。
- 已复习：`tasks/lessons.md`。
- 已读取 UI/UX 规则：`ui-ux-pro-max`。
- 当前 README 顶部使用 HTML table 左右并排展示 `docs/assets/branding/hermeship-wordmark.svg` 和 `docs/assets/branding/hermeship-icon.png`，GitHub 渲染会暴露表格边框与竖向分隔线。

## 本轮执行计划

- [x] 复习项目 lessons、当前 Git 状态和 README 顶部实现。
  - 命令：`sed -n '1,260p' tasks/lessons.md`。
  - 命令：`git status --short --branch && git log -5 --oneline`。
  - 命令：`sed -n '1,220p' README.md`。
  - 命令：`sed -n '1,220p' README.en.md`。

- [x] 优化 README 顶部品牌区。
  - 文件：`README.md`。
  - 文件：`README.en.md`。
  - 要求：避免表格边框/竖线造成的生硬分割。
  - 要求：统一 wordmark、图标、标题和语言切换的层级。
  - 要求：保留分文件双语入口，公开 README 不出现相关项目关联表述。
  - 结果：新增统一 `docs/assets/branding/hermeship-lockup.png`，两个 README 顶部改为无表格的居中品牌图、语义标题、副标题和对称语言切换。

- [x] 检查和必要时微调品牌资产。
  - 文件：`docs/assets/branding/hermeship-wordmark.svg`。
  - 文件：`docs/assets/branding/hermeship-icon.png`。
  - 要求：资产引用继续使用仓库内相对路径，不依赖本地桌面绝对路径或远程资源。
  - 结果：保留原 wordmark/icon 资产，并使用 bundled Node `sharp` 合成为稳定 PNG brand lockup，避免 SVG 字体渲染差异和 GitHub Markdown 表格响应式问题。

- [x] 更新状态记录。
  - 修改：`docs/development-status.md`。
  - 修改：`tasks/development-checklist.md`。
  - 修改：`tasks/todo.md` Review。

- [x] 运行验证。
  - 命令：`python3 - <<'PY' ... ElementTree.parse('docs/assets/branding/hermeship-wordmark.svg') ... PY`。
  - 命令：`rg -n "hermeship-lockup.png|README.en.md|README.md|<table|img.shields.io" README.md README.en.md`。
  - 命令：`rg -n -i "clawhip|template/clawhip|thin adapter|runtime adapter" README.md README.en.md`，预期无匹配。
  - 命令：`rg -n "真实 Discord/Hermes live verification pass 尚未获得|Real Discord/Hermes live verification has not passed yet|Slack sink|observer plugin|deterministic" README.md README.en.md`。
  - 命令：`git diff --check`。
  - 命令：`python3 -m py_compile templates/hermes-plugin/__init__.py`。
  - 命令：`cargo fmt --all -- --check`。
  - 命令：`cargo test observer_plugin`。
  - 命令：`cargo test release_preflight`。
  - 命令：`cargo run -- release preflight 0.1.0`。
  - 命令：`cargo clippy --all-targets -- -D warnings`。
  - 命令：`cargo test`。

- [x] 阶段提交。
  - 提交前检查：`git status --short --branch`、`git diff --stat`、`git diff --name-only`。
  - commit 信息：中文说明 README 顶部品牌区视觉优化、验证和影响。

## Review

- 已按用户反馈优化 README 顶部品牌区：移除 HTML table 拼接布局，避免 GitHub README 渲染出现表格边框、竖向分割线和响应式脆弱问题。
- 已新增统一仓库内品牌横幅：`docs/assets/branding/hermeship-lockup.png`，由现有 `HERMES-HIP` wordmark 和项目图标合成，尺寸为 1280 x 360 PNG。
- 已更新 `README.md` 和 `README.en.md` 顶部为单一居中 banner、语义 `h1` 项目名、简短副标题和对称语言切换；语言仍保持分文件入口，中文为 `README.md`，英文为 `README.en.md`。
- 已将 README 顶部装饰性 banner 的 `alt` 置空，避免读屏重复项目名；公开 README 仍保留真实能力边界声明。
- 已更新 `tasks/lessons.md`，记录“README 顶部品牌区不要用表格拼图”的规则，避免后续重复同类布局问题。
- 本轮只修改公开 README、静态品牌资产和状态记录，不修改功能代码，不执行真实 Discord/Hermes live check，不实现 Slack sink，不自动启用 Hermes observer plugin。
- 已运行验证：`docs/assets/branding/hermeship-lockup.png` 确认为 1280 x 360 PNG，`docs/assets/branding/hermeship-wordmark.svg` XML 解析通过，README 顶部引用检查通过，公开 README 相关项目残留关键词检查无匹配，关键能力边界声明检查通过，`git diff --check`，`python3 -m py_compile templates/hermes-plugin/__init__.py`，`cargo fmt --all -- --check`，`cargo test observer_plugin`（13 passed），`cargo test release_preflight`（16 passed），`cargo run -- release preflight 0.1.0`（9 checks ok，`live verification` 只证明记录字段存在），`cargo clippy --all-targets -- -D warnings`，`cargo test`（221 lib tests + 15 bin tests + doctests passed）。
