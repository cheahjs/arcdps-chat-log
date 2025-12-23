use anyhow::Context;
use arc_util::ui::Component;
use arcdps::imgui::Ui;

use crate::audio::{sounds, AudioTrack};

use self::settings::NotificationsSettings;

mod events;
mod settings;

#[derive(Debug)]
pub struct Notifications {
    pub settings: NotificationsSettings,
    pub ping_track: AudioTrack,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            settings: NotificationsSettings::new(),
            ping_track: AudioTrack::new(),
        }
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        crate::AUDIO_PLAYER
            .lock()
            .unwrap()
            .set_device(self.settings.audio_device.clone());
        self.update_ping_track()?;
        Ok(())
    }

    pub fn update_ping_track(&mut self) -> anyhow::Result<()> {
        self.ping_track
            .load_from_path(
                &self.settings.ping_sound_path,
                sounds::DEFAULT_PING,
                self.settings.ping_volume,
            )
            .context("failed to load ping track")?;
        Ok(())
    }
}

impl Component<()> for Notifications {
    fn render(&mut self, _ui: &Ui, _props: ()) {}
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}
