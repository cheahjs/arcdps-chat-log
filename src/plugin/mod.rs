mod events;
mod settings;
pub mod state;
pub mod ui;

use std::sync::{Arc, Mutex};

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
    plugin::state::{MumbleLinkState, NotificationsState, TtsState},
    tracking::Tracker,
    tts::TextToSpeech,
    update::{self, UpdateState},
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
    chat_database: Option<Arc<Mutex<ChatDatabase>>>,
    tts: TextToSpeech,
    tracker: Tracker,
    pub update_state: UpdateState,
}

impl Plugin {
    pub fn new() -> Self {
        Self {
            log_ui: Window::new(
                "Squad Log",
                LogUi::new(),
                WindowOptions {
                    width: 500.0,
                    height: 300.0,
                    ..WindowOptions::new()
                },
            ),
            notifications: Notifications::new(),
            ui_state: UiState::new(),
            self_account_name: String::new(),
            game_start: chrono::Utc::now().timestamp(),
            chat_database: None,
            tts: TextToSpeech::new(),
            tracker: Tracker::new(),
            update_state: UpdateState::new(
                Some(update::get_current_version()),
                update::get_dll_path().unwrap_or_default(),
            ),
        }
    }

    pub fn load(&mut self) -> Result<(), anyhow::Error> {
        info!("loading v{}", VERSION);

        let mut settings = Settings::from_file(SETTINGS_FILE);

        settings.load_component(&mut self.log_ui);
        settings.load_component(&mut self.notifications);
        settings.load_component(&mut self.tts);
        settings.load_component(&mut self.update_state);

        // Clean up leftover update files from a previous run, then (if enabled) check for a new version.
        if !self.update_state.install_path.as_os_str().is_empty() {
            update::clear_old_files(&self.update_state.install_path);
        }
        if self.update_state.settings.check_enabled {
            let include_prereleases = self.update_state.settings.include_prereleases;
            update::check_for_update(&mut self.update_state, include_prereleases);
        }

        self.log_ui.buffer.buffer_max_size = self.log_ui.settings.log_buffer as usize;

        match ChatDatabase::try_new(&self.log_ui.settings.log_path, self.game_start)
            .context("failed to init database")
        {
            Ok(chat_database) => {
                self.chat_database = Some(Arc::new(Mutex::new(chat_database)));
                self.log_ui.chat_database = self.chat_database.clone();
            }
            Err(err) => error!("{:#}", err),
        }

        match self
            .notifications
            .load()
            .context("failed to load notifications module")
        {
            Ok(_) => self.ui_state.notifications_state = NotificationsState::Loaded,
            Err(err) => {
                self.ui_state.notifications_state = NotificationsState::Errored;
                error!("{:#}", err)
            }
        }

        match crate::MUMBLE_LINK.lock().unwrap().load() {
            Ok(mumble_link_name) => {
                self.ui_state.mumblelink_state = MumbleLinkState::Loaded(mumble_link_name)
            }
            Err(err) => {
                self.ui_state.mumblelink_state = MumbleLinkState::Errored;
                error!("{:#}", err)
            }
        }

        match self.tts.init().context("failed to load tts module") {
            Ok(_) => self.ui_state.tts_state = TtsState::Loaded,
            Err(err) => {
                self.ui_state.tts_state = TtsState::Errored;
                error!("{:#}", err)
            }
        }

        Ok(())
    }

    pub fn release(&mut self) {
        self.update_state.finish_pending_tasks();
        if let Some(chat_database) = &self.chat_database {
            chat_database.lock().unwrap().release();
        }
        let mut settings = Settings::from_file(SETTINGS_FILE);
        settings.store_component(&self.log_ui);
        settings.store_component(&self.notifications);
        settings.store_component(&self.tts);
        settings.store_component(&self.update_state);
        settings.save_file();
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new()
    }
}
