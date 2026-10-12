# .github/workflows/
> L2 | 父级: ../CLAUDE.md
windows.yml: 社区 Windows Server 2022 构建与前端/Python/原生测试、安装后宿主/浏览器 smoke 检查，输出 NSIS 与 SHA256SUMS。
windows-release.yml: 仅从本仓库 main 手动触发；验证成功 Windows 构建、草稿目标和安装包摘要，将现有字节上传到预发布草稿并核验远端摘要。只授予本任务 actions:read / contents:write，不覆盖不同附件，不自动公开草稿。
release.yml: 仅 liseami 在 liseami/CCBaga 的 release push/手动触发及重跑可发布；统一版本后并行 macOS、Ubuntu x86_64 和固定摘要 Arch；Linux 无签名凭据且解包核验 DEB/RPM/AppImage/pacman 后才允许发布；arm64 runner 哈希安装 DMG 工具、缓存 Rust 依赖（版本号仅构建步骤注入，不进入依赖缓存键）、测试前端/原生/Skill/开发安装边界 并签名公证，临时钥匙串持有 Secrets，固定 SHA Actions；独立最小权限发布任务读取不可变产物，校验实际签名更新包并交付 latest.json，成功后显式固定并核对递增版本 tag，用当前产品与安装说明发布 Release（不生成历史比较列表），避免分支推进后隐式建 tag 的权限失败，公开仓库附带来源证明。
[PROTOCOL]: 变更时更新此头部，然后检查 CLAUDE.md
