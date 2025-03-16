mod audio;
mod db;
mod logui;
mod mumblelink;
mod notifications;
mod panic_handler;
mod plugin;
mod tracking;
mod tts;

use arcdps::{
    extras::{
        ExtrasAddonInfo,
        message::SquadMessage,
        user::UserInfoIter,
    },
    Event as CombatEvent,
    Agent, StateChange,
};
use arcdps::imgui::Ui;
use audio::player::AudioPlayer;
use log::*;
use mumblelink::MumbleLink;
use plugin::Plugin;

use std::sync::Mutex;

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::error::Error;
use std::ffi::c_void;
use std::ptr::NonNull;

static PLUGIN: Lazy<Mutex<Option<Plugin>>> = Lazy::new(|| Mutex::new(None));
static MUMBLE_LINK: Lazy<Mutex<MumbleLink>> = Lazy::new(|| Mutex::new(MumbleLink::new()));
static AUDIO_PLAYER: Lazy<Mutex<AudioPlayer>> = Lazy::new(|| Mutex::new(AudioPlayer::new()));

arcdps::export! {
    name: "Squad Log",
    sig: 0x1337,
    init: init_wrapper,
    release,
    imgui,
    extras_init: extras_init_wrapper,
    extras_squad_chat_message: extras_chat_callback_wrapper,
    extras_squad_update: squad_update_wrapper,
    wnd_filter,
}

fn init_wrapper() -> Result<(), String> {
    init(None).map_err(|e| e.to_string())
}

fn init(swapchain: Option<NonNull<c_void>>) -> Result<(), Box<dyn Error>> {
    debug!("arc init");
    if let Err(e) = panic_handler::install_panic_handler() {
        return Err(Box::new(e));
    }

    let mut plugin = PLUGIN.lock().unwrap();
    *plugin = Some(Plugin::new());
    Ok(())
}

fn extras_init_wrapper(addon_info: ExtrasAddonInfo, account_name: Option<&str>) {
    if let Ok(mut plugin) = PLUGIN.lock() {
        if let Some(plugin) = plugin.as_mut() {
            if plugin::events::extras_init(&addon_info) {
                if let Some(tracking) = &mut plugin.tracking {
                    tracking.self_account_name = account_name.unwrap_or("").to_string();
                }
            }
        }
    }
}

fn extras_chat_callback_wrapper(msg: &SquadMessage) {
    if let Ok(mut plugin) = PLUGIN.lock() {
        if let Some(plugin) = plugin.as_mut() {
            plugin::events::extras_chat_callback(msg);
        }
    }
}

fn squad_update_wrapper(users: UserInfoIter) {
    if let Ok(mut plugin) = PLUGIN.lock() {
        if let Some(plugin) = plugin.as_mut() {
            for user in users {
                plugin::events::squad_update(&user, true);
            }
        }
    }
}

fn imgui(ui: &Ui, not_loading_or_character_selection: bool) {
    if let Ok(mut plugin) = PLUGIN.lock() {
        if let Some(plugin) = plugin.as_mut() {
            plugin.render_windows(ui, not_loading_or_character_selection);
        }
    }
}

fn release() {
    debug!("arc release");
    if let Ok(mut plugin) = PLUGIN.lock() {
        if let Some(plugin) = plugin.as_mut() {
            plugin.release();
        }
    }
}

fn wnd_filter(key: usize, key_down: bool, prev_key_down: bool) -> bool {
    if let Ok(mut plugin) = PLUGIN.lock() {
        if let Some(plugin) = plugin.as_mut() {
            plugin.key_event(key, key_down, prev_key_down)
        } else {
            false
        }
    } else {
        false
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
    if let Some(plugin) = PLUGIN.lock().unwrap().as_mut() {
        plugin.combat(event, src, dst, skill_name, id, revision);
    }
}

fn internal_chat_callback(chat_message_info: &SquadMessage) -> Result<(), anyhow::Error> {
    if let Some(plugin) = PLUGIN.lock().unwrap().as_mut() {
        plugin.process_message(chat_message_info)
    } else {
        Ok(())
    }
}
