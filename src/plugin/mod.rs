mod events;
mod settings;

use anyhow::Context;
use arc_util::settings::Settings;
use log::info;

use crate::{db::ChatDatabase, logui::LogUi, notifications::Notifications};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const SETTINGS_FILE: &str = "arcdps_chat_log.json";

pub struct Plugin {
    pub log_ui: LogUi,
    pub notifications: Notifications,
    game_start: i64,
    chat_database: Option<ChatDatabase>,
}

impl Plugin {
    pub fn new() -> Self {
        Self {
            log_ui: LogUi::new(),
            notifications: Notifications::new(),
            game_start: chrono::Utc::now().timestamp(),
            chat_database: None,
        }
    }

    pub fn load(&mut self) -> Result<(), anyhow::Error> {
        info!("loading v{}", VERSION);

        let mut settings = Settings::from_file(SETTINGS_FILE);

        settings.load_component(&mut self.log_ui);
        settings.load_component(&mut self.notifications);

        self.chat_database = Some(
            ChatDatabase::try_new(&self.log_ui.settings.log_path, self.game_start)
                .context("failed to init database")?,
        );

        Ok(())
    }

    pub fn release(&mut self) {
        if let Some(chat_database) = self.chat_database.as_mut() {
            chat_database.release();
        }
        let mut settings = Settings::from_file(SETTINGS_FILE);
        settings.store_component(&self.log_ui);
        settings.store_component(&self.notifications);
        settings.save_file();
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new()
    }
}
