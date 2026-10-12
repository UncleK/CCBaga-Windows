// Only current-user processes; every stop rechecks executable and creation time.
use serde::{Deserialize, Serialize};
use std::{os::windows::ffi::OsStrExt, path::Path};
use windows_sys::Win32::{Foundation::LocalFree, UI::Shell::CommandLineToArgvW};
#[derive(Clone, Deserialize, Serialize)]
pub struct Process {
    pub pid: i32,
    pub executable: String,
    pub command_line: String,
    pub started: String,
}
pub fn list() -> Result<Vec<Process>, String> {
    super::powershell::run(
        r#"
$sid=[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$session=(Get-Process -Id $PID).SessionId
$rows=@(Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(claude(-code|-code-vm)?|node|bun|chrome|chromium)\.exe$' } | ForEach-Object {
    $p=$_
    $owner=Invoke-CimMethod -InputObject $p -MethodName GetOwnerSid -ErrorAction Stop
    if ($owner.ReturnValue -ne 0 -and $p.SessionId -eq $session) { throw 'Cannot inspect a process in this session' }
    if ($owner.Sid -eq $sid) {
        if (!$p.ExecutablePath -or !$p.CreationDate) { throw 'Process identity unavailable' }
        @{pid=[int]$p.ProcessId; executable=$p.ExecutablePath; command_line=[string]$p.CommandLine; started=$p.CreationDate.ToUniversalTime().Ticks.ToString()}
    }
})
ConvertTo-Json -InputObject $rows -Compress
"#,
        &(),
        30,
    )
}
pub fn arguments(line: &str) -> Vec<String> {
    let wide: Vec<u16> = std::ffi::OsStr::new(line)
        .encode_wide()
        .chain(Some(0))
        .collect();
    unsafe {
        let mut count = 0;
        let argv = CommandLineToArgvW(wide.as_ptr(), &mut count);
        if argv.is_null() {
            return Vec::new();
        }
        let result = std::slice::from_raw_parts(argv, count as usize)
            .iter()
            .map(|ptr| {
                let mut n = 0;
                while *(*ptr).add(n) != 0 {
                    n += 1;
                }
                String::from_utf16_lossy(std::slice::from_raw_parts(*ptr, n))
            })
            .collect();
        LocalFree(argv.cast());
        result
    }
}
fn normalize(path: &str) -> String {
    path.trim_start_matches(r"\\?\")
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}
pub fn same_path(a: &str, b: &str) -> bool {
    normalize(a) == normalize(b)
}
pub fn claude_kind(p: &Process) -> Option<&'static str> {
    let exe = normalize(&p.executable);
    let home = normalize(&std::env::var("USERPROFILE").ok()?);
    if exe == format!("{home}\\.local\\bin\\claude.exe")
        || (exe.starts_with(&format!("{home}\\.local\\share\\claude\\versions\\"))
            && exe.ends_with(".exe"))
    {
        return Some("Claude Code");
    }
    let local = normalize(&std::env::var("LOCALAPPDATA").ok()?);
    let desktop = [
        format!("{local}\\anthropicclaude\\"),
        format!("{local}\\programs\\claude\\"),
    ];
    let store = std::env::var("ProgramFiles")
        .ok()
        .map(|p| format!("{}\\windowsapps\\claude_", normalize(&p)));
    if exe.ends_with("\\claude.exe")
        && (desktop.iter().any(|p| exe.starts_with(p))
            || store.is_some_and(|p| exe.starts_with(&p)))
    {
        return Some("Claude Desktop");
    }
    if exe.ends_with("\\node.exe") || exe.ends_with("\\bun.exe") {
        if let Some(script) = arguments(&p.command_line).get(1) {
            if normalize(script).ends_with("\\@anthropic-ai\\claude-code\\cli.js") {
                let package = Path::new(script).parent()?.join("package.json");
                if std::fs::read(package)
                    .ok()
                    .and_then(|v| serde_json::from_slice::<serde_json::Value>(&v).ok())
                    .is_some_and(|v| v["name"] == "@anthropic-ai/claude-code")
                {
                    return Some("Claude Code");
                }
            }
        }
    }
    None
}
pub fn browser(profile: &Path) -> Result<Vec<Process>, String> {
    let expected = profile.to_string_lossy();
    Ok(list()?
        .into_iter()
        .filter(|p| {
            let exe = normalize(&p.executable);
            if !exe.ends_with("\\chrome.exe") && !exe.ends_with("\\chromium.exe") {
                return false;
            }
            let args = arguments(&p.command_line);
            args.iter().enumerate().any(|(i, a)| {
                a.strip_prefix("--user-data-dir=")
                    .is_some_and(|v| same_path(v, &expected))
                    || (a == "--user-data-dir"
                        && args.get(i + 1).is_some_and(|v| same_path(v, &expected)))
            })
        })
        .collect())
}
pub fn stop(target: &Process, force: bool) -> Result<(), String> {
    let _: bool = super::powershell::run(
        r#"
$data=[Console]::In.ReadToEnd() | ConvertFrom-Json
$p=Get-CimInstance Win32_Process -Filter ('ProcessId=' + [int]$data.target.pid)
if ($null -ne $p) {
    $owner=Invoke-CimMethod -InputObject $p -MethodName GetOwnerSid
    $sid=[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    if ($owner.Sid -ne $sid -or $p.ExecutablePath -ne $data.target.executable -or $p.CreationDate.ToUniversalTime().Ticks.ToString() -ne $data.target.started) { throw 'Process identity changed' }
    $process=Get-Process -Id $p.ProcessId -ErrorAction Stop
    if ($data.force) { $process.Kill() } else { $null=$process.CloseMainWindow() }
}
$true | ConvertTo-Json
"#,
        &serde_json::json!({"target":target,"force":force}),
        30,
    )?;
    Ok(())
}
pub fn quit_browser(profile: &Path) -> Result<(), String> {
    for p in browser(profile)? {
        stop(&p, false)?;
    }
    for _ in 0..5 {
        if browser(profile)?.is_empty() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    Err("浏览器仍在运行，请保存任务并关闭 CCBaga 浏览器窗口后重试。".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn argument_boundaries() {
        let args = arguments(
            r#""C:\Program Files\Browser\chrome.exe" "--user-data-dir=C:\用户 空间\profile" --profile-directory=Default"#,
        );
        assert_eq!(args[1], r"--user-data-dir=C:\用户 空间\profile");
        assert!(same_path(r"\\?\C:\Users\Me", r"c:\users\me"));
        assert!(!same_path(r"C:\profile", r"C:\profile-other"));
    }
}
