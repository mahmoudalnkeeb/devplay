pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Truncates so the terminating NUL always fits.
pub fn copy_truncated(dst: &mut [u16], s: &str) {
    let Some(max) = dst.len().checked_sub(1) else {
        return;
    };
    let mut len = 0;
    for (slot, unit) in dst.iter_mut().zip(s.encode_utf16().take(max)) {
        *slot = unit;
        len += 1;
    }
    dst[len] = 0;
}

pub fn from_wide(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_wide_appends_nul() {
        assert_eq!(to_wide("ab"), vec![b'a' as u16, b'b' as u16, 0]);
    }

    #[test]
    fn copy_truncated_fits_short_strings() {
        let mut buf = [0xFFFFu16; 8];
        copy_truncated(&mut buf, "hi");
        assert_eq!(from_wide(&buf), "hi");
        assert_eq!(buf[2], 0);
    }

    #[test]
    fn copy_truncated_leaves_room_for_nul() {
        let mut buf = [0u16; 4];
        copy_truncated(&mut buf, "abcdef");
        assert_eq!(from_wide(&buf), "abc");
        assert_eq!(buf[3], 0);
    }

    #[test]
    fn copy_truncated_ignores_empty_buffer() {
        let mut buf: [u16; 0] = [];
        copy_truncated(&mut buf, "abc");
    }

    #[test]
    fn from_wide_without_nul_reads_whole_buffer() {
        let buf: Vec<u16> = "xyz".encode_utf16().collect();
        assert_eq!(from_wide(&buf), "xyz");
    }
}
