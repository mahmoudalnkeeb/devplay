// Window procedures can't capture context, so tray state lives in statics.

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU8, Ordering};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::HICON;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Dev,
    Play,
}

impl Mode {
    pub const ALL: [Mode; 2] = [Mode::Dev, Mode::Play];

    pub fn tooltip(self) -> &'static str {
        match self {
            Mode::Dev => "DevPlay: Dev Mode",
            Mode::Play => "DevPlay: Play Mode",
        }
    }

    fn to_u8(self) -> u8 {
        match self {
            Mode::Dev => 0,
            Mode::Play => 1,
        }
    }

    fn from_u8(value: u8) -> Self {
        if value == Mode::Play.to_u8() {
            Mode::Play
        } else {
            Mode::Dev
        }
    }

    fn slot(self) -> usize {
        usize::from(self.to_u8())
    }
}

static MODE: AtomicU8 = AtomicU8::new(0);
static BUSY: AtomicBool = AtomicBool::new(false);
static WINDOW: AtomicIsize = AtomicIsize::new(0);
static ICONS: [AtomicIsize; 2] = [AtomicIsize::new(0), AtomicIsize::new(0)];
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

pub fn mode() -> Mode {
    Mode::from_u8(MODE.load(Ordering::Relaxed))
}

pub fn set_mode(mode: Mode) {
    MODE.store(mode.to_u8(), Ordering::Relaxed);
}

/// Returns false if a task is already running.
pub fn try_begin_task() -> bool {
    !BUSY.swap(true, Ordering::AcqRel)
}

pub fn end_task() {
    BUSY.store(false, Ordering::Release);
}

pub fn is_busy() -> bool {
    BUSY.load(Ordering::Acquire)
}

pub fn window() -> HWND {
    WINDOW.load(Ordering::Relaxed) as HWND
}

pub fn set_window(hwnd: HWND) {
    WINDOW.store(hwnd as isize, Ordering::Relaxed);
}

pub fn icon(mode: Mode) -> HICON {
    ICONS[mode.slot()].load(Ordering::Relaxed) as HICON
}

pub fn set_icon(mode: Mode, icon: HICON) {
    ICONS[mode.slot()].store(icon as isize, Ordering::Relaxed);
}

pub fn taskbar_created_message() -> u32 {
    TASKBAR_CREATED.load(Ordering::Relaxed)
}

pub fn set_taskbar_created_message(message: u32) {
    TASKBAR_CREATED.store(message, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_round_trips_through_u8() {
        for mode in Mode::ALL {
            assert_eq!(Mode::from_u8(mode.to_u8()), mode);
        }
    }

    #[test]
    fn modes_use_distinct_icon_slots() {
        assert_ne!(Mode::Dev.slot(), Mode::Play.slot());
    }

    #[test]
    fn busy_flag_allows_one_task_at_a_time() {
        assert!(try_begin_task());
        assert!(is_busy());
        assert!(!try_begin_task());
        end_task();
        assert!(!is_busy());
    }
}
