# Windows setup and build

[简体中文](README.windows.md) | **English** | [Overview](README.en.md)

For **Windows 10/11 x64**. This independent community port remains experimental. Automated builds and disposable-fixture tests do not replace installation, startup, permission and connectivity checks on a real desktop.

## Get an installer

Open this repository's **Actions → Windows**, select a successful green run, and download `CCBaga-Windows-x64` under **Artifacts**. The ZIP contains an NSIS `.exe` installer and `SHA256SUMS`. Actions artifact downloads require a GitHub login; artifacts are retained for 30 days. If there is no successful run or its artifact has expired, build from source below.

The installer is per-user and includes the dedicated Chromium engine. If Microsoft Edge WebView2 Runtime is missing, it downloads Microsoft's bootstrapper, so initial installation may require internet access. Code signing is not configured yet. Do not disable Windows security protections to work around installation problems.

## Build from source

Install these tools and make sure they are available in your terminal PATH:

- Git, Node.js with npm, and Bun 1.3.5.
- Stable Rust using the `x86_64-pc-windows-msvc` toolchain.
- Visual Studio 2022 Build Tools with **Desktop development with C++**, including MSVC and the Windows SDK.
- Python 3.11 or later; Windows PowerShell 5.1, included with Windows.

```powershell
git clone https://github.com/UncleK/CCBaga-Windows.git
cd CCBaga-Windows
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows.ps1 -Action Check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows.ps1 -Action Prepare
```

`ExecutionPolicy Bypass` applies only to that child process; it does not change system execution policy. `Prepare` installs locked frontend dependencies, downloads and verifies the browser, then builds the frontend. The first engine download is large and requires connectivity.

For development, double-click `Start-Windows.cmd` or run `bun run windows:dev`.

To build an installer, double-click `Build-Windows.cmd` or run `bun run windows:build`. Output directory:

```text
src-tauri/target/release/bundle/nsis/
```

Run tests with:

```powershell
bun run windows:test
```

Run these scripts on Windows. The port uses `tauri.windows.conf.json`; Bash, WSL, Xcode and the macOS engine-packaging scripts are not required. Release builds require `custom-protocol`, which the scripts already enable.

## Features and limits

| Feature | Windows behavior |
| --- | --- |
| Claude Code cleanup | Scans allowlisted current-user `.claude` and `.claude.json` data; configuration edits remove only specified keys. Folder-scoped scanning is supported. |
| Claude Desktop cleanup | Scans caches, logs and separately selected site data in `%APPDATA%\Claude` inside the user profile; preserves the overall Desktop configuration directory. |
| File protection | Rejects links and junction ancestors; checks ownership, file identities and snapshots before changes. New configuration files receive a current-user private ACL and are replaced atomically. |
| Process protection | Recognizes current-user known installation paths and official npm CLI entries; rechecks user, executable path and creation time before stopping. Uncertain identities block the operation. |
| CLI uninstall/reinstall | Manages only verified official global npm installations, preserving settings, skills and projects. Use the original installer for native, WinGet, Store and custom installations. |
| Dedicated browser | Uses a separate profile, fingerprint arguments, cache controls, explicit site-data selection and a loopback measurement page. Everyday Chrome/Edge profiles are not used. |
| Timezone and region | Unicode CLDR maps Windows/IANA timezones. Preferred language and regional formats can change while retaining other languages. Display language packs may require separate installation. |
| Location | Opens Windows Settings for a manual change, then reads state. Unreadable state stays unknown and is never reported as disabled. |
| Default browser | Opens Windows Default apps settings; this version does not register as a default browser. |
| Credential Manager | Opens the system control panel; does not automatically delete Windows credentials. |
| Updates | Install a new build from this repository or rebuild from source. Upstream macOS automatic updates are disabled. |
| Network Skill | Retains the separate upstream Skill entry point; its Windows host workflow is not validated. The app does not run SSH or collect keys. |

System policies may prevent timezone, language or regional changes. Run as a regular user; if permissions are insufficient, prefer a manual change in Windows Settings. Running cleanup as administrator is not recommended.

**Cleanup permanently deletes selected data without the Recycle Bin, backup or undo.** The default cache preset excludes login, history and configuration items. Custom `CLAUDE_CONFIG_DIR`, Microsoft Store sandboxes, Windows Credential Manager and AppData redirected outside the user profile are not automatically scanned.

## Browser source

The engine is the Fingerprint Chromium **148.0.7778.215** Windows x64 ZIP:

```text
SHA-256: 9ef3f471b7a6641b4224532522b29141ce3746e27d55788d88e2fd951f362579
```

The acquisition script verifies the digest before extraction, checks ZIP paths, PE architecture and required resources. Engine binaries are not stored in Git. For offline preparation, download the exact archive from the upstream URL in the script, then run:

```powershell
python scripts/fetch-browser-windows.py --archive C:\Downloads\browser.zip
```

The same digest check still applies. The prepared engine lives in `browser-runtime/windows/`. To prepare it again, move that directory aside before running `Prepare`.

## Troubleshooting

- **Missing compiler/linker:** install the Visual Studio C++ workload, verify the Rust MSVC toolchain, and reopen the terminal.
- **Missing engine:** run `Prepare`; do not place macOS/Linux binaries in the Windows engine directory.
- **Browser still running:** save your work, close the dedicated browser windows and refresh. Do not manually remove an active profile.
- **Timezone/language mismatch:** close the dedicated browser, apply settings, reopen it and measure again. Some language changes require signing out of Windows.
- **Cleanup item missing:** check its location and permissions. Unknown, external or unverifiable paths are skipped; inspect scan warnings.
- **Network measurement failure:** check current TUN/proxy settings and connectivity. PAC auto-configuration is not treated as a verified static proxy.

For reports, include Windows version, app version, reproduction steps and sanitized errors. Never upload credentials, browser profiles, conversation history or authenticated subscription URLs.
