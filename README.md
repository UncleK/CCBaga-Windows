# CCBaga Windows

**简体中文** | [English](README.en.md)

基于 [liseami/CCBaga](https://github.com/liseami/CCBaga) 的 Windows 10/11 x64 原生移植，使用 Rust、Tauri 和 React，无需 WSL。

> **状态：实验性开发，尚未完成 Windows 验收。** Windows 源码正在恢复和同步；目前没有经过验证的 Windows 安装包。上游 macOS/Linux 安装包不能用于 Windows。

## 适配范围

- Claude Code / Desktop 当前用户数据扫描，预览后按所选项目清理。
- Windows 路径、文件身份、权限、junction 检查与进程保护。
- 固定版本指纹浏览器、独立资料目录、缓存管理和实际环境观测。
- Windows 时区、语言和区域格式；定位通过系统设置手动处理并刷新核验。
- 中英双语界面、PowerShell 启动脚本、NSIS 安装包和 Windows CI。

凭据管理器、Microsoft Store 沙箱、原生/WinGet CLI 管理、默认浏览器注册和自动更新不自动处理。上游独立 Network Skill 的 Windows 主机兼容性尚未验收。

## 构建与验证

源码构建需要 Windows 10/11 x64、Visual Studio 2022 C++ Build Tools、Rust MSVC、Node.js、Bun 和 Python 3.11+。Windows 构建说明会随移植代码提交。

请以本仓库 Windows Actions 的实际结果为准。编译通过不等于真实电脑运行通过；安装、启动、系统授权与浏览器联网均需分别验证。

## 数据边界

**确认清理后直接删除，不经过回收站，不自动备份，也无法撤回。** 清理应保留项目源码、skills、plugins 和 MCP 配置。专用浏览器不使用日常 Chrome/Edge 资料目录。

环境检测会实际访问 `ipwho.is`，失败时经同一路由使用 `ipapi.co`。检测通过只说明已测项目一致，不保证匿名性或账号可用性；DNS/WebRTC 不宣称已经独立核验。

## 来源与许可

保留上游历史和 [MIT LICENSE](LICENSE)，Windows 改动沿用 MIT。第三方组件、引擎和素材保留各自许可，见 [NOTICE.md](NOTICE.md)。时区映射使用 Unicode CLDR，见 [LICENSE.unicode](docs/LICENSE.unicode)。

这是独立的社区 Windows 移植，不是上游官方 Windows 发行版，也不隶属于 Anthropic、Microsoft 或 Apple。
