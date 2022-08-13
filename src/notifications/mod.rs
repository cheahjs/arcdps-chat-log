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
    ping_track: Option<AudioTrack>,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            settings: NotificationsSettings::new(),
            ping_track: None,
        }
    }

    pub fn load(&mut self) -> Result<(), anyhow::Error> {
        let mut ping_track = AudioTrack::new();
        ping_track
            .load_from_path(
                &self.settings.ping_sound_path,
                sounds::DEFAULT_PING,
                self.settings.ping_volume,
            )
            .context("failed to load ping track")?;
        self.ping_track = Some(ping_track);
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
