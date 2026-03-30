# Keycard CLI — Windows 配置指南

面向首次在 **Windows** 上使用 `keycard` 命令行的用户：安装、路径、保险库、Profile，以及 **与 POSIX `export` 有关的注意事项**。

---

## 1. 前置条件

- **Rust**：安装 [rustup -windows x86_64](https://rustup.rs/)，使用 **stable** toolchain。
- **Visual Studio C++ Build Tools**（编译 `keycard-cli` 常见依赖时通常需要）  
  - 安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，勾选 **「使用 C++ 的桌面开发」** 或至少 **「MSVC」** 与 Windows SDK。  
  - 若你使用 **GNU 工具链**（`x86_64-pc-windows-gnu`），可改用对应 target，但团队默认文档以 **MSVC** 为主。

安装后重新打开 **PowerShell** 或 **cmd**。

---

## 2. 获取 `keycard` 可执行文件

在仓库根目录（已 clone 本仓库）执行：

```powershell
cd C:\path\to\keycard
cargo build -p keycard-cli --release
```

产物路径：

```text
target\release\keycard.exe
```

### 加入 PATH（便于任意目录调用）

1. 将 `keycard.exe` 复制到固定目录，例如：  
   `C:\Users\<你>\AppData\Local\Keycard\bin\keycard.exe`
2. **设置 → 系统 → 关于 → 高级系统设置 → 环境变量 → 用户变量 Path**，添加上述目录。
3. **新开**一个 PowerShell / cmd 窗口，执行：

   ```powershell
   keycard.exe --help
   ```

若文件名在 Path 中，也可直接：

```powershell
keycard --help
```

---

## 3. 默认保险库路径（与桌面端一致）

| 平台 | 默认 `vault.db` |
|------|------------------|
| Windows | `%LOCALAPPDATA%\Keycard\vault.db` |

通常展开为：

```text
C:\Users\<用户名>\AppData\Local\Keycard\vault.db
```

自定义（与桌面共用同一文件时显式指定）：

```powershell
keycard --vault "$env:LOCALAPPDATA\Keycard\vault.db" …
```

或在 cmd：

```bat
keycard --vault "%LOCALAPPDATA%\Keycard\vault.db" …
```

---

## 4. 首次创建保险库

- **CLI**：在项目目录外也可执行：

  ```powershell
  keycard.exe init
  ```

  按提示输入主密码两次。

- **桌面端**：若在 App 中已完成「创建保险库」，则 CLI **不要**对同一路径再执行 `init`。

---

## 5. Profile（`env` / `run` 前必读）

与 macOS 相同：需存在 `profiles` 与 `profile_env` 数据。桌面端若暂无可视化「新建 Profile」，请在 **备份 vault.db、关闭 Keycard** 后，用 **SQLite CLI** 或 DB 浏览器维护（示例 SQL 见 `docs/cli-setup-macos.md` 第 5 节），并把路径换成本机：

```powershell
sqlite3 "$env:LOCALAPPDATA\Keycard\vault.db"
```

---

## 6. `keycard env` 与 Windows Shell（重要）

v1 的 `keycard env` 输出为 **POSIX 风格**：

```bash
export VAR='value'
```

这在 **bash**（Git Bash、MSYS2、WSL）里可以配合 `eval` 使用：

```bash
eval "$(keycard.exe env --profile dev)"
```

在 **PowerShell** 与 **cmd.exe** 中，**不能**直接 `eval` 上述输出。推荐做法：

### 推荐（全平台一致）

优先用 **单次子进程注入**（不依赖 shell 语法）：

```powershell
keycard.exe run --profile dev -- cargo build
keycard.exe run --profile dev -- npm run test
```

### 可选：Git Bash 用户

若日常已在 **Git for Windows** 的 Bash 里工作，可把 `keycard.exe` 放进该环境 `PATH`，并按 macOS 文档使用 `eval "$(keycard env -p dev)"`。

### 可选：手写导入 PowerShell（高级）

可把 `keycard env` 的输出来 **人工改编** 为：

```powershell
$env:OPENAI_API_KEY = '……'
```

（注意：不要把主密码或明文密钥提交到脚本仓库。）

---

## 7. `keycard run` 与路径中的空格

`--` 之后为子进程命令；路径含空格时建议加引号，或使用 `Run` 时当前工作目录已切换：

```powershell
keycard.exe run --profile dev -- & "C:\Program Files\nodejs\npm.cmd" run build
```

（具体引用方式取决于你使用的 shell。）

---

## 8. 已保存的 CLI 指令

```powershell
keycard.exe saved list
keycard.exe saved run "你的保存名称"
```

---

## 9. Windows 上没有的 macOS 功能

- **Automator「服务」** 为 macOS 专有；Windows 可把常用命令写成 **`.bat` / PowerShell 脚本** 或 **快捷方式**，自行调用 `keycard.exe`。

---

## 10. 常见问题

| 现象 | 处理 |
|------|------|
| 链接错误 / 缺 MSVC | 安装 VS Build Tools，或按 Rust 官网说明安装 Windows 依赖。 |
| `could not determine vault path` | 检查 `%LOCALAPPDATA%` 是否存在；或显式 `keycard --vault …`。 |
| PowerShell 里想 `env` 一整段 | 用 **Git Bash / WSL**，或只用 `keycard run`。 |
| 杀毒软件误报 | 自编译二进制可向安全软件提交白名单；开源可自行校验源码构建。 |

---

## 11. 相关文档

- 总体用法：`README.md` → CLI  
- Profile SQL 示例与日常流：`docs/cli-setup-macos.md` §5、§6、§7  
- 设计：`docs/superpowers/specs/2026-03-29-keycard-design.md`
