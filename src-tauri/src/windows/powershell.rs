// Fixed UTF-16 script text; data arrives only as JSON on stdin.
use base64::Engine;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{Read, Write},
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
pub fn system_binary(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()))
        .join("System32")
        .join(name)
}
fn drain(mut pipe: impl Read) -> Vec<u8> {
    let mut result = Vec::new();
    let mut bytes = [0; 8192];
    while let Ok(n) = pipe.read(&mut bytes) {
        if n == 0 {
            break;
        }
        result.extend_from_slice(&bytes[..n]);
        if result.len() > 1048576 {
            result.drain(..result.len() - 1048576);
        }
    }
    result
}
pub fn run<T: DeserializeOwned>(
    script: &str,
    input: &impl Serialize,
    seconds: u64,
) -> Result<T, String> {
    let input = serde_json::to_vec(input).map_err(|_| "Windows 参数编码失败。")?;
    let prefix="$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue'; [Console]::InputEncoding=New-Object System.Text.UTF8Encoding($false); [Console]::OutputEncoding=New-Object System.Text.UTF8Encoding($false); ";
    let bytes: Vec<u8> = format!("{prefix}{script}")
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let mut child = Command::new(system_binary("WindowsPowerShell/v1.0/powershell.exe"))
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-STA",
            "-EncodedCommand",
            &encoded,
        ])
        .creation_flags(0x08000000)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "无法启动 Windows PowerShell 5.1。")?;
    let mut stdin = child.stdin.take().unwrap();
    let output = child.stdout.take().unwrap();
    let errors = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || drain(output));
    let err = std::thread::spawn(move || drain(errors));
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if start.elapsed() < Duration::from_secs(seconds) => {
                std::thread::sleep(Duration::from_millis(75))
            }
            _ => {
                let _ = Command::new(system_binary("taskkill.exe"))
                    .args(["/PID", &child.id().to_string(), "/T", "/F"])
                    .creation_flags(0x08000000)
                    .output();
                let _ = child.kill();
                let _ = child.wait();
                break Err("Windows 操作超时；请刷新确认实际状态。".to_string());
            }
        }
    };
    let _ = writer.join();
    let output = out.join().map_err(|_| "Windows 输出被中断。")?;
    let _ = err.join();
    if !status?.success() {
        return Err("Windows 操作失败；请检查权限、依赖和系统设置后重试。".into());
    }
    serde_json::from_slice(&output).map_err(|_| "Windows 返回数据不完整。".into())
}
