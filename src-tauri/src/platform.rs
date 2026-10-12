// Shared file boundaries. Windows junctions are links, not ordinary directories.
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::{
    fs,
    path::{Path, PathBuf},
};
pub fn home() -> Result<PathBuf, String> {
    fs::canonicalize(
        std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
            .ok_or("无法定位用户目录。")?,
    )
    .map_err(|_| "无法读取用户目录。".into())
}
pub fn is_link(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(unix)]
    {
        meta.file_type().is_symlink()
    }
}
pub fn real_path(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    let mut current = PathBuf::new();
    for part in path.components() {
        if matches!(part, std::path::Component::ParentDir) {
            return false;
        }
        current.push(part);
        if matches!(part, std::path::Component::Prefix(_)) {
            continue;
        }
        match fs::symlink_metadata(&current) {
            Ok(m) if is_link(&m) => return false,
            Err(_) => return false,
            _ => {}
        }
    }
    fs::canonicalize(path).is_ok()
}
pub fn owned(path: &Path, meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        let _ = meta;
        crate::windows::files::owned(path)
    }
    #[cfg(unix)]
    {
        let _ = path;
        extern "C" {
            fn getuid() -> u32;
        }
        meta.uid() == unsafe { getuid() }
    }
}
pub fn fingerprint(path: &Path, meta: &fs::Metadata) -> Result<Vec<u64>, String> {
    #[cfg(windows)]
    {
        crate::windows::files::fingerprint(path, meta)
    }
    #[cfg(unix)]
    {
        let _ = path;
        Ok(vec![
            meta.dev(),
            meta.ino(),
            meta.mtime() as u64,
            meta.mtime_nsec() as u64,
            meta.ctime() as u64,
            meta.ctime_nsec() as u64,
            meta.len(),
        ])
    }
}
pub fn mode(meta: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        meta.permissions().mode()
    }
    #[cfg(windows)]
    {
        let _ = meta;
        0o600
    }
}
pub fn private_file(path: &Path, mode: u32) -> Result<fs::File, String> {
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(mode & 0o777);
    let file = options.open(path).map_err(|_| "无法创建私有文件。")?;
    #[cfg(windows)]
    {
        let _ = mode;
        if let Err(e) = crate::windows::files::restrict(path) {
            drop(file);
            let _ = fs::remove_file(path);
            return Err(e);
        }
    }
    Ok(file)
}
pub fn replace(from: &Path, to: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        crate::windows::files::replace(from, to)
    }
    #[cfg(unix)]
    {
        fs::rename(from, to)
    }
}
pub fn write(path: &Path, bytes: &[u8], mode: u32) -> Result<(), String> {
    use std::io::Write;
    let mut random = [0u8; 12];
    getrandom::fill(&mut random).map_err(|_| "无法生成临时文件名。")?;
    let name: String = random.iter().map(|x| format!("{x:02x}")).collect();
    let temp = path.with_file_name(format!(".ccbaga-{name}.tmp"));
    let result = (|| {
        let mut file = private_file(&temp, mode)?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| "无法保存完整配置。")?;
        drop(file);
        replace(&temp, path).map_err(|_| "无法替换配置。".to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
