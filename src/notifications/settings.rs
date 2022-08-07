use arc_util::settings::HasSettings;
use serde::{Deserialize, Serialize};

use super::Notifications;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsSettings {
    pub ping_volume: i32,
    pub ping_sound_path: String,
    pub ping_on_all_new_messages: bool,
    pub ping_in_combat: bool,
    pub ping_out_of_combat: bool,
}

impl NotificationsSettings {
    pub fn new() -> Self {
        Self {
            ping_volume: 100,
            ping_sound_path: "".to_owned(),
            ping_on_all_new_messages: false,
            ping_in_combat: true,
            ping_out_of_combat: true,
        }
    }
}

impl Default for NotificationsSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl HasSettings for Notifications {
    type Settings = NotificationsSettings;

    const SETTINGS_ID: &'static str = "notifications";

    fn current_settings(&self) -> Self::Settings {
        self.settings.clone()
    }

    fn load_settings(&mut self, loaded: Self::Settings) {
        self.settings = loaded;
    }
}
