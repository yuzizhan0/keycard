# Keycard 开源资料汇编

面向「开源路线」的仓库说明：文档地图、协议、上线前检查、与商业化并存时的注意点。  
（实现细节仍以 `README.md` 与 `docs/superpowers/specs/` 为准。）

---

## 1. 仓库文档地图

| 内容 | 路径 |
|------|------|
| 快速上手 / 构建 / CLI / 安全提示 | `README.md` |
| 贡献与测试 | `CONTRIBUTING.md` |
| 威胁模型与设计 | `docs/superpowers/specs/2026-03-29-keycard-design.md` |
| 实现计划（历史/任务拆分） | `docs/superpowers/plans/2026-03-29-keycard.md` |
| 手工测试备忘 | `docs/MANUAL_TEST.md` |
| CLI 前期配置（macOS） | `docs/cli-setup-macos.md` |
| CLI 前期配置（Windows） | `docs/cli-setup-windows.md` |
| 漏洞报告方式 | `SECURITY.md` |
| **许可证全文** | `LICENSE`（MIT） |
| 行为准则 | `CODE_OF_CONDUCT.md` |
| Issue / PR 模板 | `.github/ISSUE_TEMPLATE/`、`pull_request_template.md` |

---

## 2. 开源协议说明（MIT）

- 当前默认采用 **MIT**：使用、修改、再发行成本低，便于个人与企业采用。
- 典型义务：**保留版权声明与许可全文**（分发源码或二进制时带上 `LICENSE`）。
- 免责声明：软件按「原样」提供；敏感数据安全仍依赖用户环境与主密码强度。  
- 若你希望 **专利明示** 或与企业协议更一致，可评估改为 **Apache-2.0**，或采用 **MIT OR Apache-2.0** 双许可（需统一替换 `LICENSE` 与各 `Cargo.toml` 的 `license` 字段）。

---

## 3. 与「商业发行 / 支持」并存（开源友好）

以下与 MIT **不冲突**，常见于开源 + 商业：

| 模式 | 说明 |
|------|------|
| 双轨产物 | 仓库内源码永远可自构建；你售卖 **签名安装包、自动更新、人工支持**，卖的是发行与服务，不是独占功能许可。 |
| 商标 | 代码可 fork；**「Keycard」名称与 Logo** 可单独说明：他人可再发行改名版，避免冒充官方。建议在 README 增加 **Trademark** 一小节（见下文模板）。 |
| Open Core（可选） | 核心库长期开源；若将来有仅面向企业的模块，再单独拆仓库与协议，避免临时闭源争议。 |

**README 商标段模板（按需粘贴到 `README.md` 末尾）：**

```markdown
## Trademark

“Keycard” and the Keycard logo are trademarks of ZizhanYu.
You may use this project’s source code under the LICENSE, but you may not use
the marks to imply endorsement or an official distribution without permission.
```

（将 bracket 换成你的法律主体；暂无主体可写「项目维护者」并日后补全。）

---

## 4. 对外开源前检查清单

**法律与元信息**

- [ ] 根目录已有 `LICENSE`，且与 `Cargo.toml` / `package.json` 的 `license` 字段一致（若声明）。
- [ ] 第三方依赖许可证可接受（Rust/npm 可用 `cargo deny` / `npx license-checker` 等自行审计）。
- [ ] 无用户隐私数据、真实密钥、个人路径的误提交（必要时 `git filter-repo`）。

**工程与信任**

- [ ] `README.md` 写清：范围、**不负责托管密钥**、默认数据路径。
- [ ] `SECURITY.md` 写清：**私密上报渠道**（邮箱或 GitHub 非公开报告）。
- [ ] `CONTRIBUTING.md` 说明如何跑测试、最小 PR 期望。

**社区**

- [ ] Issue 模板（Bug / Feature）、PR 模板（可选）。
- [ ] 行为准则：可选用 [Contributor Covenant](https://www.contributor-covenant.org/)，另存 `CODE_OF_CONDUCT.md` 并在 README 链接。

**发布**

- [ ] 打 Tag 与 Release Notes（变更摘要、迁移注意）。
- [ ] 附着构建产物校验和（SHA256）与**最小**「已知限制」说明。

---

## 5. 建议的对外一句话

> **Keycard**：API 密钥等放在本地加密保险库（SQLite），配套桌面端与 CLI；**密钥默认不离机**；源码在 MIT 下开放，欢迎审计与贡献。

---

## 6. 维护者待办（GitHub 网页上操作）

- 在仓库 **Settings → Security** 中开启 **Private vulnerability reporting**（与 `SECURITY.md` 第 2 条呼应）。
- 在 **Settings → General → Features** 中按需开启 **Issues / Discussions**。
- 视需要新建 **`bug` / `enhancement` 标签**（与 Issue 模板默认 `labels` 一致；没有则模板仍可用，只是不会自动打标）。

商标与安全联系方式若变更，请同步更新 **`README.md`**（Trademark）、**`SECURITY.md`**、**`CODE_OF_CONDUCT.md`**（Enforcement 邮箱）。
