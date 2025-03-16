use std::path::PathBuf;

use arc_util::settings::HasSettings;
use serde::{Deserialize, Serialize};
use arcdps::imgui::ImColor32;

use super::LogUi;

const DEFAULT_LOG_PATH: &str = "arcdps_chat_log.db";
const DEFAULT_HOTKEY: u32 = 0x4A; // VK_J

#[derive(Debug, Clone)]
pub struct FilterSettings {
    pub hover_char_name_for_account_name: bool,
    pub squad_message: bool,
    pub party_message: bool,
    pub squad_updates: bool,
    pub combat_updates: bool,
    pub others: bool,
}

impl FilterSettings {
    pub fn new() -> Self {
        Self {
            hover_char_name_for_account_name: true,
            squad_message: true,
            party_message: true,
            squad_updates: true,
            combat_updates: true,
            others: true,
        }
    }
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorSettings {
    pub squad_message: ImColor32,
    pub party_message: ImColor32,
    pub squad_updates: ImColor32,
    pub combat_updates: ImColor32,
    pub others: ImColor32,
}

impl ColorSettings {
    pub fn new() -> Self {
        Self {
            squad_message: ImColor32::from_rgb(255, 255, 255),
            party_message: ImColor32::from_rgb(0, 255, 0),
            squad_updates: ImColor32::from_rgb(255, 255, 0),
            combat_updates: ImColor32::from_rgb(255, 0, 0),
            others: ImColor32::from_rgb(128, 128, 128),
        }
    }
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ChatLogSettings {
    pub show_filters: bool,
    pub show_seen_users: bool,
    pub filter_text: String,
    pub filter_settings: FilterSettings,
    pub color_settings: ColorSettings,
    pub hotkey: Option<u32>,
}

impl ChatLogSettings {
    pub fn new() -> Self {
        Self {
            show_filters: false,
            show_seen_users: false,
            filter_text: String::new(),
            filter_settings: FilterSettings::default(),
            color_settings: ColorSettings::default(),
            hotkey: Some(DEFAULT_HOTKEY),
        }
    }

    fn default_log_path() -> PathBuf {
        arcdps::exports::config_path()
            .map(|mut path| {
                if !path.is_dir() {
                    path.pop();
                }
                path.push(DEFAULT_LOG_PATH);
                path
            })
            .unwrap()
    }
}

impl Default for ChatLogSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl HasSettings for LogUi {
    type Settings = ChatLogSettings;

    const SETTINGS_ID: &'static str = "log";

    fn current_settings(&self) -> Self::Settings {
        self.settings.clone()
    }

    fn load_settings(&mut self, loaded: Self::Settings) {
        self.settings = loaded;
        self.update_settings();
    }
}
