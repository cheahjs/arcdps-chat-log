use std::sync::{Arc, Mutex};

use log::error;

/// RAII guard that resets the `refreshing` flag to false when dropped.
/// This ensures the flag is cleared even on panic or early return.
struct RefreshingGuard {
    flag: Arc<Mutex<bool>>,
}

impl Drop for RefreshingGuard {
    fn drop(&mut self) {
        match self.flag.lock() {
            Ok(mut refreshing) => *refreshing = false,
            Err(mut poisoned) => **poisoned.get_mut() = false,
        }
    }
}

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
    pub audio_devices: Arc<Mutex<Vec<String>>>,
    pub refreshing_audio_devices: Arc<Mutex<bool>>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            extras_state: ExtrasState::Unknown,
            notifications_state: NotificationsState::Unknown,
            mumblelink_state: MumbleLinkState::Unknown,
            tts_state: TtsState::Unknown,
            audio_devices: Arc::new(Mutex::new(Vec::new())),
            refreshing_audio_devices: Arc::new(Mutex::new(false)),
        }
    }

    pub fn refresh_audio_devices(&self) {
        let refreshing = self.refreshing_audio_devices.clone();
        let devices_mutex = self.audio_devices.clone();

        {
            let mut refreshing = refreshing.lock().unwrap();
            if *refreshing {
                return;
            }
            *refreshing = true;
        }

        std::thread::spawn(move || {
            // Drop guard ensures `refreshing` is reset to false even if cpal panics
            // or we return early, preventing the UI from being stuck on "Refreshing...".
            let _guard = RefreshingGuard { flag: refreshing };

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                use rodio::cpal::traits::{DeviceTrait, HostTrait};
                let host = rodio::cpal::default_host();
                if let Ok(devices) = host.output_devices() {
                    let mut new_devices: Vec<String> =
                        devices.filter_map(|d| d.name().ok()).collect();
                    new_devices.sort();
                    let mut devices = devices_mutex.lock().unwrap();
                    *devices = new_devices;
                }
            }));

            if let Err(err) = result {
                error!("panic while refreshing audio devices: {:?}", err);
            }
        });
    }
}
