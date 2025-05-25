use std::{
    collections::VecDeque,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
};

use crate::{db::query::ArchivedMessage, db::ChatDatabase};

use self::{buffer::LogBuffer, settings::ChatLogSettings};
use windows::System::VirtualKey;

pub mod buffer;
mod settings;
mod ui;

#[derive(Debug)]
struct LocalProps {
    pub account_filter: String,
    pub text_filter: String,
    pub account_width: f32,
    pub search_input: String,
    pub search_results: VecDeque<ArchivedMessage>,
    pub search_status: String,
    pub search_result_receiver: Option<Receiver<Vec<ArchivedMessage>>>,
}

impl LocalProps {
    pub fn new() -> Self {
        Self {
            account_filter: String::new(),
            text_filter: String::new(),
            account_width: 100.0,
            search_input: String::new(),
            search_results: VecDeque::new(),
            search_status: String::new(),
            search_result_receiver: None,
        }
    }
}

pub struct LogUi {
    pub settings: ChatLogSettings,
    pub buffer: LogBuffer,
    pub chat_database: Option<Arc<Mutex<ChatDatabase>>>,
    ui_props: LocalProps,
}

impl LogUi {
    pub const DEFAULT_HOTKEY: u32 = VirtualKey::J.0 as u32;

    pub fn new() -> Self {
        Self {
            settings: ChatLogSettings::new(),
            buffer: LogBuffer::new(),
            chat_database: None,
            ui_props: LocalProps::new(),
        }
    }

    pub fn update_settings(&mut self) {
        self.buffer.colors = self.settings.color_settings;
    }
}

impl Default for LogUi {
    fn default() -> Self {
        Self::new()
    }
}
