use crate::MUMBLE_LINK;
use arcdps::extras::ChannelType;

use super::TextToSpeech;
use arcdps::extras::message::ChatMessageInfo;
use log::error;
use regex::Regex;

impl TextToSpeech {
    pub fn process_message(&mut self, message: &ChatMessageInfo, self_account_name: &str) {
        if !self.settings.play_on_self_message && message.account_name == self_account_name {
            return;
        }
        if !self.settings.play_on_all_new_messages {
            return;
        }
        if !self.settings.play_party_messages {
            if message.channel_type == ChannelType::Party {
                return;
            }
            if message.channel_type == ChannelType::Squad && message.subgroup != 255 {
                return;
            }
        }
        if !self.settings.play_squad_messages
            && message.channel_type == ChannelType::Squad
            && message.subgroup == 255
            && !message.is_broadcast
        {
            return;
        }
        if message.is_broadcast && !self.settings.play_squad_broadcasts {
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
        self.play(&Self::sanitize_message(message.text));
    }

    pub fn play(&mut self, text: &str) {
        if let Some(tts_instance) = &mut self.tts {
            match tts_instance.speak(text, false) {
                Ok(_) => {}
                Err(err) => {
                    error!("failed to speak tts: {:#}", err);
                }
            }
        }
    }

    fn sanitize_message(message: &str) -> String {
        // Clean up chat codes until we have an implementation to strip it
        let re = Regex::new(r"\[&[A-Za-z0-9+/=]+\]").unwrap();
        return re.replace_all(message, "chatcode").to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::TextToSpeech;

    #[test]
    fn sanitize_message() {
        assert_eq!(
            "this message has no chat codes",
            TextToSpeech::sanitize_message("this message has no chat codes")
        );
        assert_eq!(
            "incomplete chat code [asdasdsa]",
            TextToSpeech::sanitize_message("incomplete chat code [asdasdsa]")
        );
        assert_eq!(
            "single chatcode chatcode",
            TextToSpeech::sanitize_message("single chatcode [&AgGqtgAA]")
        );
        assert_eq!(
            "multiple chatcode here chatcode and here chatcode",
            TextToSpeech::sanitize_message(
                "multiple chatcode here [&AgGqtgAA] and here [&AgGqtgAA]"
            )
        );
    }
}
