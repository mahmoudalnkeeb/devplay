use std::ptr::null_mut;

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconFromResourceEx, GetSystemMetrics, LoadIconW, HICON, IDI_APPLICATION, LR_DEFAULTCOLOR,
    SM_CXSMICON,
};

use super::state::Mode;

const DEV_ICO: &[u8] = include_bytes!("../../assets/tray-dev.ico");
const PLAY_ICO: &[u8] = include_bytes!("../../assets/tray-play.ico");

const ICON_FORMAT_VERSION: u32 = 0x0003_0000;
const ICO_HEADER_LEN: usize = 6;
const ICO_ENTRY_LEN: usize = 16;

fn ico_file(mode: Mode) -> &'static [u8] {
    match mode {
        Mode::Dev => DEV_ICO,
        Mode::Play => PLAY_ICO,
    }
}

pub fn load(mode: Mode) -> HICON {
    let size = unsafe { GetSystemMetrics(SM_CXSMICON) }.max(16);
    if let Some(image) = best_image(ico_file(mode), size.unsigned_abs()) {
        let icon = unsafe {
            CreateIconFromResourceEx(
                image.as_ptr(),
                image.len() as u32,
                1,
                ICON_FORMAT_VERSION,
                size,
                size,
                LR_DEFAULTCOLOR,
            )
        };
        if !icon.is_null() {
            return icon;
        }
    }
    unsafe { LoadIconW(null_mut(), IDI_APPLICATION) }
}

/// Prefers an exact match, then the closest larger image, then the closest smaller one.
fn best_image(ico: &[u8], size: u32) -> Option<&[u8]> {
    let count = usize::from(u16::from_le_bytes(ico.get(4..6)?.try_into().ok()?));
    let mut best: Option<((u8, u32), &[u8])> = None;
    for index in 0..count {
        let start = ICO_HEADER_LEN + index * ICO_ENTRY_LEN;
        let entry = ico.get(start..start + ICO_ENTRY_LEN)?;
        let dim = if entry[0] == 0 {
            256
        } else {
            u32::from(entry[0])
        };
        let len = u32::from_le_bytes(entry[8..12].try_into().ok()?) as usize;
        let offset = u32::from_le_bytes(entry[12..16].try_into().ok()?) as usize;
        let image = ico.get(offset..offset.checked_add(len)?)?;
        let score = fit(dim, size);
        if best.is_none_or(|(best_score, _)| score < best_score) {
            best = Some((score, image));
        }
    }
    best.map(|(_, image)| image)
}

fn fit(dim: u32, want: u32) -> (u8, u32) {
    if dim >= want {
        (0, dim - want)
    } else {
        (1, want - dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image_width(image: &[u8]) -> u32 {
        if image.starts_with(b"\x89PNG") {
            u32::from_be_bytes(image[16..20].try_into().unwrap())
        } else {
            u32::from_le_bytes(image[4..8].try_into().unwrap())
        }
    }

    #[test]
    fn picks_exact_size() {
        for ico in [DEV_ICO, PLAY_ICO] {
            assert_eq!(image_width(best_image(ico, 16).unwrap()), 16);
            assert_eq!(image_width(best_image(ico, 32).unwrap()), 32);
        }
    }

    #[test]
    fn prefers_next_larger_size() {
        assert_eq!(image_width(best_image(DEV_ICO, 22).unwrap()), 24);
    }

    #[test]
    fn falls_back_to_largest_when_too_big() {
        assert_eq!(image_width(best_image(DEV_ICO, 512).unwrap()), 48);
    }

    #[test]
    fn rejects_malformed_files() {
        assert_eq!(best_image(&[], 16), None);
        assert_eq!(best_image(&[0, 0, 1, 0, 5, 0], 16), None);
        let mut truncated = DEV_ICO[..40].to_vec();
        truncated[4] = 1;
        assert_eq!(best_image(&truncated, 16), None);
    }
}
