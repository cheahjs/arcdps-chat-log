fn main() {
    if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default() != "windows" {
        return;
    }

    let mut res = winres::WindowsResource::new();

    // `winres` can discover `rc.exe` automatically for MSVC targets, but it
    // still expects a `windres` binary when targeting the GNU toolchain. Only
    // override the resource compiler when we know we're targeting GNU to avoid
    // leaking that requirement onto the MSVC + cargo-xwin workflow.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        if let Ok(windres) = std::env::var("WINDRES") {
            res.set_windres_path(&windres);
        } else {
            // Fall back to the common cross toolchain binary so building on
            // non-Windows hosts works without extra environment configuration.
            res.set_windres_path("x86_64-w64-mingw32-windres");
        }
    } else if let Ok(llvm_rc) = std::env::var("LLVM_RC") {
        // Allow explicitly providing llvm-rc for the MSVC target. winres looks
        // for an `rc.exe` inside its toolkit directory, so we shim one that
        // points at llvm-rc.
        let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| ".".into());
        let shim_dir = std::path::PathBuf::from(out_dir).join("llvm-rc-shim");
        let shim_rc = shim_dir.join("rc.exe");
        if std::fs::create_dir_all(&shim_dir).is_ok() {
            let llvm_rc_path = std::path::PathBuf::from(&llvm_rc);
            let shim_result = {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(&llvm_rc_path, &shim_rc)
                }
                #[cfg(not(unix))]
                {
                    std::fs::copy(&llvm_rc_path, &shim_rc).map(|_| ())
                }
            };
            if shim_result.is_ok() {
                if let Some(toolkit_path) = shim_rc.parent() {
                    res.set_toolkit_path(toolkit_path.to_string_lossy().as_ref());
                    eprintln!("info: using llvm-rc via shim at {}", shim_rc.display());
                }
            } else {
                eprintln!("warning: failed to create llvm-rc shim, falling back to rc.exe lookup");
            }
        }
    }

    if let Err(err) = res.compile() {
        eprintln!("warning: skipping Windows resource compilation (rc missing?): {err}");
    }
}
