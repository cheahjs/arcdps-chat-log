use crate::MUMBLE_LINK;
use arcdps::extras::ChannelType;
use arcdps::extras::message::{SquadMessage};

use super::TextToSpeech;
use arcdps::extras::message::Message;
use log::error;
use regex::Regex;

impl TextToSpeech {
    pub fn process_message(&self, message: &SquadMessage, self_account_name: &str) -> bool {
        if !self.settings.play_on_self_message && message.account_name() == self_account_name {
            return false;
        }

        if !self.settings.play_party_messages {
            if message.channel() == ChannelType::Party {
                return false;
            }
        }

        if !self.settings.play_subgroup_messages {
            if message.channel() == ChannelType::Squad && message.subgroup() != 255 {
                return false;
            }
        }

        if !self.settings.play_squad_messages
            && message.channel() == ChannelType::Squad
            && message.subgroup() == 255
            && !message.is_broadcast()
        {
            return false;
        }

        if message.is_broadcast() && !self.settings.play_squad_broadcasts {
            return false;
        }

        if !self.settings.play_whispers {
            if message.channel() == ChannelType::Whisper {
                return false;
            }
        }

        if !self.settings.play_say {
            if message.channel() == ChannelType::Say {
                return false;
            }
        }

        self.play(&Self::sanitize_message(message.text()));
        true
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
        re.replace_all(message, "chatcode").to_string()
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
