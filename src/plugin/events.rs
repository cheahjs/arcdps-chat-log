use arcdps::extras::{message::ChatMessageInfo, ExtrasAddonInfo};
use log::error;

use super::{state::ExtrasState, Plugin};

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

    pub fn extras_init(&mut self, addon_info: &ExtrasAddonInfo, account_name: Option<&str>) {
        if addon_info.is_compatible() && addon_info.supports_chat_message_callback() {
            self.ui_state.extras_state = ExtrasState::Loaded;
        } else {
            self.ui_state.extras_state = ExtrasState::Incompatible;
        }
        if let Some(account_name) = account_name {
            self.self_account_name = account_name.to_string();
        }
    }
}
