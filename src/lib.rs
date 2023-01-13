mod audio;
mod db;
mod logui;
mod mumblelink;
mod notifications;
mod panic_handler;
mod plugin;
mod tracking;
mod tts;

use arcdps::callbacks::CombatCallback;
use arcdps::extras::UserInfoIter;
use arcdps::extras::{message::ChatMessageInfo, ExtrasAddonInfo};
use arcdps::imgui::Ui;
use arcdps::{Agent, CombatEvent};
use audio::player::AudioPlayer;
use log::*;
use mumblelink::MumbleLink;
use plugin::Plugin;

use std::sync::Mutex;

use anyhow::{Context, Result};
use once_cell::sync::Lazy;

static PLUGIN: Lazy<Mutex<Plugin>> = Lazy::new(|| Mutex::new(Plugin::new()));
static MUMBLE_LINK: Lazy<Mutex<MumbleLink>> = Lazy::new(|| Mutex::new(MumbleLink::new()));
static AUDIO_PLAYER: Lazy<Mutex<AudioPlayer>> = Lazy::new(|| Mutex::new(AudioPlayer::new()));

arcdps::export! {
    name: "Chat Log",
    sig: 0x23affe80u32,
    init,
    release,
    options_end,
    options_windows,
    imgui,
    combat,
    extras_init,
    extras_chat_message: extras_chat_callback,
    extras_squad_update,
    wnd_filter,
}

fn imgui(ui: &Ui, not_loading_or_character_selection: bool) {
    PLUGIN
        .lock()
        .unwrap()
        .render_windows(ui, not_loading_or_character_selection)
}

fn extras_init(addon_info: ExtrasAddonInfo, account_name: Option<&str>) {
    PLUGIN
        .lock()
        .unwrap()
        .extras_init(&addon_info, account_name);
}

fn extras_chat_callback(chat_message_info: &ChatMessageInfo) {
    debug!("chat callback: {:?}", chat_message_info);
    match internal_chat_callback(chat_message_info) {
        Ok(_) => {}
        Err(err) => {
            error!("failed to process chat message: {:#}", err)
        }
    }
}

fn combat(
    event: Option<CombatEvent>,
    src: Option<Agent>,
    dst: Option<Agent>,
    skill_name: Option<&'static str>,
    id: u64,
    revision: u64,
) {
    PLUGIN
        .lock()
        .unwrap()
        .combat(event, src, dst, skill_name, id, revision)
}

fn extras_squad_update(users: UserInfoIter) {
    PLUGIN.lock().unwrap().squad_update(users)
}

fn internal_chat_callback(chat_message_info: &ChatMessageInfo) -> Result<(), anyhow::Error> {
    PLUGIN.lock().unwrap().process_message(chat_message_info)
}

fn options_end(ui: &Ui) {
    PLUGIN.lock().unwrap().render_settings(ui)
}

fn options_windows(ui: &Ui, option_name: Option<&str>) -> bool {
    PLUGIN
        .lock()
        .unwrap()
        .render_window_options(ui, option_name)
}

fn wnd_filter(key: usize, key_down: bool, prev_key_down: bool) -> bool {
    PLUGIN
        .lock()
        .unwrap()
        .key_event(key, key_down, prev_key_down)
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
    AUDIO_PLAYER.lock().unwrap().release();
}
