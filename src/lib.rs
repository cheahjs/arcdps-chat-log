mod db;
mod panic_handler;

#[macro_use]
extern crate simple_error;

use arcdps::extras::{message::ChatMessageInfo, ExtrasAddonInfo};
use db::ChatDatabase;
use log::*;

use std::sync::Mutex;

use anyhow::{Context, Result};
use arc_util::settings::Settings;
use once_cell::sync::OnceCell;

const SETTINGS_FILE: &str = "arcdps_chat_log.json";

static SETTINGS: OnceCell<Mutex<Settings>> = OnceCell::new();
static DB: OnceCell<Mutex<ChatDatabase>> = OnceCell::new();
static GAME_START: OnceCell<i64> = OnceCell::new();

arcdps::export! {
    name: "Chat Log",
    sig: 0x23affe80u32,
    init,
    release,
    extras_init: extras_init,
    extras_chat_message: extras_chat_callback,
}

fn extras_init(_addon_info: ExtrasAddonInfo, _account_name: Option<&'static str>) {}

fn extras_chat_callback(chat_message_info: &ChatMessageInfo) {
    debug!("chat callback: {:?}", chat_message_info);
    match internal_chat_callback(chat_message_info) {
        Ok(_) => {}
        Err(err) => {
            error!("failed to process chat message: {}", err)
        }
    }
}

fn internal_chat_callback(chat_message_info: &ChatMessageInfo) -> Result<(), anyhow::Error> {
    DB.get()
        .context("failed to get database")?
        .lock()
        .unwrap()
        .process_message(chat_message_info)
        .context("failed to process message")?;
    Ok(())
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    debug!("arc init");
    panic_handler::install_panic_handler();

    if GAME_START.set(chrono::Utc::now().timestamp()).is_err() {
        bail!("game start init twice");
    };

    if SETTINGS
        .set(Mutex::new(Settings::from_file(SETTINGS_FILE)))
        .is_err()
    {
        // This shouldn't be possible unless init is called twice
        bail!("settings init twice");
    };

    if DB
        .set(Mutex::new(
            ChatDatabase::try_new().context("failed to init database")?,
        ))
        .is_err()
    {
        bail!("database init twice");
    }

    Ok(())
}

fn release() {
    debug!("release called");
    if let Some(mutex) = DB.get() {
        mutex.lock().unwrap().release();
    }
    if let Some(settings) = SETTINGS.get() {
        settings.lock().unwrap().save_file();
    }
}
