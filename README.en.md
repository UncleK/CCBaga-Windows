# CCBaga Windows

[简体中文](README.md) | **English**

A native Windows 10/11 x64 port of [liseami/CCBaga](https://github.com/liseami/CCBaga), built with Rust, Tauri and React, without WSL.

> **Status: experimental Windows preview.** Windows Server 2022 CI passed 39 tests, NSIS packaging and installation/startup checks. Manual Windows 10/11 desktop acceptance remains incomplete. The upstream macOS/Linux packages are not Windows releases.

## Download and install

[Download Windows x64 preview v0.3.0-preview.1](https://github.com/UncleK/CCBaga-Windows/releases/tag/v0.3.0-windows-preview.1). Under **Assets**, download `CCBaga-Windows-0.3.0-preview.1-x64-setup.exe` and `SHA256SUMS`, verify the checksum and run the installer. No development tools are needed to install it. See the [Windows guide](README.windows.en.md) for installation, verification and usage.

## Port scope

- Current-user Claude Code / Desktop data scanning with preview and explicit selection before cleanup.
- Windows paths, stable file identities, private permissions, junction checks and process protection.
- A pinned fingerprint browser, dedicated profile, cache controls and actual environment measurements.
- Windows timezone, language and regional formats; manual location changes in Settings followed by verification.
- English/Chinese interface, PowerShell launch scripts, NSIS packaging and Windows CI.

Credential Manager, Microsoft Store sandbox data, native/WinGet CLI maintenance, default-browser registration and automatic updates are not handled automatically. Windows host compatibility of the separate upstream Network Skill has not been validated.

## Build and validation

Source builds require Windows 10/11 x64, Visual Studio 2022 C++ Build Tools, Rust MSVC, Node.js, Bun and Python 3.11+. See [Windows setup and build](README.windows.en.md) ([简体中文](README.windows.md)).

```powershell
git clone https://github.com/UncleK/CCBaga-Windows.git
cd CCBaga-Windows
.\Start-Windows.cmd
```

Run `Build-Windows.cmd` to create an installer. Automated builds and installer artifacts are available in [Windows Actions](https://github.com/UncleK/CCBaga-Windows/actions/workflows/windows.yml).

[Preview validation record](https://github.com/UncleK/CCBaga-Windows/actions/runs/38211799505): 16 frontend tests, 3 acquisition/packaging safety tests, 20 native tests, plus installation, engine integrity, host startup and companion browser startup checks. CI does not replace manual Windows 10/11 desktop, system permission and real network acceptance.

## Data boundaries

**Confirmed cleanup deletes directly, bypassing the Recycle Bin, with no automatic backup or undo.** Project source, skills, plugins and MCP configuration should be preserved. The companion browser does not use everyday Chrome/Edge profiles.

Environment checks make real requests to `ipwho.is`, with `ipapi.co` as a fallback on the same route. Passing means only that measured checks match; it does not guarantee anonymity or account acceptance. DNS/WebRTC leak checks are not independently verified.

## Contributing

See the [contribution guide](CONTRIBUTING.md) for builds and bug reports.

## Source and license

Upstream history and the [MIT LICENSE](LICENSE) are preserved; Windows changes also use MIT. Third-party components, browser binaries and assets retain their own licenses: see [NOTICE.md](NOTICE.md). Timezone mappings use Unicode CLDR: see [LICENSE.unicode](docs/LICENSE.unicode).

This is an independent community Windows port, not an official upstream Windows release, and is not affiliated with Anthropic, Microsoft or Apple.
