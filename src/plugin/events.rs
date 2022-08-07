use arcdps::extras::message::ChatMessageInfo;
use log::error;

use super::Plugin;

impl Plugin {
    pub fn process_message(
        &mut self,
        chat_message_info: &ChatMessageInfo,
    ) -> Result<(), anyhow::Error> {
        if let Err(err) = self.notifications.process_message(chat_message_info) {
            error!("failed to process message for notifications: {}", err);
        }
        if !self.log_ui.settings.log_enabled {
            return Ok(());
        }
        if let Some(chat_database) = self.chat_database.as_mut() {
            chat_database.process_message(chat_message_info)?;
        }
        Ok(())
    }
}
