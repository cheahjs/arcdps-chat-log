use std::sync::Arc;
use std::sync::mpsc;
use std::sync::RwLock;
use std::thread::Builder;

use anyhow::Context;
use arcdps::ChatMessageInfo;
use arcdps::arcdps_export;
use arcdps::imgui;
use arcdps::UserInfoIter;

arcdps_export! {
    name: "Chat Log",
    sig: 0x23affe80u32,
    options_end: options_end,
    init: init,
    release: release,
    unofficial_extras_init: unofficial_extras_init,
    unofficial_extras_chat_message: unofficial_extra_chat_callback,
}

fn unofficial_extras_init(
    self_account_name: Option<&str>,
    _unofficial_extras_version: Option<&'static str>,
) {
    if let Some(name) = self_account_name {
    }
}

fn unofficial_extra_chat_callback(chat_message_info: &ChatMessageInfo) {
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn release() {}

fn options_end(ui: &imgui::Ui) {}
