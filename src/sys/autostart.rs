use super::process::run;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "DevPlay";

pub fn is_enabled() -> bool {
    run("reg", &["query", RUN_KEY, "/v", VALUE_NAME])
}

pub fn set_enabled(enabled: bool) -> bool {
    if !enabled {
        return run("reg", &["delete", RUN_KEY, "/v", VALUE_NAME, "/f"]);
    }
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let command = format!("\"{}\"", exe.display());
    run(
        "reg",
        &[
            "add", RUN_KEY, "/v", VALUE_NAME, "/t", "REG_SZ", "/d", &command, "/f",
        ],
    )
}
