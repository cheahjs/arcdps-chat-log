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
pub struct UiState {
    pub extras_state: ExtrasState,
    pub notifications_state: NotificationsState,
    pub mumblelink_state: MumbleLinkState,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            extras_state: ExtrasState::Unknown,
            notifications_state: NotificationsState::Unknown,
            mumblelink_state: MumbleLinkState::Unknown,
        }
    }
}
