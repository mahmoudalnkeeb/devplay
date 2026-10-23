use std::mem::{size_of, zeroed};

use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO, NIIF_NOSOUND, NIM_ADD,
    NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
};

use super::state;
use super::WM_TRAY;
use crate::sys::wide::copy_truncated;

const ICON_ID: u32 = 1;
const BALLOON_TITLE: &str = "DevPlay";

pub fn add() {
    send(NIM_ADD, None);
}

pub fn remove() {
    send(NIM_DELETE, None);
}

/// Also refreshes the icon and tooltip to match the current mode.
pub fn show_message(text: &str) {
    send(NIM_MODIFY, Some(text));
}

fn send(message: u32, balloon: Option<&str>) {
    let mode = state::mode();
    let mut data: NOTIFYICONDATAW = unsafe { zeroed() };
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = state::window();
    data.uID = ICON_ID;
    data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = state::icon(mode);
    copy_truncated(&mut data.szTip, mode.tooltip());
    if let Some(text) = balloon {
        data.uFlags |= NIF_INFO;
        data.dwInfoFlags = NIIF_INFO | NIIF_NOSOUND;
        copy_truncated(&mut data.szInfoTitle, BALLOON_TITLE);
        copy_truncated(&mut data.szInfo, text);
    }
    unsafe { Shell_NotifyIconW(message, &data) };
}
