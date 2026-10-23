use std::mem::{size_of, zeroed};

use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};

use super::GB;

pub fn free_ram_gb() -> f64 {
    unsafe {
        let mut status: MEMORYSTATUSEX = zeroed();
        status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
        GlobalMemoryStatusEx(&mut status);
        status.ullAvailPhys as f64 / GB
    }
}

pub fn trim_working_set() {
    unsafe { SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX) };
}
