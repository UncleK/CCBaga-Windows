use crate::{
    cleaner::Cleaner,
    progress::{Progress, Reporter},
    windows::powershell,
};
use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, Manager};
#[derive(Debug, Serialize, Deserialize)]
pub struct CliStatus {
    method: String,
    path: String,
    version: String,
}
fn execute(action: &str) -> Result<CliStatus, String> {
    powershell::run(
        include_str!("../../../scripts/claude-cli-windows.ps1"),
        &action,
        if action == "read" { 60 } else { 600 },
    )
}
#[tauri::command]
pub async fn claude_cli_read() -> Result<CliStatus, String> {
    tauri::async_runtime::spawn_blocking(|| execute("read"))
        .await
        .map_err(|_| "Claude Code 状态读取被中断。".to_string())?
}
#[tauri::command]
pub async fn claude_cli_manage(
    app: tauri::AppHandle,
    action: String,
    on_progress: Channel<Progress>,
) -> Result<CliStatus, String> {
    if !matches!(action.as_str(), "uninstall" | "reinstall") {
        return Err("无效的 Claude Code 安装操作。".into());
    }
    let state = app.state::<Cleaner>();
    state.enter()?;
    let result = tauri::async_runtime::spawn_blocking(move || {
        let log = Reporter::channel(Some(on_progress));
        let result: Result<CliStatus, String> = (|| {
            if crate::cleaner::cli_running()? {
                return Err("Claude Code 仍在运行，请先结束进程。".into());
            }
            let before = execute("read")?;
            if !before.method.is_empty() && before.method != "npm" {
                return Err(
                    "此 Windows 安装不是可验证的 npm 官方包；请使用原生安装器或 WinGet 管理。"
                        .into(),
                );
            }
            log.log(
                "cli",
                "running",
                "正在管理已识别的 npm CLI 安装；保留配置和项目。",
                0,
                None,
            );
            let after = execute(&action)?;
            log.log("cli", "ok", "CLI 安装状态已核验。", 1, Some(1));
            Ok(after)
        })();
        if let Err(e) = &result {
            log.fail(e);
        }
        result
    })
    .await
    .map_err(|_| "Claude Code 操作被中断，请刷新确认实际状态。".to_string())
    .and_then(|r| r);
    state.leave();
    result
}
