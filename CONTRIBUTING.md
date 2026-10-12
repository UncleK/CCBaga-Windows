# Contributing to CCBaga Windows

[简体中文](CONTRIBUTING.zh-CN.md) | **English**

Follow the [Windows build guide](README.windows.en.md). Use the existing lockfiles and UI components. This repository is an independent community Windows fork; upstream macOS/Linux development notes remain in the module documentation and [Linux guide](docs/linux.md).

## Pull requests

- Describe the problem, resulting behavior, checks performed and remaining limits.
- Read the relevant `CLAUDE.md` map. Update file contracts and module maps with your changes.
- Keep native operations in Rust/platform bridges and UI strings in both YAML catalogs. Keep Chinese and English user guides in sync.
- Preserve cleanup scopes, current-user process checks, file identity/ACL checks and secret handling. Add a focused regression when these boundaries change.
- Run `bun run windows:test` on Windows and build with `bun run windows:build` for native or packaging changes. CI installs the resulting package and checks startup on a disposable Windows runner. Distinguish this from manual Windows 10/11 acceptance.
- Verify local documentation links when editing guides. Preserve third-party notices; original contributions use the [MIT License](LICENSE).

Windows builds are published by this fork's maintainer. Official upstream releases remain under **liseami/CCBaga** and are separate from this community port.

## Bug reports

Include Windows version, app version, reproduction steps, expected behavior and sanitized errors. Never attach private keys, tokens, passwords, authenticated subscription URLs or personal file contents. Send sensitive security details privately to the repository owner.
