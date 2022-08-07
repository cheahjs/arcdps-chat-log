use anyhow::Context;
use arc_util::ui::Component;
use arcdps::imgui::Ui;

use self::{audio::AudioTrack, settings::NotificationsSettings};

mod audio;
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
            .load_from_path(&self.settings.ping_sound_path, audio::DEFAULT_PING)
            .context("failed to load ping track")?;
        self.ping_track = Some(ping_track);
        Ok(())
    }
}

impl Component<'_> for Notifications {
    type Props = ();

    fn render(&mut self, _ui: &Ui, _props: &Self::Props) {}
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}
