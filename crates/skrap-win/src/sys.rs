//! The Win32 calls themselves.

use std::collections::HashSet;
use std::ffi::{OsString, c_void};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_HIDDEN, FILE_FLAGS_AND_ATTRIBUTES, GetDiskFreeSpaceExW, GetDriveTypeW,
    GetFileAttributesW, GetLogicalDrives, GetVolumeInformationW, INVALID_FILE_ATTRIBUTES,
    SetFileAttributesW,
};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_Music, FOLDERID_Pictures,
    FOLDERID_PublicDesktop, FOLDERID_PublicDocuments, FOLDERID_PublicMusic,
    FOLDERID_PublicPictures, FOLDERID_PublicVideos, FOLDERID_SavedGames, FOLDERID_Videos,
    KF_FLAG_DEFAULT, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND, SHEmptyRecycleBinW,
    SHGetKnownFolderPath, SHQUERYRBINFO, SHQueryRecycleBinW,
};
use windows::core::{GUID, HSTRING, PCWSTR};

use crate::Disk;

const DRIVE_FIXED: u32 = 3;

pub fn known_folders() -> Vec<PathBuf> {
    let ids: [&GUID; 12] = [
        &FOLDERID_Documents,
        &FOLDERID_Desktop,
        &FOLDERID_Pictures,
        &FOLDERID_Videos,
        &FOLDERID_Music,
        &FOLDERID_Downloads,
        &FOLDERID_SavedGames,
        &FOLDERID_PublicDocuments,
        &FOLDERID_PublicDesktop,
        &FOLDERID_PublicPictures,
        &FOLDERID_PublicVideos,
        &FOLDERID_PublicMusic,
    ];
    ids.into_iter()
        .filter_map(|id| {
            // SAFETY: a valid known folder id; the returned string is freed below.
            let path = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.ok()?;
            // SAFETY: SHGetKnownFolderPath returned a null-terminated string.
            let text = unsafe { path.to_string() };
            // SAFETY: allocated by the shell with CoTaskMemAlloc.
            unsafe { CoTaskMemFree(Some(path.0 as *const c_void)) };
            text.ok().map(PathBuf::from)
        })
        .collect()
}

pub fn running_programs() -> HashSet<String> {
    let mut names = HashSet::new();
    // SAFETY: a process snapshot, closed below.
    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
        return names;
    };
    let mut entry = PROCESSENTRY32W {
        dwSize: u32::try_from(size_of::<PROCESSENTRY32W>()).unwrap_or(0),
        ..Default::default()
    };
    // SAFETY: `entry` is a valid PROCESSENTRY32W with its size set.
    let mut more = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while more {
        let len = entry
            .szExeFile
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(entry.szExeFile.len());
        let name = OsString::from_wide(&entry.szExeFile[..len]);
        names.insert(name.to_string_lossy().to_lowercase());
        // SAFETY: as above.
        more = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    // SAFETY: the snapshot handle opened above.
    let _ = unsafe { CloseHandle(snapshot) };
    names
}

pub fn disks() -> Vec<Disk> {
    let system = std::env::var("SystemDrive")
        .unwrap_or_else(|_| "C:".into())
        .to_ascii_uppercase();
    // SAFETY: no arguments.
    let mask = unsafe { GetLogicalDrives() };
    (0..26u8)
        .filter(|i| mask & (1 << i) != 0)
        .filter_map(|i| {
            let root = format!("{}:\\", char::from(b'A' + i));
            let wide = HSTRING::from(root.as_str());
            // SAFETY: a null-terminated root path.
            if unsafe { GetDriveTypeW(&wide) } != DRIVE_FIXED {
                return None;
            }
            let (free, total) = space(&wide)?;
            let mut label = [0u16; 261];
            // SAFETY: the label buffer is writable; the other outputs are not asked for.
            let label =
                unsafe { GetVolumeInformationW(&wide, Some(&mut label), None, None, None, None) }
                    .map(|()| {
                        let len = label.iter().position(|&c| c == 0).unwrap_or(0);
                        String::from_utf16_lossy(&label[..len])
                    })
                    .unwrap_or_default();
            Some(Disk {
                system: root.starts_with(&system),
                root,
                label,
                total,
                free,
            })
        })
        .collect()
}

pub fn free_space(path: &Path) -> Option<u64> {
    space(&HSTRING::from(path.as_os_str())).map(|(free, _)| free)
}

/// (free for this user, total) of the disk holding `path`.
fn space(path: &HSTRING) -> Option<(u64, u64)> {
    let (mut free, mut total) = (0u64, 0u64);
    // SAFETY: both outputs point to live u64s.
    unsafe { GetDiskFreeSpaceExW(path, Some(&mut free), Some(&mut total), None) }.ok()?;
    Some((free, total))
}

pub fn is_elevated() -> bool {
    let mut token = HANDLE::default();
    // SAFETY: the current process's pseudo handle; the token is closed below.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }.is_err() {
        return false;
    }
    let mut elevation = TOKEN_ELEVATION::default();
    let mut len = 0u32;
    // SAFETY: the buffer is a TOKEN_ELEVATION of the size given.
    let ok = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some((&raw mut elevation).cast()),
            u32::try_from(size_of::<TOKEN_ELEVATION>()).unwrap_or(0),
            &mut len,
        )
    }
    .is_ok();
    // SAFETY: the token opened above.
    let _ = unsafe { CloseHandle(token) };
    ok && elevation.TokenIsElevated != 0
}

pub fn hide(path: &Path) {
    let wide = HSTRING::from(path.as_os_str());
    // SAFETY: a null-terminated path.
    let attributes = unsafe { GetFileAttributesW(&wide) };
    if attributes == INVALID_FILE_ATTRIBUTES {
        return;
    }
    let hidden = FILE_FLAGS_AND_ATTRIBUTES(attributes) | FILE_ATTRIBUTE_HIDDEN;
    // SAFETY: as above.
    let _ = unsafe { SetFileAttributesW(&wide, hidden) };
}

/// (bytes, items) in the Recycle Bins of every disk.
pub fn recycle_bin() -> Option<(u64, u64)> {
    let mut info = SHQUERYRBINFO {
        cbSize: u32::try_from(size_of::<SHQUERYRBINFO>()).unwrap_or(0),
        ..Default::default()
    };
    // SAFETY: `info` has its size set; a null root means every disk.
    unsafe { SHQueryRecycleBinW(PCWSTR::null(), &mut info) }.ok()?;
    Some((
        u64::try_from(info.i64Size).unwrap_or(0),
        u64::try_from(info.i64NumItems).unwrap_or(0),
    ))
}

pub fn empty_recycle_bin() -> Result<(), String> {
    // SAFETY: no window, every disk, no dialog nor sound.
    match unsafe {
        SHEmptyRecycleBinW(
            None,
            PCWSTR::null(),
            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
        )
    } {
        Ok(()) => Ok(()),
        // Already empty.
        Err(err) if err.code().0 as u32 == 0x8000_FFFF => Ok(()),
        Err(err) => Err(err.message()),
    }
}

/// A string value under HKEY_CURRENT_USER.
pub fn user_registry_string(key: &str, value: &str) -> Option<String> {
    let key = HSTRING::from(key);
    let value = HSTRING::from(value);
    let mut buffer = [0u16; 1024];
    let mut size = u32::try_from(size_of_val(&buffer)).unwrap_or(0);
    // SAFETY: the buffer and its size in bytes.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &key,
            &value,
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status.is_err() {
        return None;
    }
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..len]))
}
