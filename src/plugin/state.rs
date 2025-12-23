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

#[derive(Debug)]
pub struct UiState {
    pub extras_state: ExtrasState,
    pub notifications_state: NotificationsState,
    pub mumblelink_state: MumbleLinkState,
    pub tts_state: TtsState,
    pub audio_devices: Vec<String>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            extras_state: ExtrasState::Unknown,
            notifications_state: NotificationsState::Unknown,
            mumblelink_state: MumbleLinkState::Unknown,
            tts_state: TtsState::Unknown,
            audio_devices: Vec::new(),
        }
    }

    pub fn refresh_audio_devices(&mut self) {
        use rodio::cpal::traits::{DeviceTrait, HostTrait};
        let host = rodio::cpal::default_host();
        if let Ok(devices) = host.output_devices() {
            self.audio_devices = devices
                .filter_map(|d| d.name().ok())
                .collect();
            self.audio_devices.sort();
        }
    }
}
