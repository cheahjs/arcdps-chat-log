use arc_util::settings::HasSettings;
use serde::{Deserialize, Serialize};

use super::Notifications;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsSettings {}

impl NotificationsSettings {
    /// Creates new reminder settings with the defaults.
    pub fn new() -> Self {
        Self {}
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
