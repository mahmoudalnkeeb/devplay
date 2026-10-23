use std::ptr::null;

use windows_sys::Win32::Foundation::{HWND, POINT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, PostMessageW, SetForegroundWindow,
    TrackPopupMenu, MF_CHECKED, MF_GRAYED, MF_SEPARATOR, MF_STRING, TPM_NONOTIFY, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, WM_NULL,
};

use super::state::{self, Mode};
use crate::sys::{autostart, wide::to_wide};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    PlayMode,
    DevMode,
    Cleanup,
    EditApps,
    Autostart,
    Exit,
}

/// `None` is a separator.
const LAYOUT: [Option<MenuItem>; 8] = [
    Some(MenuItem::PlayMode),
    Some(MenuItem::DevMode),
    Some(MenuItem::Cleanup),
    None,
    Some(MenuItem::EditApps),
    Some(MenuItem::Autostart),
    None,
    Some(MenuItem::Exit),
];

impl MenuItem {
    const ALL: [MenuItem; 6] = [
        MenuItem::PlayMode,
        MenuItem::DevMode,
        MenuItem::Cleanup,
        MenuItem::EditApps,
        MenuItem::Autostart,
        MenuItem::Exit,
    ];

    /// Never 0: TrackPopupMenu returns 0 when the menu is dismissed.
    fn id(self) -> usize {
        match self {
            MenuItem::PlayMode => 1,
            MenuItem::DevMode => 2,
            MenuItem::Cleanup => 3,
            MenuItem::EditApps => 4,
            MenuItem::Autostart => 5,
            MenuItem::Exit => 6,
        }
    }

    fn from_id(id: usize) -> Option<Self> {
        Self::ALL.into_iter().find(|item| item.id() == id)
    }

    fn label(self) -> &'static str {
        match self {
            MenuItem::PlayMode => "Play Mode",
            MenuItem::DevMode => "Dev Mode",
            MenuItem::Cleanup => "Run cleanup now",
            MenuItem::EditApps => "Edit apps to close...",
            MenuItem::Autostart => "Start with Windows",
            MenuItem::Exit => "Exit",
        }
    }
}

struct MenuContext {
    mode: Mode,
    busy: bool,
    autostart: bool,
}

fn item_flags(item: MenuItem, ctx: &MenuContext) -> u32 {
    let (grayed, checked) = match item {
        MenuItem::PlayMode => (ctx.busy, ctx.mode == Mode::Play),
        MenuItem::DevMode => (ctx.busy, ctx.mode == Mode::Dev),
        MenuItem::Cleanup => (ctx.busy, false),
        MenuItem::Autostart => (false, ctx.autostart),
        MenuItem::EditApps | MenuItem::Exit => (false, false),
    };
    let mut flags = MF_STRING;
    if grayed {
        flags |= MF_GRAYED;
    }
    if checked {
        flags |= MF_CHECKED;
    }
    flags
}

pub fn pick(hwnd: HWND) -> Option<MenuItem> {
    let ctx = MenuContext {
        mode: state::mode(),
        busy: state::is_busy(),
        autostart: autostart::is_enabled(),
    };

    unsafe {
        let menu = CreatePopupMenu();
        for entry in LAYOUT {
            match entry {
                Some(item) => {
                    let label = to_wide(item.label());
                    AppendMenuW(menu, item_flags(item, &ctx), item.id(), label.as_ptr());
                }
                None => {
                    AppendMenuW(menu, MF_SEPARATOR, 0, null());
                }
            }
        }

        let mut cursor = POINT { x: 0, y: 0 };
        GetCursorPos(&mut cursor);
        // Without this the menu doesn't close when the user clicks elsewhere.
        SetForegroundWindow(hwnd);
        let flags = TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON;
        let command = TrackPopupMenu(menu, flags, cursor.x, cursor.y, 0, hwnd, null());
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);

        usize::try_from(command).ok().and_then(MenuItem::from_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(mode: Mode, busy: bool, autostart: bool) -> MenuContext {
        MenuContext {
            mode,
            busy,
            autostart,
        }
    }

    #[test]
    fn ids_round_trip_and_are_unique() {
        for item in MenuItem::ALL {
            assert_ne!(item.id(), 0);
            assert_eq!(MenuItem::from_id(item.id()), Some(item));
        }
        assert_eq!(MenuItem::from_id(0), None);
    }

    #[test]
    fn layout_contains_every_item_once() {
        for item in MenuItem::ALL {
            let count = LAYOUT.iter().filter(|e| **e == Some(item)).count();
            assert_eq!(count, 1, "{item:?}");
        }
    }

    #[test]
    fn current_mode_is_checked() {
        let play = ctx(Mode::Play, false, false);
        assert_ne!(item_flags(MenuItem::PlayMode, &play) & MF_CHECKED, 0);
        assert_eq!(item_flags(MenuItem::DevMode, &play) & MF_CHECKED, 0);
    }

    #[test]
    fn busy_grays_only_actions() {
        let busy = ctx(Mode::Dev, true, false);
        for item in [MenuItem::PlayMode, MenuItem::DevMode, MenuItem::Cleanup] {
            assert_ne!(item_flags(item, &busy) & MF_GRAYED, 0, "{item:?}");
        }
        for item in [MenuItem::EditApps, MenuItem::Autostart, MenuItem::Exit] {
            assert_eq!(item_flags(item, &busy) & MF_GRAYED, 0, "{item:?}");
        }
    }

    #[test]
    fn autostart_checkmark_follows_setting() {
        assert_ne!(
            item_flags(MenuItem::Autostart, &ctx(Mode::Dev, false, true)) & MF_CHECKED,
            0
        );
        assert_eq!(
            item_flags(MenuItem::Autostart, &ctx(Mode::Dev, false, false)) & MF_CHECKED,
            0
        );
    }
}
