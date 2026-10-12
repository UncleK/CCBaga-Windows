// Disposable fixtures only, never the real user's Claude data.
use super::*;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "ccbaga-测试-{}",
            crate::browser::server::nonce().unwrap()
        ));
        fs::create_dir(&p).unwrap();
        crate::windows::files::restrict(&p).unwrap();
        Self(fs::canonicalize(p).unwrap())
    }
    fn file(&self, name: &str, text: &str) -> PathBuf {
        let p = self.0.join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, text).unwrap();
        // CI may use an elevated token whose default owner is Administrators.
        // Fixtures explicitly model ordinary current-user data ownership.
        for path in p.ancestors().take_while(|path| path.starts_with(&self.0)) {
            crate::windows::files::restrict(path).unwrap();
        }
        p
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn cleanup_preserves_assets() {
    let f = Fixture::new();
    f.file(".claude/cache/item", "cache");
    f.file(".claude/skills/custom/SKILL.md", "keep");
    f.file("project/source.rs", "keep");
    f.file("AppData/Roaming/Claude/Cache/item", "cache");
    f.file(
        "AppData/Roaming/Claude/claude_desktop_config.json",
        r#"{"mcpServers":{"keep":{}}}"#,
    );
    f.file("AppData/Roaming/Claude/Network/Cookies", "keep");
    let scan = collect(&f.0, None).unwrap();
    let ids = scan
        .items
        .iter()
        .filter(|i| i.risk == "safe")
        .map(|i| i.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    let items = selected(&scan, &ids).unwrap();
    let result = transaction::perform_items(
        &f.0,
        &items,
        &Cleaner::default(),
        || Ok(false),
        &Reporter::channel(None),
    )
    .unwrap();
    assert!(result.completed);
    assert_eq!(result.cleaned, 2);
    for p in [
        ".claude/skills/custom/SKILL.md",
        "project/source.rs",
        "AppData/Roaming/Claude/claude_desktop_config.json",
        "AppData/Roaming/Claude/Network/Cookies",
    ] {
        assert!(f.0.join(p).is_file());
    }
}
#[test]
fn stale_snapshot_blocks_delete() {
    let f = Fixture::new();
    let p = f.file(".claude/cache/item", "original");
    let scan = collect(&f.0, None).unwrap();
    assert_eq!(scan.items.len(), 1);
    fs::write(p, "changed-content").unwrap();
    assert!(identity(&scan.items[0], &f.0).is_err());
    assert!(selected(&scan, &["stale-id".into()]).is_err());
}
#[test]
fn junction_not_followed() {
    let f = Fixture::new();
    let outside = Fixture::new();
    outside.file("keep", "keep");
    fs::create_dir(f.0.join(".claude")).unwrap();
    crate::windows::files::restrict(&f.0.join(".claude")).unwrap();
    let link = f.0.join(".claude/cache");
    // mklink's parser does not accept the Win32 verbatim prefix.
    let normal = |p: &Path| p.to_string_lossy().trim_start_matches(r"\\?\").to_string();
    let out = std::process::Command::new(crate::windows::powershell::system_binary("cmd.exe"))
        .args(["/D", "/C", "mklink", "/J"])
        .arg(normal(&link))
        .arg(normal(&outside.0))
        .output()
        .unwrap();
    assert!(out.status.success(), "junction creation failed");
    assert!(collect(&f.0, None).unwrap().items.is_empty());
    assert!(outside.0.join("keep").is_file());
    fs::remove_dir(link).unwrap();
}
#[test]
fn atomic_replacement_uses_current_user_acl() {
    let f = Fixture::new();
    let p = f.file("config.json", r#"{"old":true}"#);
    file_io::atomic_write(&p, br#"{"new":true}"#, 0o600).unwrap();
    assert_eq!(fs::read_to_string(&p).unwrap(), r#"{"new":true}"#);
    assert!(crate::platform::owned(&p, &fs::metadata(&p).unwrap()));
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 1);
}
