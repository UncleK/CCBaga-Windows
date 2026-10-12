# 参与 CCBaga Windows

**简体中文** | [English](CONTRIBUTING.md)

按 [Windows 构建指南](README.windows.md) 准备环境，使用现有锁文件和 UI 组件。本仓库是独立的社区 Windows 分支；上游 macOS/Linux 开发说明保留在模块文档和 [Linux 指南](docs/linux.md) 中。

## 提交变更

- 说明要解决的问题、修改后的行为、已执行的验证和剩余限制。
- 阅读相关 `CLAUDE.md` 模块说明，并随代码同步更新文件契约和模块地图。
- 原生操作放在 Rust/平台边界；UI 文案同时更新两份 YAML，用户指南保持中英对等。
- 保留清理范围、当前用户进程检查、文件身份/ACL 核验和凭据保护；修改这些边界时加入有针对性的回归检查。
- 在 Windows 运行 `bun run windows:test`；原生或打包变更另运行 `bun run windows:build`。CI 在一次性 Windows 环境安装产物并检查启动，这与人工 Windows 10/11 验收分别记录。
- 修改文档后检查本地链接；保留第三方许可说明，原创贡献沿用 [MIT License](LICENSE)。

Windows 构建由本分支维护者发布。上游官方发行仍属于 **liseami/CCBaga**，与本社区移植分别维护。

## 反馈问题

请提供 Windows 版本、应用版本、复现步骤、预期行为和脱敏错误。不要附上私钥、令牌、密码、含认证信息的订阅链接或个人文件内容。敏感安全细节请私下联系仓库所有者。
