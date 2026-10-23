use std::ptr::null_mut;

use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

use super::wide::to_wide;
use super::GB;

pub fn free_space_gb(path: &str) -> Option<f64> {
    let path = to_wide(path);
    let mut free = 0u64;
    let ok = unsafe { GetDiskFreeSpaceExW(path.as_ptr(), &mut free, null_mut(), null_mut()) };
    (ok != 0).then(|| free as f64 / GB)
}
