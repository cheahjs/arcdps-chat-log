// Taken from https://github.com/Krappa322/arcdps_squad_manager/blob/98aeddde1d5af6fe5525dffdada9e851816bd917/src/infra.rs

use backtrace::Backtrace;
use log::*;
use winapi::shared::ntdef::TRUE;
use winapi::um::dbghelp;
use winapi::um::processthreadsapi::GetCurrentProcess;

pub fn install_panic_handler() {
    debug!("Installing panic handler");
    std::panic::set_hook(Box::new(panic_handler));
    debug!("Installed panic handler")
}

fn panic_handler(panic_info: &std::panic::PanicInfo) {
    unsafe {
        let result = dbghelp::SymCleanup(GetCurrentProcess());
        debug!("SymCleanup returned {}", result);
        let result = dbghelp::SymInitializeW(GetCurrentProcess(), std::ptr::null(), TRUE.into());
        debug!("SymInitializeW returned {}", result);
    }

    let bt = Backtrace::new();
    error!("Caught panic \"{}\"", panic_info);

    for (i, frame) in bt.frames().iter().enumerate() {
        // Resolve module name
        let mut mod_info: dbghelp::IMAGEHLP_MODULEW64;
        unsafe {
            mod_info = std::mem::zeroed::<dbghelp::IMAGEHLP_MODULEW64>();
            mod_info.SizeOfStruct = std::mem::size_of_val(&mod_info) as u32;
            dbghelp::SymGetModuleInfoW64(GetCurrentProcess(), frame.ip() as u64, &mut mod_info);
        }

        let symbol_name = frame
            .symbols()
            .first()
            .and_then(|x| x.name().map(|y| format!("{}", y)))
            .unwrap_or_else(|| "<unknown function>".to_string());

        let symbol_offset = frame
            .symbols()
            .first()
            .and_then(|x| x.addr().map(|y| frame.ip() as u64 - y as u64))
            .unwrap_or(0x0);

        let file_name = frame
            .symbols()
            .first()
            .and_then(|x| x.filename().and_then(|y| y.to_str()))
            .unwrap_or("<unknown file>");

        let line = frame
            .symbols()
            .first()
            .and_then(|x| x.lineno())
            .unwrap_or(0);

        let module_path_buf = std::path::PathBuf::from(String::from_utf16_lossy(
            mod_info
                .ImageName
                .iter()
                .take_while(|c| **c != 0) // truncate string to null termination
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
        ));
        let module_name = module_path_buf
            .file_name()
            .and_then(|y| y.to_str())
            .unwrap_or("<unknown module>");

        let module_offset = if mod_info.BaseOfImage > 0 {
            frame.ip() as u64 - mod_info.BaseOfImage
        } else {
            0x0
        };

        error!(
            "{i}: {module_name}+0x{module_offset:x}({symbol_name}+0x{symbol_offset:x}) [0x{addr:x}] {file}:{line}",
            i = i,
            module_name = module_name,
            module_offset = module_offset,
            symbol_name = symbol_name,
            symbol_offset = symbol_offset,
            addr = frame.ip() as u64,
            file = file_name,
            line = line,
        );
    }
}
