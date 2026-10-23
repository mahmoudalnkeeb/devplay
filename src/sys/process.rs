use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

use super::wide::from_wide;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Runs a console program without a window; true if it exits with code 0.
pub fn run(program: impl AsRef<OsStr>, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .is_ok_and(|status| status.success())
}

pub fn kill(exe: &str) {
    run("taskkill", &["/F", "/IM", exe]);
}

/// Like clicking X, so the app can save first.
pub fn close(exe: &str) {
    run("taskkill", &["/IM", exe]);
}

pub fn is_running(exe: &str) -> bool {
    let Some(snapshot) = Snapshot::processes() else {
        return false;
    };
    let found = snapshot
        .exe_names()
        .any(|name| name.eq_ignore_ascii_case(exe));
    found
}

struct Snapshot(HANDLE);

impl Snapshot {
    fn processes() -> Option<Self> {
        let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        (handle != INVALID_HANDLE_VALUE).then_some(Self(handle))
    }

    fn exe_names(&self) -> impl Iterator<Item = String> + '_ {
        let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut first = true;
        std::iter::from_fn(move || {
            let ok = unsafe {
                if first {
                    Process32FirstW(self.0, &mut entry)
                } else {
                    Process32NextW(self.0, &mut entry)
                }
            };
            first = false;
            (ok != 0).then(|| from_wide(&entry.szExeFile))
        })
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_own_process() {
        let exe = std::env::current_exe().unwrap();
        let name = exe.file_name().unwrap().to_str().unwrap();
        assert!(is_running(name));
        assert!(is_running(&name.to_uppercase()));
    }

    #[test]
    fn does_not_find_missing_process() {
        assert!(!is_running("definitely-not-running-devplay-test.exe"));
    }

    #[test]
    fn run_reports_exit_status() {
        assert!(run("cmd", &["/c", "exit", "0"]));
        assert!(!run("cmd", &["/c", "exit", "3"]));
        assert!(!run("no-such-program-devplay-test", &[]));
    }
}
