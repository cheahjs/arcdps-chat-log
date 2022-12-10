use crate::MUMBLE_LINK;

use super::TextToSpeech;
use arcdps::extras::message::ChatMessageInfo;
use log::error;

impl TextToSpeech {
    pub fn process_message(&mut self, message: &ChatMessageInfo, self_account_name: &str) {
        if !self.settings.play_on_self_message && message.account_name == self_account_name {
            return;
        }
        if !self.settings.play_on_all_new_messages {
            return;
        }
        match MUMBLE_LINK.lock().unwrap().tick() {
            Some(linked_mem) => {
                if linked_mem.context.is_in_combat() && !self.settings.play_in_combat {
                    return;
                }
                if !linked_mem.context.is_in_combat() && !self.settings.play_out_of_combat {
                    return;
                }
            }
            None => {
                return;
            }
        };
        self.play(message.text);
    }

    pub fn play(&mut self, text: &str) {
        if let Some(tts_instance) = &mut self.tts {
            match tts_instance.speak(text, false) {
                Ok(_) => {}
                Err(err) => {
                    error!("failed to speak tts: {}", err);
                }
            }
        }
    }
}
