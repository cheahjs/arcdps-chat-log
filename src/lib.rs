mod panic_handler;

#[macro_use]
extern crate simple_error;

use arcdps::extras::{ExtrasAddonInfo, message::{ChatMessageInfoOwned, ChatMessageInfo}};
use log::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Result, Context};
use arc_util::settings::Settings;
use chrono::Utc;
use once_cell::sync::OnceCell;

const SETTINGS_FILE: &str = "arcdps_chat_log.json";
const DEFAULT_LOG_PATH: &str = "chatlogs";

static SETTINGS: OnceCell<Mutex<Settings>> = OnceCell::new();
static LOG_FILE: OnceCell<File> = OnceCell::new();

arcdps::export! {
    name: "Chat Log",
    sig: 0x23affe80u32,
    init,
    extras_init: extras_init,
    extras_chat_message: extras_chat_callback,
}

fn extras_init(
    _addon_info: ExtrasAddonInfo,
    _account_name: Option<&'static str>,
) {}


fn extras_chat_callback(chat_message_info: &ChatMessageInfo) {
    let log_file = LOG_FILE.get();
    if log_file.is_none() {
        return;
    }
    let json_str = serde_json::to_string(&ChatMessageInfoOwned::from(chat_message_info.to_owned()));
    if json_str.is_err() {
        return;
    }
    log_file.unwrap().write_all(format!("{}\n", json_str.unwrap()).as_bytes());

    log_file.unwrap().flush();
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    debug!("arc init");
    panic_handler::install_panic_handler();

    SETTINGS.set(Mutex::new(Settings::from_file(SETTINGS_FILE))).unwrap();

    let log_path: String = SETTINGS.get().unwrap().lock().unwrap().load_data("log_path").unwrap_or(DEFAULT_LOG_PATH.to_string());
    let mut current_log_path = PathBuf::from(log_path);
    let now = Utc::now();
    current_log_path.push(now.format("%Y-%m-%d-%H-%M-%S").to_string());
    current_log_path.set_extension("jsonl");
    let parent = current_log_path.parent().unwrap();
    std::fs::create_dir_all(parent).context(format!("failed to create directory {}", parent.display()))?;
    let file = File::create(&current_log_path).context(format!("failed to create log file {}", current_log_path.display()))?;
    let file_set = LOG_FILE.set(file);
    if file_set.is_err() {
        bail!("failed to set file");
    }
    Ok(())
}

