use std::sync::{Arc, Mutex};

use crate::db::{query::SearchResultMessage, ChatDatabase};

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
}

impl LocalProps {
    pub fn new() -> Self {
        Self {
            account_filter: String::new(),
            text_filter: String::new(),
            account_width: 100.0,
        }
    }
}

/// State for the search UI
#[derive(Debug)]
pub struct SearchUiState {
    /// Current search query text
    pub search_text: String,
    /// Filter by account name
    pub account_filter: String,
    /// Filter by channel type (index into CHANNEL_TYPES)
    pub channel_type_index: usize,
    /// Whether the search window is open
    pub window_open: bool,
    /// Counter for generating unique search IDs
    pub search_id_counter: u64,
    /// Current search results (cached from db for display)
    pub cached_results: Vec<SearchResultMessage>,
    /// Whether there are more results to load
    pub has_more: bool,
    /// Current offset for pagination
    pub current_offset: u32,
    /// Whether a search is in progress
    pub is_searching: bool,
    /// Last error message
    pub error_message: Option<String>,
    /// The search ID of the cached results
    pub cached_search_id: u64,
}

/// Channel type options for the filter dropdown
pub const CHANNEL_TYPES: &[(&str, Option<&str>)] = &[
    ("All", None),
    ("Squad", Some("Squad")),
    ("Party", Some("Party")),
];

impl SearchUiState {
    pub fn new() -> Self {
        Self {
            search_text: String::new(),
            account_filter: String::new(),
            channel_type_index: 0,
            window_open: false,
            search_id_counter: 0,
            cached_results: Vec::new(),
            has_more: false,
            current_offset: 0,
            is_searching: false,
            error_message: None,
            cached_search_id: 0,
        }
    }

    /// Generate a new unique search ID
    pub fn next_search_id(&mut self) -> u64 {
        self.search_id_counter += 1;
        self.search_id_counter
    }

    /// Clear current results for a new search
    pub fn clear_for_new_search(&mut self) {
        self.cached_results.clear();
        self.has_more = false;
        self.current_offset = 0;
        self.error_message = None;
    }
}

impl Default for SearchUiState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LogUi {
    pub settings: ChatLogSettings,
    pub buffer: LogBuffer,
    pub chat_database: Option<Arc<Mutex<ChatDatabase>>>,
    ui_props: LocalProps,
    pub search_state: SearchUiState,
}

impl LogUi {
    pub const DEFAULT_HOTKEY: u32 = VirtualKey::J.0 as u32;

    pub fn new() -> Self {
        Self {
            settings: ChatLogSettings::new(),
            buffer: LogBuffer::new(),
            chat_database: None,
            ui_props: LocalProps::new(),
            search_state: SearchUiState::new(),
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
