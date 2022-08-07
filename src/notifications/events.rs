use arcdps::extras::message::ChatMessageInfo;

use crate::{mumblelink::LinkedMem, MUMBLE_LINK};

use super::Notifications;

impl Notifications {
    pub fn process_message(&self, _message: &ChatMessageInfo) -> Result<(), anyhow::Error> {
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

    pub fn ping(&self) {
        if let Some(ping_track) = &self.ping_track {
            ping_track.play(self.settings.ping_volume);
        }
    }
}
