//! Foreground-window watcher. Emits a `foreground:changed` Tauri event with
//! the lowercased executable filename (e.g. `lol.exe`, `spotify.exe`) every
//! time the user's focused application changes. The frontend matches that
//! against each profile's `app_matches` list to auto-apply per-app profiles.

#![cfg(target_os = "windows")]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub fn install(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        let mut last = String::new();
        loop {
            if let Some(name) = current_process_name() {
                if name != last {
                    last = name.clone();
                    let _ = app.emit("foreground:changed", name);
                }
            }
            thread::sleep(Duration::from_millis(750));
        }
    });
}

fn current_process_name() -> Option<String> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }
        let mut pid: u32 = 0;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let res = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        res.ok()?;

        let slice = &buf[..size as usize];
        let path: PathBuf = OsString::from_wide(slice).into();
        path.file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
    }
}
