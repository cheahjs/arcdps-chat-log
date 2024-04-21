use arc_util::settings::HasSettings;
use serde::{Deserialize, Serialize};

use super::TextToSpeech;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TextToSpeechSettings {
    #[serde(default = "u32::default")]
    pub version: u32,
    pub voice_id: String,
    pub rate: f32,
    pub pitch: f32,
    pub volume: i32,
    pub silence_between_messages: i32,
    pub play_on_all_new_messages: bool,
    pub play_on_self_message: bool,
    pub play_in_combat: bool,
    pub play_out_of_combat: bool,
    #[serde(default = "default_as_true")]
    pub play_squad_messages: bool,
    #[serde(default = "default_as_true")]
    pub play_squad_broadcasts: bool,
    #[serde(default = "default_as_true")]
    pub play_party_messages: bool,
}

fn default_as_true() -> bool {
    true
}

impl TextToSpeechSettings {
    pub fn new() -> Self {
        Self {
            version: 2,
            voice_id: String::new(),
            rate: 1.,
            pitch: 1.,
            volume: 100,
            silence_between_messages: 100,
            play_on_all_new_messages: false,
            play_on_self_message: false,
            play_in_combat: true,
            play_out_of_combat: true,
            play_squad_messages: true,
            play_squad_broadcasts: true,
            play_party_messages: true,
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
        // Perform settings migration
        if self.settings.version < 2 {
            // Version 2 introduced the ability to toggle playing squad or party messages
            self.settings.play_squad_messages = true;
            self.settings.play_squad_broadcasts = true;
            self.settings.play_party_messages = true;
        }
        self.settings.version = 2;
    }
}
