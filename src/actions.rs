use std::fmt;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use crate::sys::{disk, memory, process};
use crate::{config, docker, log};

const CLOSE_GRACE: Duration = Duration::from_secs(5);
const DOCKER_START_TIMEOUT: Duration = Duration::from_secs(180);
const SYSTEM_DRIVE: &str = "C:\\";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Play,
    Dev,
    Cleanup,
}

impl Action {
    /// Runs the action and returns its report, which is also logged.
    pub fn run(self) -> String {
        let report = match self {
            Action::Play => play_mode(),
            Action::Dev => dev_mode(),
            Action::Cleanup => cleanup(),
        };
        log::write(&report);
        report
    }
}

impl FromStr for Action {
    type Err = UnknownAction;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "play" => Ok(Action::Play),
            "dev" => Ok(Action::Dev),
            "cleanup" => Ok(Action::Cleanup),
            _ => Err(UnknownAction(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAction(String);

impl fmt::Display for UnknownAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown command '{}'. Use play, dev or cleanup.", self.0)
    }
}

impl std::error::Error for UnknownAction {}

fn play_mode() -> String {
    let before = memory::free_ram_gb();
    docker::stop();
    process::run("wsl", &["--shutdown"]);
    for app in config::apps_to_close() {
        if process::is_running(&app) {
            process::close(&app);
        }
    }
    // Give politely closed apps time to exit before measuring.
    thread::sleep(CLOSE_GRACE);
    play_report(before, memory::free_ram_gb())
}

fn dev_mode() -> String {
    if let Err(err) = docker::start() {
        return format!("Dev Mode: couldn't start Docker Desktop ({err}).");
    }
    dev_report(docker::wait_until_ready(DOCKER_START_TIMEOUT))
}

fn cleanup() -> String {
    let before = disk::free_space_gb(SYSTEM_DRIVE);
    let mut steps = Vec::new();

    if docker::is_ready() {
        docker::prune_and_trim();
        steps.push("Docker pruned and trimmed");
    } else {
        steps.push("Docker not running, skipped");
    }
    if process::run("cmd", &["/c", "npm", "cache", "verify"]) {
        steps.push("npm cache verified");
    }

    cleanup_report(&steps, before, disk::free_space_gb(SYSTEM_DRIVE))
}

fn play_report(free_before: f64, free_after: f64) -> String {
    format!(
        "Play Mode on. Freed {:.1} GB RAM ({free_after:.1} GB free).",
        (free_after - free_before).max(0.0)
    )
}

fn dev_report(docker_ready: bool) -> String {
    if docker_ready {
        "Dev Mode on. Docker is ready.".to_owned()
    } else {
        "Dev Mode on, but Docker is still starting. Check its tray icon.".to_owned()
    }
}

fn cleanup_report(steps: &[&str], free_before: Option<f64>, free_after: Option<f64>) -> String {
    let space = match (free_before, free_after) {
        (Some(before), Some(after)) => {
            format!(
                " C: {after:.1} GB free (+{:.2} GB).",
                (after - before).max(0.0)
            )
        }
        _ => String::new(),
    };
    format!("Cleanup done: {}.{space}", steps.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_actions_case_insensitively() {
        assert_eq!("play".parse(), Ok(Action::Play));
        assert_eq!("DEV".parse(), Ok(Action::Dev));
        assert_eq!("Cleanup".parse(), Ok(Action::Cleanup));
    }

    #[test]
    fn rejects_unknown_action_with_helpful_message() {
        let err = "plya".parse::<Action>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Unknown command 'plya'. Use play, dev or cleanup."
        );
    }

    #[test]
    fn play_report_never_shows_negative_savings() {
        assert_eq!(
            play_report(6.0, 5.5),
            "Play Mode on. Freed 0.0 GB RAM (5.5 GB free)."
        );
        assert_eq!(
            play_report(3.8, 6.4),
            "Play Mode on. Freed 2.6 GB RAM (6.4 GB free)."
        );
    }

    #[test]
    fn dev_report_covers_both_outcomes() {
        assert_eq!(dev_report(true), "Dev Mode on. Docker is ready.");
        assert!(dev_report(false).contains("still starting"));
    }

    #[test]
    fn cleanup_report_includes_space_when_known() {
        assert_eq!(
            cleanup_report(&["a", "b"], Some(50.0), Some(51.3)),
            "Cleanup done: a, b. C: 51.3 GB free (+1.30 GB)."
        );
        assert_eq!(cleanup_report(&["a"], None, Some(1.0)), "Cleanup done: a.");
    }
}
