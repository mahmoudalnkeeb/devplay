use std::fs;
use std::io;
use std::path::PathBuf;

const APPS_FILE: &str = "close-apps.txt";

const DEFAULT_APPS: &str = "\
# Apps Play Mode asks to close (like clicking X, so they can save).
# One process name per line. Lines starting with # are ignored.
brave.exe
WhatsApp.Root.exe
Code.exe
";

pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
        .unwrap_or_default()
}

/// Creates the file with defaults on first use.
pub fn apps_file() -> io::Result<PathBuf> {
    let path = exe_dir().join(APPS_FILE);
    if !path.exists() {
        fs::write(&path, DEFAULT_APPS)?;
    }
    Ok(path)
}

pub fn apps_to_close() -> Vec<String> {
    let text = apps_file()
        .and_then(fs::read_to_string)
        .unwrap_or_else(|_| DEFAULT_APPS.to_owned());
    parse_apps(&text)
}

fn parse_apps(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skips_comments_and_blank_lines() {
        let text = "# comment\n\n  brave.exe  \r\n#Code.exe\nfoo.exe\n";
        assert_eq!(parse_apps(text), ["brave.exe", "foo.exe"]);
    }

    #[test]
    fn defaults_parse_to_three_apps() {
        assert_eq!(
            parse_apps(DEFAULT_APPS),
            ["brave.exe", "WhatsApp.Root.exe", "Code.exe"]
        );
    }
}
