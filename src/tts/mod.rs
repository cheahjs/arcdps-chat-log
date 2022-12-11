use anyhow::Context;
use tts::{Tts, Voice};

use self::settings::TextToSpeechSettings;

mod events;
mod settings;

pub struct TextToSpeech {
    pub settings: TextToSpeechSettings,
    pub tts: Option<Tts>,
    voice_cache: Option<Vec<Voice>>,
}

impl TextToSpeech {
    pub fn new() -> Self {
        Self {
            settings: TextToSpeechSettings::new(),
            tts: None,
            voice_cache: None,
        }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        self.tts = Some(Tts::default().context("failed to init tts")?);
        self.update_settings()
            .context("failed to set voice settings")?;
        // populate voice cache
        let _ = self.voices();
        Ok(())
    }

    pub fn update_settings(&mut self) -> anyhow::Result<()> {
        if let Some(tts_instance) = &mut self.tts {
            tts_instance.set_pitch(self.settings.pitch)?;
            tts_instance.set_rate(self.settings.rate)?;
            tts_instance.set_volume(self.settings.volume as f32 / 100.)?;
            tts_instance.set_silence(self.settings.silence_between_messages)?;
            if !self.settings.voice_id.is_empty() {
                let voices = tts_instance.voices()?;
                for v in voices {
                    if v.id() == self.settings.voice_id {
                        tts_instance.set_voice(&v)?
                    }
                }
            }
            // Speak to force an update
            tts_instance.speak("", false)?;
        }
        Ok(())
    }

    pub fn voices(&mut self) -> Option<&Vec<Voice>> {
        if self.voice_cache.is_none() {
            self.voice_cache = self
                .tts
                .as_ref()
                .map(|tts_instance| tts_instance.voices())
                .and_then(Result::ok);
        }
        self.voice_cache.as_ref()
    }

    pub fn current_voice(&self) -> Option<Voice> {
        self.tts
            .as_ref()
            .map(|tts_instance| tts_instance.voice())
            .and_then(Result::ok)
            .flatten()
    }

    pub fn get_display_name_for_voice(voice: &Voice) -> String {
        format!("{} - {}", voice.name(), voice.language().full_language())
    }

    pub fn min_rate() -> f32 {
        0.5
    }

    pub fn max_rate() -> f32 {
        6.0
    }

    pub fn min_pitch() -> f32 {
        0.
    }

    pub fn max_pitch() -> f32 {
        2.
    }
}
