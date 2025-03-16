pub mod events;
mod settings;
pub mod state;
pub mod ui;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

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
};

use self::state::UiState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const SETTINGS_FILE: &str = "arcdps_chat_log.json";

pub struct Plugin<'a> {
    pub log_ui: Window<LogUi<'a>>,
    pub notifications: Option<Notifications>,
    pub ui_state: UiState,
    pub self_account_name: String,
    pub tracking: Option<Arc<Mutex<Tracker>>>,
    game_start: i64,
    chat_database: Option<Arc<Mutex<ChatDatabase>>>,
    tts: TextToSpeech,
}

impl<'a> Plugin<'a> {
    pub fn new() -> Self {
        let game_start = chrono::Utc::now().timestamp();
        Self {
            log_ui: Window::new("Chat Log"),
            notifications: None,
            ui_state: UiState::default(),
            self_account_name: String::new(),
            tracking: Some(Arc::new(Mutex::new(Tracker::new(String::new())))),
            game_start,
            chat_database: None,
            tts: TextToSpeech::new(),
        }
    }

    pub fn load(&mut self) -> Result<(), anyhow::Error> {
        info!("loading v{}", VERSION);

        let mut settings = Settings::from_file(SETTINGS_FILE);

        settings.load_component(&mut self.log_ui);
        settings.load_component(&mut self.notifications);
        settings.load_component(&mut self.tts);

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
        if let Some(chat_database) = &self.chat_database {
            chat_database.lock().unwrap().release();
        }
        let mut settings = Settings::from_file(SETTINGS_FILE);
        settings.store_component(&self.log_ui);
        settings.store_component(&self.notifications);
        settings.store_component(&self.tts);
        settings.save_file();
    }

    pub fn get() -> Option<&'static mut Plugin> {
        if let Ok(mut plugin) = crate::PLUGIN.lock() {
            plugin.as_mut()
        } else {
            None
        }
    }

    pub fn render_windows(&mut self, ui: &Ui, not_loading_or_character_selection: bool) {
        if not_loading_or_character_selection {
            self.log_ui.render(ui, &self.tracking);
        }
    }

    pub fn key_event(&mut self, key: usize, key_down: bool, prev_key_down: bool) -> bool {
        if key_down && !prev_key_down {
            if let Some(settings) = &self.log_ui.settings {
                if key == settings.toggle_key {
                    self.log_ui.toggle_visibility();
                    return true;
                }
            }
        }
        false
    }
}

impl<'a> Default for Plugin<'a> {
    fn default() -> Self {
        Self::new()
    }
}
