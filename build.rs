fn main() {
    slint_build::compile("ui/app.slint").expect("Slint build failed");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "AURI");
        res.set("FileDescription", "AURI - Minimalist, Ultra-Lightweight Desktop Music Player");
        res.set("LegalCopyright", "Copyright (c) 2026 AURI Contributors");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource: {}", e);
        }
    }
}

