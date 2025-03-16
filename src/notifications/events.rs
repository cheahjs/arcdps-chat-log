use arcdps::extras::message::Message;
use arcdps::extras::message::SquadMessage;

use crate::MUMBLE_LINK;

use super::Notifications;
use crate::notifications::NotificationsSettings;

pub struct NotificationEvents {
    pub settings: NotificationSettings,
}

impl NotificationEvents {
    pub fn new(settings: NotificationSettings) -> Self {
        Self { settings }
    }

    pub fn process_message(&self, message: &SquadMessage, self_account_name: &str) -> bool {
        if !self.settings.ping_on_self_message && message.account_name() == self_account_name {
            return false;
        }

        true
    }
}

impl Notifications {
    pub fn process_message(
        &mut self,
        message: &Message,
        self_account_name: &str,
    ) -> Result<(), anyhow::Error> {
        if !self.settings.ping_on_self_message && message.account_name() == self_account_name {
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
