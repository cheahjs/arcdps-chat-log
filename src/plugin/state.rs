#[derive(Debug)]
pub enum ExtrasState {
    Loaded,
    Incompatible,
    Unknown,
}

#[derive(Debug)]
pub enum NotificationsState {
    Loaded,
    Errored,
    Unknown,
}

#[derive(Debug)]
pub enum MumbleLinkState {
    Loaded(String),
    Errored,
    Unknown,
}

#[derive(Debug)]
pub enum TtsState {
    Loaded,
    Errored,
    Unknown,
}

use crate::db::insert::ClearTimeRange;

#[derive(Debug)]
pub struct UiState {
    pub extras_state: ExtrasState,
    pub notifications_state: NotificationsState,
    pub mumblelink_state: MumbleLinkState,
    pub tts_state: TtsState,
    pub clear_data_selected_range: ClearTimeRange,
    pub clear_data_confirm_popup: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            extras_state: ExtrasState::Unknown,
            notifications_state: NotificationsState::Unknown,
            mumblelink_state: MumbleLinkState::Unknown,
            tts_state: TtsState::Unknown,
            clear_data_selected_range: ClearTimeRange::LastHour,
            clear_data_confirm_popup: false,
        }
    }
}
