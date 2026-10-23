use std::fs::OpenOptions;
use std::io::Write;

use crate::config::exe_dir;
use crate::sys::time::timestamp;

const LOG_FILE: &str = "devplay.log";

pub fn write(msg: &str) {
    let path = exe_dir().join(LOG_FILE);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        // Best-effort: a failed log write must never stop the action it describes.
        let _ = writeln!(file, "{}  {msg}", timestamp());
    }
}
