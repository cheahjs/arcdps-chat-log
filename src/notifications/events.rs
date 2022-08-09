use arcdps::extras::message::ChatMessageInfo;

use crate::{mumblelink::LinkedMem, MUMBLE_LINK};

use super::Notifications;

impl Notifications {
    pub fn process_message(&mut self, _message: &ChatMessageInfo) -> Result<(), anyhow::Error> {
        if !self.settings.ping_on_all_new_messages {
            return Ok(());
        }
        let mumble_link: LinkedMem = MUMBLE_LINK.lock().unwrap().tick();
        if mumble_link.context.is_in_combat() && !self.settings.ping_in_combat {
            return Ok(());
        }
        if !mumble_link.context.is_in_combat() && !self.settings.ping_out_of_combat {
            return Ok(());
        }
        self.ping();
        Ok(())
    }

    pub fn ping(&mut self) {
        if let Some(ping_track) = &mut self.ping_track {
            ping_track.set_volume(self.settings.ping_volume);
            crate::AUDIO_PLAYER.lock().unwrap().play_track(ping_track);
        }
    }
}
