use arcdps::extras::message::ChatMessageInfo;

use crate::MUMBLE_LINK;

use super::Notifications;

impl Notifications {
    pub fn process_message(
        &mut self,
        message: &ChatMessageInfo,
        self_account_name: &str,
    ) -> Result<(), anyhow::Error> {
        if !self.settings.ping_on_self_message && message.account_name == self_account_name {
            return Ok(());
        }
        if !self.settings.ping_on_all_new_messages {
            return Ok(());
        }
        match MUMBLE_LINK.lock().unwrap().tick() {
            Some(linked_mem) => {
                if linked_mem.context.is_in_combat() && !self.settings.ping_in_combat {
                    return Ok(());
                }
                if !linked_mem.context.is_in_combat() && !self.settings.ping_out_of_combat {
                    return Ok(());
                }
            }
            None => {
                return Ok(());
            }
        };
        self.ping();
        Ok(())
    }

    pub fn ping(&mut self) {
        if !self.ping_track.is_valid() {
            return;
        }
        self.ping_track.set_volume(self.settings.ping_volume);
        crate::AUDIO_PLAYER
            .lock()
            .unwrap()
            .play_track(&self.ping_track);
    }
}
