fn main() {
    println!("cargo:rerun-if-changed=assets/devplay.ico");

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/devplay.ico")
        .set("ProductName", "DevPlay")
        .set("FileDescription", "DevPlay")
        .set("LegalCopyright", "Copyright (c) 2026 Mahmoud Alnakeeb");

    if let Err(err) = res.compile() {
        // Releases must carry the icon and version info; debug builds may skip
        // them so machines without a resource compiler can still run the tests.
        if std::env::var("PROFILE").as_deref() == Ok("release") {
            panic!("failed to compile Windows resources ({err}): release builds need rc.exe (Windows SDK) or windres (MinGW)");
        }
        println!("cargo:warning=Windows resources skipped: {err}");
    }
}
