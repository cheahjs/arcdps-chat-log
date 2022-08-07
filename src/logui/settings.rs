use std::path::PathBuf;

use arc_util::settings::HasSettings;
use serde::{Deserialize, Serialize};

use super::LogUi;

const DEFAULT_LOG_PATH: &str = "arcdps_chat_log.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatLogSettings {
    pub log_enabled: bool,
    pub log_path: String,
}

impl ChatLogSettings {
    /// Creates new reminder settings with the defaults.
    pub fn new() -> Self {
        Self {
            log_enabled: true,
            log_path: Self::default_log_path().to_str().unwrap().to_string(),
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
    }
}
