use std::path::PathBuf;

use arc_util::settings::HasSettings;
use serde::{Deserialize, Serialize};

use super::LogUi;

const DEFAULT_LOG_PATH: &str = "arcdps_chat_log.db";

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(default)]
pub struct FilterSettings {
    pub squad_message: bool,
    pub party_message: bool,
    pub squad_updates: bool,
    pub combat_updates: bool,
    pub others: bool,
    pub hover_char_name_for_account_name: bool,
}

impl FilterSettings {
    pub fn new() -> Self {
        Self {
            squad_message: true,
            party_message: true,
            squad_updates: true,
            combat_updates: true,
            others: true,
            hover_char_name_for_account_name: true,
        }
    }
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(default)]
pub struct ColorSettings {
    pub squad_chat: [f32; 4],
    pub squad_user: [f32; 4],
    pub party_chat: [f32; 4],
    pub party_user: [f32; 4],
}

impl ColorSettings {
    pub fn new() -> Self {
        #[allow(clippy::eq_op)]
        Self {
            squad_chat: [205.0 / 255.0, 255.0 / 255.0, 239.0 / 255.0, 1.0],
            party_chat: [188.0 / 255.0, 222.0 / 255.0, 255.0 / 255.0, 1.0],
            squad_user: [192.0 / 255.0, 241.0 / 255.0, 97.0 / 255.0, 1.0],
            party_user: [68.0 / 255.0, 188.0 / 255.0, 255.0 / 255.0, 1.0],
        }
    }
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatLogSettings {
    pub log_enabled: bool,
    pub log_path: String,
    pub log_buffer: i32,
    pub color_settings: ColorSettings,
    pub filter_settings: FilterSettings,
}

impl ChatLogSettings {
    pub fn new() -> Self {
        Self {
            log_enabled: true,
            log_path: Self::default_log_path().to_str().unwrap().to_string(),
            log_buffer: 10000,
            color_settings: ColorSettings::new(),
            filter_settings: FilterSettings::new(),
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
