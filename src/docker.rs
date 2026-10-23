use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::sys::{self, process};

const DESKTOP_EXE: &str = "Docker Desktop.exe";
const PROCESSES: [&str; 3] = [
    DESKTOP_EXE,
    "com.docker.backend.exe",
    "com.docker.build.exe",
];
const POLL_INTERVAL: Duration = Duration::from_secs(5);

fn install_dir() -> PathBuf {
    sys::local_app_data().join(r"Programs\DockerDesktop")
}

fn cli() -> PathBuf {
    install_dir().join(r"resources\bin\docker.exe")
}

fn docker(args: &[&str]) -> bool {
    process::run(cli(), args)
}

pub fn is_ready() -> bool {
    cli().exists() && docker(&["info"])
}

pub fn is_desktop_running() -> bool {
    process::is_running(DESKTOP_EXE)
}

pub fn stop() {
    for exe in PROCESSES {
        if process::is_running(exe) {
            process::kill(exe);
        }
    }
}

pub fn start() -> io::Result<()> {
    if !is_desktop_running() {
        Command::new(install_dir().join(DESKTOP_EXE)).spawn()?;
    }
    Ok(())
}

pub fn wait_until_ready(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if is_ready() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// Never touches containers or volumes. The final fstrim lets a sparse
/// VHDX hand the freed space back to Windows.
pub fn prune_and_trim() {
    docker(&["image", "prune", "-f"]);
    docker(&["builder", "prune", "-f", "--filter", "until=168h"]);
    docker(&[
        "run",
        "--rm",
        "--privileged",
        "--pid=host",
        "alpine",
        "nsenter",
        "-t",
        "1",
        "-m",
        "--",
        "fstrim",
        "-a",
    ]);
}
