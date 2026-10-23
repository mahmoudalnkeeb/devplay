pub mod autostart;
pub mod disk;
pub mod memory;
pub mod process;
pub mod time;
pub mod wide;

use std::path::PathBuf;

pub const GB: f64 = 1024.0 * 1024.0 * 1024.0;

pub fn local_app_data() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_default()
}
