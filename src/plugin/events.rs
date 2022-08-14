use std::collections::hash_map::Entry;

use arcdps::extras::{message::ChatMessageInfo, ExtrasAddonInfo, UserInfoIter, UserRole};
use log::error;

use super::{state::ExtrasState, Plugin};

impl Plugin {
    pub fn process_message(
        &mut self,
        chat_message_info: &ChatMessageInfo,
    ) -> Result<(), anyhow::Error> {
        if let Err(err) = self
            .notifications
            .process_message(chat_message_info, &self.self_account_name)
        {
            error!("failed to process message for notifications: {}", err);
        }
        self.log_ui.buffer.process_message(chat_message_info);
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

    pub fn squad_update(&mut self, users: UserInfoIter) {
        for user_update in users {
            let account_name = match user_update.account_name {
                Some(x) => arcdps::strip_account_prefix(x),
                None => continue,
            };
            match user_update.role {
                UserRole::SquadLeader | UserRole::Lieutenant | UserRole::Member => {
                    let entry = self.squad_members.entry(account_name.to_string());
                    match entry {
                        Entry::Occupied(entry) => {
                            let user = entry.into_mut();

                            if user_update.ready_status && !user.ready_status {
                                self.log_ui
                                    .buffer
                                    .insert_squad_update(format!("{} readied up", account_name));
                            }
                            if !user_update.ready_status && user.ready_status {
                                self.log_ui
                                    .buffer
                                    .insert_squad_update(format!("{} unreadied", account_name));
                            }
                            if user_update.role != user.role {
                                self.log_ui.buffer.insert_squad_update(format!(
                                    "{} changed roles from {} to {}",
                                    account_name, user.role, user_update.role
                                ));
                            }
                            if user_update.subgroup != user.subgroup {
                                self.log_ui.buffer.insert_squad_update(format!(
                                    "{} moved from subgroup {} to {}",
                                    account_name,
                                    user.subgroup + 1,
                                    user_update.subgroup + 1
                                ));
                            }

                            user.ready_status = user_update.ready_status;
                            user.role = user_update.role;
                            user.subgroup = user_update.subgroup;
                        }
                        Entry::Vacant(entry) => {
                            self.log_ui.buffer.insert_squad_update(format!(
                                "{} joined the squad as {}",
                                account_name, user_update.role
                            ));
                            entry.insert(user_update.into());
                        }
                    };
                }
                UserRole::None => {
                    if account_name == self.self_account_name {
                        self.log_ui
                            .buffer
                            .insert_squad_update(format!("{} (self) left the squad", account_name));
                        self.squad_members.clear();
                    } else {
                        let _result = self.squad_members.remove(account_name);
                        self.log_ui
                            .buffer
                            .insert_squad_update(format!("{} left the squad", account_name));
                    }
                }
                UserRole::Invited => {
                    self.log_ui
                        .buffer
                        .insert_squad_update(format!("{} invited to squad", account_name));
                }
                UserRole::Applied => {
                    self.log_ui
                        .buffer
                        .insert_squad_update(format!("{} applied to join squad", account_name));
                }
                UserRole::Invalid => {}
            };
        }
    }
}
