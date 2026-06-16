# Hermeship Operations

本文记录 Hermeship 当前可用的本地运维路径。默认命令只做本地文件操作，不会自动运行 `systemctl` 或 `launchctl`。

## Install

```bash
hermeship install
hermeship setup --default-channel <channel-id>
hermeship hermes install-hooks --scope global --force
hermeship start
```

`hermeship install` 会创建：

```text
~/.hermeship/
  config.toml
  hooks/
  logs/
  state/
```

可重复 dry-run：

```bash
hermeship install --dry-run
```

## Setup

```bash
hermeship setup \
  --discord-token-stdin \
  --default-channel <channel-id> \
  --daemon-url http://127.0.0.1:25295
```

命令会从 stdin 读取 Discord token，避免 token 出现在 shell history 或 process argv 中。输出会将 Discord token 脱敏，不应把 token 打到日志中。

也可以从环境变量读取 token：

```bash
hermeship setup --discord-token-env HERMESHIP_SETUP_DISCORD_TOKEN
```

## Service

Systemd user service 模板位于：

```text
deploy/hermeship.service
```

当前阶段只提交模板，不自动安装。需要手动安装时可按本机策略复制到 user service 目录后启用。

macOS launchd 可使用同等语义的用户级 plist，关键点是设置 `HERMESHIP_CONFIG` 并运行 `hermeship start`。示例：

```xml
<key>ProgramArguments</key>
<array>
  <string>/Users/you/.cargo/bin/hermeship</string>
  <string>start</string>
</array>
<key>EnvironmentVariables</key>
<dict>
  <key>HERMESHIP_CONFIG</key>
  <string>/Users/you/.hermeship/config.toml</string>
</dict>
```

## Uninstall

默认卸载不会删除用户配置：

```bash
hermeship uninstall
```

显式删除本地状态、日志、配置和 Hermeship-managed Hermes hooks：

```bash
hermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes
```

如果省略 `--hermes-home`，`--remove-hooks` 会使用 `HERMES_HOME` 或 `~/.hermes`。Hermes hook 删除复用 `.hermeship-managed.json` marker，只删除 Hermeship 管理且未被用户修改的 hook 文件。

## Release Preflight

```bash
hermeship release preflight 0.1.0
```

Preflight 只检查本地一致性：Cargo 版本、公开 CLI fixture、hook 模板、service 模板、fixture policy 和文档命令。真实 Discord/Hermes live verification 仍单独记录，缺失时显示 `pending`。
