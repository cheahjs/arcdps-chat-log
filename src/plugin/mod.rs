mod events;
mod settings;
pub mod state;
pub mod ui;

use anyhow::Context;
use arc_util::{
    settings::Settings,
    ui::{Window, WindowOptions},
};

use log::{error, info};

use crate::{
    db::ChatDatabase,
    logui::LogUi,
    notifications::Notifications,
    plugin::state::{MumbleLinkState, NotificationsState},
    tracking::Tracker,
};

use self::state::UiState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const SETTINGS_FILE: &str = "arcdps_chat_log.json";

pub struct Plugin {
    pub log_ui: Window<LogUi>,
    pub notifications: Notifications,
    pub ui_state: UiState,
    pub self_account_name: String,
    game_start: i64,
    chat_database: Option<ChatDatabase>,
    tracker: Tracker,
}

impl Plugin {
    pub fn new() -> Self {
        Self {
            log_ui: Window::new(
                WindowOptions {
                    width: 500.0,
                    height: 300.0,
                    ..WindowOptions::new("Squad Log")
                },
                LogUi::new(),
            ),
            notifications: Notifications::new(),
            ui_state: UiState::new(),
            self_account_name: String::new(),
            game_start: chrono::Utc::now().timestamp(),
            chat_database: None,
            tracker: Tracker::new(),
        }
    }

    pub fn load(&mut self) -> Result<(), anyhow::Error> {
        info!("loading v{}", VERSION);

        let mut settings = Settings::from_file(SETTINGS_FILE);

        settings.load_component(&mut self.log_ui);
        settings.load_component(&mut self.notifications);

        self.log_ui.buffer.buffer_max_size = self.log_ui.settings.log_buffer as usize;

        match ChatDatabase::try_new(&self.log_ui.settings.log_path, self.game_start)
            .context("failed to init database")
        {
            Ok(chat_database) => self.chat_database = Some(chat_database),
            Err(err) => error!("{}", err),
        }

        match self
            .notifications
            .load()
            .context("failed to load notifications module")
        {
            Ok(_) => self.ui_state.notifications_state = NotificationsState::Loaded,
            Err(err) => {
                self.ui_state.notifications_state = NotificationsState::Errored;
                error!("{}", err)
            }
        }

        match crate::MUMBLE_LINK.lock().unwrap().load() {
            Ok(mumble_link_name) => {
                self.ui_state.mumblelink_state = MumbleLinkState::Loaded(mumble_link_name)
            }
            Err(err) => {
                self.ui_state.mumblelink_state = MumbleLinkState::Errored;
                error!("{}", err)
            }
        }

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
