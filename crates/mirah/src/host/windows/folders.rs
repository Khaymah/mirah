#![allow(
    unsafe_code,
    reason = "the known-folder API has no safe wrapper; the returned string is freed once read"
)]

use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::UI::Shell::{FOLDERID_Downloads, SHGetKnownFolderPath};

pub(crate) fn downloads() -> Option<PathBuf> {
    let mut raw: windows_sys::core::PWSTR = std::ptr::null_mut();
    unsafe {
        let result = SHGetKnownFolderPath(&FOLDERID_Downloads, 0, std::ptr::null_mut(), &mut raw);
        if raw.is_null() {
            return None;
        }
        let path = (result >= 0).then(|| {
            let len = (0..).take_while(|&i| *raw.add(i) != 0).count();
            let wide = std::slice::from_raw_parts(raw, len);
            PathBuf::from(std::ffi::OsString::from_wide(wide))
        });
        CoTaskMemFree(raw.cast());
        path
    }
}
