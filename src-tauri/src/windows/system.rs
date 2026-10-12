use super::powershell;
use serde_json::{json, Value};
pub fn zones() -> &'static Value {
    static MAP: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        serde_json::from_str(include_str!("timezones.json")).expect("embedded CLDR zones")
    })
}
pub fn same_zone(a: &str, b: &str) -> bool {
    a == b || (zones()["iana"][a].is_string() && zones()["iana"][a] == zones()["iana"][b])
}
pub fn same_language(a: &str, b: &str) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    fn normalized(s: &str) -> String {
        match s {
            "zh-CN" | "zh-SG" => "zh-Hans".into(),
            "zh-TW" | "zh-HK" | "zh-MO" => "zh-Hant".into(),
            _ => s.replace('_', "-"),
        }
    }
    let a = normalized(a);
    let b = normalized(b);
    a == b || a.starts_with(&(b.clone() + "-")) || b.starts_with(&(a + "-"))
}
pub fn read() -> Result<Value, String> {
    let mut v: Value = powershell::run(
        r#"
$location=$null
$key='HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\location'
if (Test-Path $key) { $c=Get-ItemProperty $key; if ($c.Value -eq 'Deny') {$location=$false} elseif ($c.Value -eq 'Allow') {$location=$true} }
$proxy=''
$p=Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings'
if ($p.AutoConfigURL -or $p.AutoDetect) { $proxy='pac:unsupported' }
elseif ($p.ProxyEnable -and $p.ProxyServer) {
    $server=[string]$p.ProxyServer
    if ($server -match '=') {
        $parts=@{}; foreach($pair in $server.Split(';')) {$kv=$pair.Split('=',2); if($kv.Count -eq 2){$parts[$kv[0]]=$kv[1]}}
        if($parts['https']){$server=$parts['https']} elseif($parts['http']){$server=$parts['http']} elseif($parts['socks']){$server='socks5://'+$parts['socks']} else {$server='unsupported:proxy'}
    }
    if($server -notmatch '://|^unsupported:'){$server='http://'+$server}; $proxy=$server
}
$os=Get-CimInstance Win32_OperatingSystem
@{ platform='windows'; version=[string]$os.Version; cores=[Environment]::ProcessorCount; timezone=(Get-TimeZone).Id; languages=@((Get-WinUserLanguageList).LanguageTag); locale=(Get-Culture).Name.Replace('-','_'); location_enabled=$location; automatic_timezone=$null; accessibility=$true; system_proxy=$proxy } | ConvertTo-Json -Compress
"#,
        &(),
        30,
    )?;
    let id = v["timezone"].as_str().ok_or("无法读取 Windows 时区。")?;
    v["timezone"] = zones()["windows"][id]
        .as_str()
        .ok_or("此 Windows 时区尚无 IANA 映射。")?
        .into();
    v["timezones"] = json!(zones()["iana"]
        .as_object()
        .unwrap()
        .keys()
        .collect::<Vec<_>>());
    Ok(v)
}
pub fn open_settings(pane: &str) -> Result<Value, String> {
    let uri = match pane {
        "timezone" => "ms-settings:dateandtime",
        "language" => "ms-settings:regionlanguage",
        "location" => "ms-settings:privacy-location",
        "default" => "ms-settings:defaultapps",
        _ => return Err("系统设置入口无效。".into()),
    };
    open_url(uri)?;
    Ok(json!({"ok":true}))
}
pub fn open_url(url: &str) -> Result<(), String> {
    let _: bool = powershell::run(
        "$u=[Console]::In.ReadToEnd() | ConvertFrom-Json; Start-Process $u; $true | ConvertTo-Json",
        &url,
        20,
    )?;
    Ok(())
}
pub fn public_read() -> Result<Value, String> {
    let mut v = read()?;
    v.as_object_mut().unwrap().remove("system_proxy");
    Ok(v)
}
fn result(message: &str) -> Result<Value, String> {
    Ok(json!({"state":public_read()?,"message":message}))
}
pub fn timezone(zone: &str) -> Result<Value, String> {
    let id = zones()["iana"][zone].as_str().ok_or("时区无效。")?;
    let _: bool = powershell::run(
        r#"
$zone=[Console]::In.ReadToEnd() | ConvertFrom-Json
Set-TimeZone -Id $zone
if((Get-TimeZone).Id -ne $zone){throw 'Timezone verification failed'}
$true | ConvertTo-Json
"#,
        &id,
        30,
    )?;
    result("时区已保存并核验。")
}
fn language_tag(language: &str) -> Result<&str, String> {
    Ok(match language {
        "zh-Hans" => "zh-CN",
        "zh-Hant" => "zh-TW",
        "en" => "en-US",
        "fr" => "fr-FR",
        "de" => "de-DE",
        "es" => "es-ES",
        "it" => "it-IT",
        "ja" => "ja-JP",
        "ko" => "ko-KR",
        "ru" => "ru-RU",
        "ar" => "ar-SA",
        "en-US" | "en-CA" | "en-GB" | "en-SG" | "en-AU" | "pt-BR" => language,
        _ => return Err("语言无效。".into()),
    })
}
pub fn region(language: &str, locale: Option<&str>) -> Result<Value, String> {
    let tag = language_tag(language)?;
    let culture = locale
        .map(|x| x.replace('_', "-"))
        .unwrap_or_else(|| tag.into());
    if locale.is_some()
        && !crate::regions::get_regions()
            .iter()
            .any(|r| r.language == language && r.locale.replace('_', "-") == culture)
    {
        return Err("区域无效。".into());
    }
    let _: bool = powershell::run(
        r#"
$data=[Console]::In.ReadToEnd() | ConvertFrom-Json
$list=New-WinUserLanguageList $data.language
foreach($item in Get-WinUserLanguageList){if($item.LanguageTag -ne $data.language){$list.Add($item)}}
Set-WinUserLanguageList -LanguageList $list -Force
Set-Culture -CultureInfo $data.culture
if((Get-WinUserLanguageList)[0].LanguageTag -ne $data.language){throw 'Language verification failed'}
$true | ConvertTo-Json
"#,
        &json!({"language":tag,"culture":culture}),
        60,
    )?;
    let state = public_read()?;
    if state["locale"].as_str() != Some(culture.replace('-', "_").as_str()) {
        return Err("区域设置尚未生效，请重新登录 Windows 后刷新。".into());
    }
    Ok(
        json!({"state":state,"message":"语言与区域已保存；显示语言包可能需要另行安装，部分应用需重新登录。"}),
    )
}
pub fn location() -> Result<Value, String> {
    open_settings("location")?;
    result("已打开 Windows 定位设置。请手动关闭定位服务，然后返回刷新核验。")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aliases_and_validation() {
        assert!(same_zone("America/Toronto", "America/New_York"));
        assert!(same_zone("Asia/Hong_Kong", "Asia/Shanghai"));
        assert!(same_language("zh-TW", "zh-Hant"));
        assert!(timezone("bad; command").is_err());
        assert!(region("bad", None).is_err());
    }
}
