use arcdps::extras::message::ChatMessageInfo;

use super::Notifications;

impl Notifications {
    pub fn process_message(&self, _message: &ChatMessageInfo) -> Result<(), anyhow::Error> {
        if !self.settings.ping_on_all_new_messages {
            // TODO: implement combat things
            return Ok(());
        }
        self.ping();
        Ok(())
    }

    pub fn ping(&self) {
        if let Some(ping_track) = &self.ping_track {
            ping_track.play(self.settings.ping_volume);
        }
    }
}
