# Windows 使用与构建

**简体中文** | [English](README.windows.en.md) | [项目介绍](README.md)

适用于 **Windows 10/11 x64**。这是独立的社区移植，仍处于实验阶段。自动构建和临时样本测试不能替代真实电脑上的安装、启动、授权和联网验收。

## 获取安装包

打开 [Windows 预览版发布页](https://github.com/UncleK/CCBaga-Windows/releases/tag/v0.3.0-windows-preview.1)，在 **Assets** 下载 `CCBaga-Windows-0.3.0-preview.1-x64-setup.exe` 和 `SHA256SUMS`。在下载目录打开 PowerShell，对比以下命令显示的摘要与校验文件中的值：

```powershell
Get-FileHash .\CCBaga-Windows-0.3.0-preview.1-x64-setup.exe -Algorithm SHA256
Get-Content .\SHA256SUMS
```

摘要一致后运行 `.exe`。直接安装无需 Rust、Node.js 或其他开发工具。预览版已在 Windows Server 2022 CI 完成测试和安装/启动检查，Windows 10/11 人工桌面验收仍未完成。

开发构建也可在 **Actions → Windows → 成功的构建 → Artifacts** 下载 `CCBaga-Windows-x64`。Actions ZIP 包含安装包与校验文件，需要登录 GitHub，保留 30 天；公开预览版安装包请优先使用上面的发布页。

安装包按当前用户安装，并包含独立 Chromium 引擎。缺少 Microsoft Edge WebView2 Runtime 时，安装程序会下载官方引导程序，因此首次安装可能需要联网。安装包暂未配置代码签名证书。不要关闭 Windows 安全保护来绕过安装问题。

## 源码构建

准备以下工具，并确认都在终端 PATH 中：

- Git、Node.js 与 npm、Bun 1.3.5。
- Rust stable，使用 `x86_64-pc-windows-msvc` 工具链。
- Visual Studio 2022 Build Tools 的 **使用 C++ 的桌面开发**，包含 MSVC 与 Windows SDK。
- Python 3.11 或更高版本；Windows PowerShell 5.1（Windows 自带）。

```powershell
git clone https://github.com/UncleK/CCBaga-Windows.git
cd CCBaga-Windows
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows.ps1 -Action Check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows.ps1 -Action Prepare
```

`ExecutionPolicy Bypass` 仅对该子进程生效，不修改系统执行策略。`Prepare` 安装锁定的前端依赖、下载并核验浏览器引擎，然后构建前端。首次下载较大，请保持网络连接。

开发启动：双击 `Start-Windows.cmd`，或运行 `bun run windows:dev`。

构建安装包：双击 `Build-Windows.cmd`，或运行 `bun run windows:build`。输出目录：

```text
src-tauri/target/release/bundle/nsis/
```

执行测试：

```powershell
bun run windows:test
```

脚本必须在 Windows 上运行。Windows 使用 `tauri.windows.conf.json`，无需 Bash、WSL、Xcode 或 macOS 引擎打包脚本。发布构建必须启用 `custom-protocol`，脚本已包含该参数。

## 功能与边界

| 功能 | Windows 行为 |
| --- | --- |
| Claude Code 清理 | 扫描当前用户 `.claude` 与 `.claude.json` 的白名单；配置只移除指定键。支持目录范围扫描。 |
| Claude Desktop 清理 | 扫描用户目录内 `%APPDATA%\Claude` 的缓存、日志和单独选择的站点数据；不整体删除 Desktop 配置目录。 |
| 文件保护 | 拒绝链接与 junction 祖先；执行前检查所有者、文件身份和快照；新配置使用当前用户私有 ACL 并原子替换。 |
| 进程保护 | 识别当前用户的已知安装路径和官方 npm CLI；结束前重新检查用户、路径与创建时间。无法确认身份时停止操作。 |
| CLI 卸载/重装 | 仅管理能核验为官方包的 npm 全局安装；保留配置、skills 和项目。原生、WinGet、Store 与自定义安装交由原安装器管理。 |
| 独立浏览器 | 使用专用资料目录；支持指纹参数、缓存清理、站点数据选择和回环实测页。不会使用日常 Chrome/Edge 资料。 |
| 时区与区域 | 使用 Unicode CLDR 转换 Windows/IANA 时区；修改首选语言和区域格式，保留其他语言。显示语言包可能需另外安装。 |
| 定位 | 打开 Windows 设置供手动关闭，再读取状态；读不到状态时显示未知，不报告已关闭。 |
| 默认浏览器 | 打开 Windows 默认应用设置；本版本尚未注册为默认浏览器。 |
| 凭据管理器 | 仅提供系统入口；不自动删除 Windows 凭据。 |
| 更新 | 从此仓库安装包或源码更新；禁用上游 macOS 自动更新。 |
| Network Skill | 保留上游独立 Skill 入口；Windows 主机执行流程尚未验收。应用不执行 SSH 或收集密钥。 |

系统管理策略可能阻止时区、语言或区域操作。普通用户运行即可；遇到权限不足时优先在 Windows 设置中手动修改，不建议以管理员身份执行清理。

**清理会永久删除所选数据，不进入回收站、不生成备份、不能撤回。** 默认缓存预设不选中登录、历史或配置。自定义 `CLAUDE_CONFIG_DIR`、Microsoft Store 沙箱、Windows 凭据管理器以及用户目录外重定向的 AppData 不会自动纳入扫描。

## 浏览器来源

引擎为 Fingerprint Chromium **148.0.7778.215** 的 Windows x64 ZIP：

```text
SHA-256: 9ef3f471b7a6641b4224532522b29141ce3746e27d55788d88e2fd951f362579
```

下载脚本先验证摘要，再检查 ZIP 路径、PE 架构和必要资源；源码仓库不存储引擎二进制。离线准备可先从脚本所列上游 URL 下载同一文件，再运行：

```powershell
python scripts/fetch-browser-windows.py --archive C:\Downloads\browser.zip
```

仍会执行同一摘要校验。已准备的引擎保存在 `browser-runtime/windows/`；需要重新下载时，先将该目录移走，再运行 `Prepare`。

## 常见问题

- **找不到编译器或链接器**：安装 Visual Studio C++ 工作负载，确认 Rust 使用 MSVC 工具链，然后重新打开终端。
- **缺少浏览器引擎**：运行 `Prepare`；不要把 macOS/Linux 引擎放进 Windows 目录。
- **浏览器仍在运行**：保存任务并关闭专用浏览器窗口，再刷新。不要手动删除正在使用的资料目录。
- **时区/语言仍不一致**：关闭专用浏览器，应用设置，再重开并实测；部分语言变化需要注销 Windows。
- **清理项目未列出**：检查路径与权限；未知、外部或无法核验的路径会被跳过，请查看扫描警告。
- **网络实测失败**：检查当前 TUN/代理和网络连通性。PAC 自动配置不被当作已验证的静态代理。

反馈问题请提供 Windows 版本、应用版本、操作步骤和脱敏错误。不要上传凭据、浏览器资料、聊天历史或含认证信息的订阅链接。
