use std::{fs,path::Path,os::windows::{ffi::OsStrExt,fs::{MetadataExt,OpenOptionsExt},io::AsRawHandle}};
use windows_sys::Win32::{Foundation::{CloseHandle,LocalFree},
    Security::{Authorization::{GetNamedSecurityInfoW,SetEntriesInAclW,SetNamedSecurityInfoW,EXPLICIT_ACCESS_W,NO_MULTIPLE_TRUSTEE,SET_ACCESS,SE_FILE_OBJECT,TRUSTEE_IS_SID,TRUSTEE_IS_USER,TRUSTEE_W},
    EqualSid,GetTokenInformation,TokenUser,DACL_SECURITY_INFORMATION,NO_INHERITANCE,OWNER_SECURITY_INFORMATION,PROTECTED_DACL_SECURITY_INFORMATION,TOKEN_QUERY,TOKEN_USER},
    Storage::FileSystem::{GetFileInformationByHandle,MoveFileExW,BY_HANDLE_FILE_INFORMATION,FILE_ALL_ACCESS,FILE_FLAG_BACKUP_SEMANTICS,FILE_FLAG_OPEN_REPARSE_POINT,FILE_READ_ATTRIBUTES,MOVEFILE_REPLACE_EXISTING,MOVEFILE_WRITE_THROUGH},
    System::Threading::{GetCurrentProcess,OpenProcessToken}};
fn wide(path:&Path)->Vec<u16>{path.as_os_str().encode_wide().chain(Some(0)).collect()}
fn current_user()->Option<Vec<usize>>{unsafe{
    let mut token=std::ptr::null_mut();if OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&mut token)==0{return None;}
    let mut length=0;GetTokenInformation(token,TokenUser,std::ptr::null_mut(),0,&mut length);
    let mut data=vec![0usize;(length as usize+std::mem::size_of::<usize>()-1)/std::mem::size_of::<usize>()];
    let ok=GetTokenInformation(token,TokenUser,data.as_mut_ptr().cast(),length,&mut length);CloseHandle(token);
    if ok==0{None}else{Some(data)}
}}
pub fn owned(path:&Path)->bool{
    let Some(data)=current_user() else{return false;};unsafe{
    let mut owner=std::ptr::null_mut();let mut descriptor=std::ptr::null_mut();
    if GetNamedSecurityInfoW(wide(path).as_ptr(),SE_FILE_OBJECT,OWNER_SECURITY_INFORMATION,&mut owner,std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null_mut(),&mut descriptor)!=0{return false;}
    let user=&*(data.as_ptr() as *const TOKEN_USER);let result=EqualSid(owner,user.User.Sid)!=0;LocalFree(descriptor);result
}}
pub fn fingerprint(path:&Path,meta:&fs::Metadata)->Result<Vec<u64>,String>{
    let file=fs::OpenOptions::new().access_mode(FILE_READ_ATTRIBUTES).custom_flags(FILE_FLAG_BACKUP_SEMANTICS|FILE_FLAG_OPEN_REPARSE_POINT).open(path).map_err(|_|"无法核验文件身份。")?;
    let mut info:BY_HANDLE_FILE_INFORMATION=unsafe{std::mem::zeroed()};
    if unsafe{GetFileInformationByHandle(file.as_raw_handle(),&mut info)}==0{return Err("无法核验文件身份。".into());}
    Ok(vec![info.dwVolumeSerialNumber as u64,((info.nFileIndexHigh as u64)<<32)|info.nFileIndexLow as u64,meta.creation_time(),meta.last_write_time(),meta.file_size(),meta.file_attributes() as u64])
}
pub fn replace(from:&Path,to:&Path)->std::io::Result<()>{
    if unsafe{MoveFileExW(wide(from).as_ptr(),wide(to).as_ptr(),MOVEFILE_REPLACE_EXISTING|MOVEFILE_WRITE_THROUGH)}==0{Err(std::io::Error::last_os_error())}else{Ok(())}
}
pub fn restrict(path:&Path)->Result<(),String>{
    let data=current_user().ok_or("无法读取当前 Windows 用户权限。")?;unsafe{
    let user=&*(data.as_ptr() as *const TOKEN_USER);
    let entry=EXPLICIT_ACCESS_W{grfAccessPermissions:FILE_ALL_ACCESS,grfAccessMode:SET_ACCESS,grfInheritance:NO_INHERITANCE,Trustee:TRUSTEE_W{
        pMultipleTrustee:std::ptr::null_mut(),MultipleTrusteeOperation:NO_MULTIPLE_TRUSTEE,TrusteeForm:TRUSTEE_IS_SID,TrusteeType:TRUSTEE_IS_USER,ptstrName:user.User.Sid.cast()}};
    let mut acl=std::ptr::null_mut();
    if SetEntriesInAclW(1,&entry,std::ptr::null(),&mut acl)!=0{return Err("无法创建私有文件权限。".into());}
    let result=SetNamedSecurityInfoW(wide(path).as_ptr(),SE_FILE_OBJECT,OWNER_SECURITY_INFORMATION|DACL_SECURITY_INFORMATION|PROTECTED_DACL_SECURITY_INFORMATION,user.User.Sid,std::ptr::null_mut(),acl,std::ptr::null());
    LocalFree(acl.cast());if result!=0{return Err("无法限制私有文件权限。".into());}Ok(())
}}
