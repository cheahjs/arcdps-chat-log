use arcdps::extras::message::ChatMessageInfo;

use super::Plugin;

impl Plugin {
    pub fn process_message(
        &mut self,
        chat_message_info: &ChatMessageInfo,
    ) -> Result<(), anyhow::Error> {
        if !self.log_ui.settings.log_enabled {
            return Ok(());
        }
        if let Some(chat_database) = self.chat_database.as_mut() {
            chat_database.process_message(chat_message_info)?;
        }
        Ok(())
    }
}
