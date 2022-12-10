use arc_util::settings::HasSettings;
use serde::{Deserialize, Serialize};

use super::TextToSpeech;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TextToSpeechSettings {
    pub voice_id: String,
    pub rate: f32,
    pub pitch: f32,
    pub volume: i32,
    pub play_on_all_new_messages: bool,
    pub play_on_self_message: bool,
    pub play_in_combat: bool,
    pub play_out_of_combat: bool,
}

impl TextToSpeechSettings {
    pub fn new() -> Self {
        Self {
            voice_id: String::new(),
            rate: 1.,
            pitch: 1.,
            volume: 100,
            play_on_all_new_messages: false,
            play_on_self_message: false,
            play_in_combat: true,
            play_out_of_combat: true,
        }
    }
}

impl Default for TextToSpeechSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl HasSettings for TextToSpeech {
    type Settings = TextToSpeechSettings;

    const SETTINGS_ID: &'static str = "tts";

    fn current_settings(&self) -> Self::Settings {
        self.settings.clone()
    }

    fn load_settings(&mut self, loaded: Self::Settings) {
        self.settings = loaded;
    }
}
