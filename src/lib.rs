mod db;
mod logui;
mod mumblelink;
mod notifications;
mod panic_handler;
mod plugin;

use arcdps::extras::{message::ChatMessageInfo, ExtrasAddonInfo};
use arcdps::imgui::Ui;
use log::*;
use mumblelink::MumbleLink;
use plugin::Plugin;

use std::sync::Mutex;

use anyhow::{Context, Result};
use once_cell::sync::Lazy;

static PLUGIN: Lazy<Mutex<Plugin>> = Lazy::new(|| Mutex::new(Plugin::new()));
static MUMBLE_LINK: Lazy<Mutex<MumbleLink>> = Lazy::new(|| Mutex::new(MumbleLink::new().unwrap()));

arcdps::export! {
    name: "Chat Log",
    sig: 0x23affe80u32,
    init,
    release,
    options_end,
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
    PLUGIN.lock().unwrap().process_message(chat_message_info)?;
    Ok(())
}

fn options_end(ui: &Ui) {
    if let Err(err) = PLUGIN.lock().unwrap().render_settings(ui) {
        error!("failed while rendering settings: {}", err);
    }
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    debug!("arc init");
    panic_handler::install_panic_handler();

    PLUGIN
        .lock()
        .unwrap()
        .load()
        .context("failed to load plugin")?;

    Ok(())
}

fn release() {
    debug!("release called");
    PLUGIN.lock().unwrap().release();
}
