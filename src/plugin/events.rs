use std::collections::{hash_map::Entry, HashSet};

use arc_util::tracking::Player;
use arcdps::{
    extras::{message::ChatMessageInfo, ExtrasAddonInfo, UserInfoIter, UserRole},
    Agent, CombatEvent,
};
use log::error;

use crate::logui::buffer::LogPart;

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
        if self.log_ui.settings.log_enabled {
            if let Some(chat_database) = self.chat_database.as_mut() {
                chat_database.process_message(chat_message_info)?;
            }
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

    pub fn combat(
        &mut self,
        event: Option<CombatEvent>,
        src: Option<Agent>,
        dst: Option<Agent>,
        _skill_name: Option<&'static str>,
        _id: u64,
        _revision: u64,
    ) {
        match event {
            Some(_) => {}
            None => {
                if let Some(src) = src {
                    // tracking change
                    if src.elite == 0 {
                        if src.prof != 0 {
                            // agent added
                            if let Some(player) =
                                dst.and_then(|dst| Player::from_tracking_change(src, dst))
                            {
                                self.tracker.add_arc_player(&player);
                                let mut parts: Vec<LogPart> = Vec::new();
                                if player.account == self.self_account_name {
                                    parts.push(LogPart::new_no_color(
                                        "You have joined an instance as ",
                                    ));
                                    parts.push(LogPart::new(
                                        &player.character,
                                        Some(&player.account),
                                        None,
                                    ));
                                } else {
                                    parts.push(LogPart::new(
                                        &player.character,
                                        Some(&player.account),
                                        None,
                                    ));
                                    parts.push(LogPart::new_no_color(" has joined your instance"));
                                }
                                self.log_ui.buffer.insert_squad_update_parts(&mut parts);
                            }
                        } else {
                            // agent removed
                            let player = self.tracker.remove_arc_player(src.id);
                            if let Some(player) = player {
                                let mut parts: Vec<LogPart> = Vec::new();
                                if player.account == self.self_account_name {
                                    parts.push(LogPart::new_no_color(
                                        "You have left an instance as ",
                                    ));
                                    parts.push(LogPart::new(
                                        &player.character,
                                        Some(&player.account),
                                        None,
                                    ));
                                } else {
                                    parts.push(LogPart::new(
                                        &player.character,
                                        Some(&player.account),
                                        None,
                                    ));
                                    parts.push(LogPart::new_no_color(" has left your instance"));
                                }
                                self.log_ui.buffer.insert_squad_update_parts(&mut parts);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn squad_update(&mut self, users: UserInfoIter) {
        for user_update in users {
            let account_name = match user_update.account_name {
                Some(x) => arcdps::strip_account_prefix(x),
                None => continue,
            };
            match self
                .log_ui
                .buffer
                .account_cache
                .entry(account_name.to_owned())
            {
                Entry::Occupied(_) => {}
                Entry::Vacant(entry) => {
                    entry.insert(HashSet::new());
                }
            };
            match user_update.role {
                UserRole::SquadLeader | UserRole::Lieutenant | UserRole::Member => {
                    let old_info = self.tracker.add_extras_player(&user_update.clone().into());
                    match old_info {
                        Some(old_info) => {
                            if user_update.ready_status && !old_info.ready_status {
                                self.log_ui
                                    .buffer
                                    .insert_squad_update(format!("{} readied up", account_name));
                            }
                            if !user_update.ready_status && old_info.ready_status {
                                self.log_ui
                                    .buffer
                                    .insert_squad_update(format!("{} unreadied", account_name));
                            }
                            if user_update.role != old_info.role {
                                self.log_ui.buffer.insert_squad_update(format!(
                                    "{} changed roles from {} to {}",
                                    account_name, old_info.role, user_update.role
                                ));
                            }
                            if user_update.subgroup != old_info.subgroup {
                                self.log_ui.buffer.insert_squad_update(format!(
                                    "{} moved from subgroup {} to {}",
                                    account_name,
                                    old_info.subgroup + 1,
                                    user_update.subgroup + 1
                                ));
                            }
                        }
                        None => {
                            self.log_ui.buffer.insert_squad_update(format!(
                                "{} joined the squad as {}",
                                account_name, user_update.role
                            ));
                        }
                    };
                }
                UserRole::None => {
                    if account_name == self.self_account_name {
                        self.log_ui
                            .buffer
                            .insert_squad_update(format!("{} (self) left the squad", account_name));
                        self.tracker.clear();
                    } else {
                        let _result = self.tracker.remove_extras_player(&user_update.into());
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
