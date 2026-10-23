use std::mem::zeroed;

use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

pub fn timestamp() -> String {
    let t: SYSTEMTIME = unsafe {
        let mut t = zeroed();
        GetLocalTime(&mut t);
        t
    };
    format(&t)
}

fn format(t: &SYSTEMTIME) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_zero_padding() {
        let t = SYSTEMTIME {
            wYear: 2026,
            wMonth: 3,
            wDayOfWeek: 0,
            wDay: 7,
            wHour: 9,
            wMinute: 5,
            wSecond: 0,
            wMilliseconds: 0,
        };
        assert_eq!(format(&t), "2026-03-07 09:05");
    }
}
