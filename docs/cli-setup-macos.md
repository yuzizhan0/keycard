# Keycard CLI — macOS 配置指南

面向首次在 **macOS** 上使用 `keycard` 命令行的用户：从安装、路径、保险库到 Profile 与日常使用。

---

## 1. 前置条件

- **Rust**：用 [rustup](https://rustup.rs/) 安装（stable 即可）。
- **Xcode Command Line Tools**（编译 GUI/部分依赖时可能需要）：

  ```bash
  xcode-select --install
  ```

---

## 2. 获取 `keycard` 可执行文件

### 方式 A：从源码构建（开发/自托管）

在仓库根目录：

```bash
cd /path/to/keycard
cargo build -p keycard-cli --release
```

二进制位置：

```text
target/release/keycard
```

### 方式 B：安装到 PATH（推荐习惯）

任选其一：

- **临时**：把 `target/release` 加入当前终端 `PATH`。
- **长期**：复制或软链到 `~/.local/bin` 或 `/usr/local/bin`，并确保 `~/.local/bin` 已写入 `~/.zshrc` / `~/.bash_profile`：

  ```bash
  mkdir -p ~/.local/bin
  cp target/release/keycard ~/.local/bin/
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
  source ~/.zshrc
  ```

验证：

```bash
keycard --version
```

（若尚未实现 `--version`，可先运行 `keycard init` 或在无子命令时看 clap 帮助。）

---

## 3. 默认保险库路径（与桌面端一致）

CLI 与桌面应用默认使用**同一**数据库文件：

| 平台 | 默认 `vault.db` |
|------|------------------|
| macOS | `~/Library/Application Support/Keycard/vault.db` |

自定义路径（与桌面端保持一致时可传同一文件）：

```bash
keycard --vault "$HOME/Library/Application Support/Keycard/vault.db" …
```

---

## 4. 首次创建保险库

### 与桌面端二选一即可

- **只用 CLI**：在项目目录执行：

  ```bash
  keycard init
  ```

  按提示设置主密码（两次）。

- **先用桌面端**：在 App 里完成「创建保险库」，会在上述默认路径生成 `vault.db`，CLI **无需再 `init`**，直接解锁使用即可。

> 不要对**同一路径**又 CLI `init` 又覆盖；一种创建方式即可。

---

## 5. Profile（使用 `env` / `run` 前必读）

`keycard env --profile …` 与 `keycard run --profile …` 依赖保险库里的 **Profile**：即「环境变量名 → 某条条目的密文」的映射。

当前桌面端主要提供**条目与已保存 CLI 指令**；**新建 Profile 及 `profile_env` 映射**若界面尚未覆盖，需在你熟悉 SQLite 且**已备份 `vault.db`**、并**退出 Keycard**（避免锁库冲突）的前提下维护，例如：

1. 在 App 里记下要映射的条目的 `id`（或通过备份后只读查看 `entries`）。
2. 关闭 Keycard。
3. 备份：  
   `cp ~/Library/Application\ Support/Keycard/vault.db ~/Desktop/vault.db.bak`
4. 只读检查条目：  
   `sqlite3 ~/Library/Application\ Support/Keycard/vault.db "SELECT id, alias FROM entries;"`

插入 Profile（示例：`dev` 为 profile id，与桌面里列表中的 **id** 一致；`entry-id` 换成真实条目 id）：

```sql
INSERT OR IGNORE INTO profiles (id, name) VALUES ('dev', 'dev');
INSERT OR IGNORE INTO profile_env (profile_id, env_var, entry_id)
VALUES ('dev', 'OPENAI_API_KEY', 'entry-id-uuid-here');
```

在 `sqlite3` 里执行前请再次确认**已备份**且应用已退出。后续若提供图形化 Profile 管理，以产品更新为准。

---

## 6. 在终端里使用环境变量（Bash / Zsh）

v1 的 `keycard env` 输出为 **POSIX `export VAR='…'`**，适合 **bash / zsh**：

```bash
eval "$(keycard env --profile dev)"
```

然后可直接运行依赖该环境变量的命令。

注意：`eval` 会把导出行为写进 shell 历史，存在**敏感操作被记录**的风险；生产环境可优先使用下面的 `run`。

---

## 7. 单次命令注入（推荐）

不污染当前 shell，只给**子进程**注入 Profile 环境：

```bash
keycard run --profile dev -- cargo build
keycard run --profile dev -- npm run dev
```

---

## 8. 已保存的 CLI 指令（与桌面端同步）

在桌面端「Saved CLI commands」保存后：

```bash
keycard saved list
keycard saved run "你的保存名称"
```

---

## 9. macOS 独有：把终端划选一键送进 Keycard

见仓库根目录 `README.md` 中 **「macOS: save a terminal selection…」** 与脚本 `scripts/macos/save-cli-to-keycard.sh`（Automator 快捷操作 + 服务）。

---

## 10. 常见问题

| 现象 | 处理 |
|------|------|
| `could not determine vault path` | 设置 `HOME` 或使用 `keycard --vault /完整路径/vault.db …`。 |
| `profile not found` | 检查 `profiles` 表是否有对应 id；`--profile` 可用 id 或名称（见 core 解析逻辑）。 |
| 桌面与 CLI 数据不一致 | 确认两者指向**同一** `vault.db` 路径。 |
| 仅测试/CI | 可用 `KEYCARD_ALLOW_ENV_PASSWORD=1` 与 `KEYCARD_MASTER_PASSWORD`（**切勿在日常环境使用**）。 |

---

## 11. 相关文档

- 总体用法：`README.md` → CLI  
- 威胁模型与设计：`docs/superpowers/specs/2026-03-29-keycard-design.md`  
- Windows：`docs/cli-setup-windows.md`
