fn main() {
    if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        if let Ok(windres) = std::env::var("WINDRES") {
            res.set_windres_path(&windres);
        }
        res.compile().unwrap();
    }
}
