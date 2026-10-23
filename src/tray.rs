mod icons;
mod menu;
mod notify;
mod state;

use std::mem::zeroed;
use std::process::Command;
use std::ptr::{null, null_mut};
use std::thread;

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostMessageW,
    PostQuitMessage, RegisterClassW, RegisterWindowMessageW, TranslateMessage, MSG, WM_APP,
    WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSW,
};

use crate::actions::Action;
use crate::sys::{autostart, memory, wide::to_wide};
use crate::{config, docker, log};
use menu::MenuItem;
use state::Mode;

const WM_TRAY: u32 = WM_APP + 1;
/// `lparam` carries a `Box<String>` report that the receiver must free.
const WM_TASK_DONE: u32 = WM_APP + 2;

const APP_NAME: &str = "DevPlay";
const WINDOW_CLASS: &str = "DevPlayTray";
const INSTANCE_MUTEX: &str = "Local\\DevPlayTray";

pub fn run() {
    let Some(_instance) = SingleInstance::acquire() else {
        return;
    };
    let Some(hwnd) = create_window() else {
        log::write("Could not create the tray window.");
        return;
    };
    state::set_window(hwnd);
    let taskbar_created = unsafe { RegisterWindowMessageW(to_wide("TaskbarCreated").as_ptr()) };
    state::set_taskbar_created_message(taskbar_created);
    for mode in Mode::ALL {
        state::set_icon(mode, icons::load(mode));
    }

    let mode = if docker::is_desktop_running() {
        Mode::Dev
    } else {
        Mode::Play
    };
    state::set_mode(mode);

    notify::add();
    memory::trim_working_set();
    message_loop();
}

struct SingleInstance(HANDLE);

impl SingleInstance {
    fn acquire() -> Option<Self> {
        let name = to_wide(INSTANCE_MUTEX);
        unsafe {
            let handle = CreateMutexW(null(), 1, name.as_ptr());
            if handle.is_null() {
                return None;
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return None;
            }
            Some(Self(handle))
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

fn create_window() -> Option<HWND> {
    let class = to_wide(WINDOW_CLASS);
    let title = to_wide(APP_NAME);
    unsafe {
        let instance = GetModuleHandleW(null());
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null(),
            lpszClassName: class.as_ptr(),
        };
        RegisterClassW(&wc);
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            title.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        (!hwnd.is_null()).then_some(hwnd)
    }
}

fn message_loop() {
    unsafe {
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAY => {
            let event = lparam as u32;
            if event == WM_LBUTTONUP || event == WM_RBUTTONUP {
                if let Some(item) = menu::pick(hwnd) {
                    on_menu(hwnd, item);
                }
                memory::trim_working_set();
            }
            0
        }
        WM_TASK_DONE => {
            // SAFETY: lparam is the pointer leaked by `start_task`, received exactly once.
            let report = Box::from_raw(lparam as *mut String);
            notify::show_message(&report);
            memory::trim_working_set();
            0
        }
        WM_DESTROY => {
            notify::remove();
            PostQuitMessage(0);
            0
        }
        // Explorer restarted and dropped our icon.
        m if m != 0 && m == state::taskbar_created_message() => {
            notify::add();
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn on_menu(hwnd: HWND, item: MenuItem) {
    match item {
        MenuItem::PlayMode => start_task(hwnd, Action::Play),
        MenuItem::DevMode => start_task(hwnd, Action::Dev),
        MenuItem::Cleanup => start_task(hwnd, Action::Cleanup),
        MenuItem::EditApps => open_apps_file(),
        MenuItem::Autostart => {
            let enable = !autostart::is_enabled();
            if !autostart::set_enabled(enable) {
                log::write("Could not change the Start with Windows setting.");
            }
        }
        MenuItem::Exit => {
            unsafe { DestroyWindow(hwnd) };
        }
    }
}

fn mode_after(action: Action) -> Option<Mode> {
    match action {
        Action::Play => Some(Mode::Play),
        Action::Dev => Some(Mode::Dev),
        Action::Cleanup => None,
    }
}

fn start_task(hwnd: HWND, action: Action) {
    if !state::try_begin_task() {
        return;
    }
    // HWND is a raw pointer and not Send.
    let hwnd = hwnd as isize;
    thread::spawn(move || {
        let report = action.run();
        if let Some(mode) = mode_after(action) {
            state::set_mode(mode);
        }
        state::end_task();

        let report = Box::into_raw(Box::new(report));
        // SAFETY: if the post succeeds the window procedure owns `report`;
        // otherwise nobody does, so free it here.
        unsafe {
            if PostMessageW(hwnd as HWND, WM_TASK_DONE, 0, report as LPARAM) == 0 {
                drop(Box::from_raw(report));
            }
        }
    });
}

fn open_apps_file() {
    let result = config::apps_file().and_then(|path| Command::new("notepad").arg(path).spawn());
    if let Err(err) = result {
        log::write(&format!("Could not open close-apps.txt: {err}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_after_matches_action() {
        assert_eq!(mode_after(Action::Play), Some(Mode::Play));
        assert_eq!(mode_after(Action::Dev), Some(Mode::Dev));
        assert_eq!(mode_after(Action::Cleanup), None);
    }
}
