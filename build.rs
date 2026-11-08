fn main() {
    slint_build::compile("ui/app.slint").expect("Slint compilation failed");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "AURI");
        res.set("FileDescription", "AURI Desktop Music Player");
        res.set("LegalCopyright", "Copyright (C) 2026");
        let _ = res.compile();
    }
}
