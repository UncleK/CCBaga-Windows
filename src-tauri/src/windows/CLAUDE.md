# Windows platform boundary
> Parent: ../CLAUDE.md

files.rs: Win32 owner SID, volume/file IDs, protected current-user ACL and atomic replacement; errors block mutation.
powershell.rs: fixed UTF-16 scripts, JSON stdin/stdout, bounded output and child-tree timeout; no secret-bearing command interpolation.
process.rs: current-user CIM enumeration, argument parsing and exact profile boundaries; recheck identity before stop.
system.rs / timezones.json: CLDR IANA/Windows mapping, read-only snapshots, verified language/timezone changes and manual location/default Settings; proxy details excluded from UI.
mod.rs: existing JSON bridge interface and explicit unsupported-credential boundary.
claude_cli.rs: shared cleaner transaction lock and verified npm CLI maintenance.
cleaner_tests.rs: disposable Unicode paths, stale snapshot, junction isolation and private atomic replacement; never use real Claude data.

Windows-specific changes require a native Windows build/test. Do not claim live desktop validation from cross-compilation or fixture tests.
