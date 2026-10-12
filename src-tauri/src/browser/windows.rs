// Windows resource layout. No macOS updater, LaunchServices or archive extraction.
fn engine(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let bundled = app
        .path()
        .resource_dir()
        .map_err(|_| "资源目录不可用。")?
        .join("browser-runtime/windows");
    if binary(&bundled).is_file() {
        return Ok(bundled);
    }
    if cfg!(debug_assertions) {
        return Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../browser-runtime/windows"));
    }
    Ok(bundled)
}
fn binary(engine: &Path) -> PathBuf {
    engine.join("chrome.exe")
}
fn prepare_engine(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let p = engine(app)?;
    if !binary(&p).is_file() {
        return Err(
            "Windows 浏览器引擎缺失，请先运行 scripts/windows.ps1 -Action Prepare。".into(),
        );
    }
    Ok(p)
}
fn register(_engine: &Path) -> Result<(), String> {
    Ok(())
}
fn version(engine: &Path) -> String {
    crate::windows::powershell::run::<String>("$p=[Console]::In.ReadToEnd() | ConvertFrom-Json; (Get-Item -LiteralPath $p).VersionInfo.ProductVersion | ConvertTo-Json",&binary(engine).to_string_lossy(),20).unwrap_or_default()
}
fn default_read() -> Result<DefaultBrowser, String> {
    let v: serde_json::Value = crate::windows::powershell::run(
        r#"
$root='HKCU:\Software\Microsoft\Windows\Shell\Associations\UrlAssociations'
function Read-Choice($scheme) { $path=Join-Path $root ($scheme+'\UserChoice'); if(Test-Path $path){(Get-ItemProperty $path).ProgId}else{''} }
@{http=(Read-Choice 'http');https=(Read-Choice 'https')} | ConvertTo-Json -Compress
"#,
        &(),
        20,
    )?;
    Ok(DefaultBrowser {
        is_default: false,
        http: v["http"].as_str().unwrap_or("").into(),
        https: v["https"].as_str().unwrap_or("").into(),
        installed: true,
        app: "Windows 默认应用设置（本版未注册为系统浏览器）".into(),
    })
}
#[tauri::command]
pub async fn browser_set_default(_app: tauri::AppHandle) -> Result<DefaultBrowser, String> {
    run(|| {
        crate::windows::system::open_settings("default")?;
        default_read()
    })
    .await
}
pub fn companion(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = browser_launch(app, Channel::new(|_| Ok(()))).await {
            eprintln!("CCBaga browser: {error}");
        }
    });
}
