// Windows implementations of the existing JSON bridge; no Objective-C linking.
pub mod files;
pub mod powershell;
pub mod process;
pub mod system;
use serde_json::{json, Value};
use std::{
    ffi::{c_char, CStr, CString},
    path::Path,
};
fn response(result: Result<Value, String>) -> *mut c_char {
    CString::new(
        result
            .unwrap_or_else(|error| json!({"error":error}))
            .to_string(),
    )
    .expect("JSON escapes NUL")
    .into_raw()
}
unsafe fn input<'a>(p: *const c_char) -> &'a str {
    CStr::from_ptr(p).to_str().unwrap_or("")
}
pub unsafe fn rf_free(p: *mut c_char) {
    drop(CString::from_raw(p));
}
pub unsafe fn rf_system_read() -> *mut c_char {
    response(system::public_read())
}
pub unsafe fn rf_browser_machine() -> *mut c_char {
    response(system::read())
}
pub unsafe fn rf_system_timezone(p: *const c_char) -> *mut c_char {
    response(system::timezone(input(p)))
}
pub unsafe fn rf_system_language(p: *const c_char) -> *mut c_char {
    response(system::region(input(p), None))
}
pub unsafe fn rf_system_region(language: *const c_char, locale: *const c_char) -> *mut c_char {
    response(system::region(input(language), Some(input(locale))))
}
pub unsafe fn rf_system_open(p: *const c_char) -> *mut c_char {
    response(system::open_settings(input(p)))
}
pub unsafe fn rf_system_location() -> *mut c_char {
    response(system::location())
}
pub unsafe fn rf_browser_running(p: *const c_char) -> *mut c_char {
    response(process::browser(Path::new(input(p))).map(|p| json!({"running":!p.is_empty()})))
}
pub unsafe fn rf_browser_quit(p: *const c_char) -> *mut c_char {
    response(process::quit_browser(Path::new(input(p))).map(|_| json!({"ok":true})))
}
pub unsafe fn rf_credentials_list() -> *mut c_char {
    response(Err(
        "Windows 凭据管理器、Microsoft Store 沙箱和自定义安装目录未纳入自动清理。".into(),
    ))
}
pub unsafe fn rf_credentials_remove(_p: *const c_char) -> *mut c_char {
    response(Err("请在 Windows 凭据管理器中手动处理凭据。".into()))
}
pub unsafe fn rf_keychain_unlock() -> *mut c_char {
    response((|| {
        let _:bool=powershell::run("Start-Process (Join-Path $env:SystemRoot 'System32/control.exe') -ArgumentList '/name Microsoft.CredentialManager'; $true | ConvertTo-Json",&(),20)?;
        Ok(json!({"message":"已打开 Windows 凭据管理器；本工具不自动删除凭据。"}))
    })())
}
pub unsafe fn rf_copy(p: *const c_char) -> Result<(), String> {
    let _:bool=powershell::run("$v=[Console]::In.ReadToEnd() | ConvertFrom-Json; Set-Clipboard -Value $v; $true | ConvertTo-Json",&input(p),20)?;
    Ok(())
}
