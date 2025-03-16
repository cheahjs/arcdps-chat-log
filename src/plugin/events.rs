use arc_util::tracking::Player;
use arcdps::{
    extras::{ExtrasAddonInfo, Message, SquadMessageOwned, UserInfoIter, UserInfoOwned, UserRole},
    Agent, Event, StateChange,
};
use log::{debug, error};

use crate::logui::buffer::LogPart;

use super::{state::ExtrasState, Plugin};

impl Plugin {
    pub fn process_message(&mut self, message: &Message) -> Result<(), anyhow::Error> {
        let Message::Squad(squad_message) = message else {
            // TODO: handle NPC messages
            return Ok(());
        };
        let squad_message_owned: SquadMessageOwned = (*squad_message).into();
        self.tracker.add_player_from_message(squad_message);
        if let Err(err) = self
            .notifications
            .process_message(squad_message, &self.self_account_name)
        {
            error!("failed to process message for notifications: {:#}", err);
        }
        self.tts
            .process_message(&squad_message_owned, &self.self_account_name);
        self.log_ui.buffer.process_message(&squad_message_owned);
        if self.log_ui.settings.log_enabled {
            if let Some(chat_database) = self.chat_database.as_mut() {
                chat_database
                    .lock()
                    .unwrap()
                    .process_message(&squad_message_owned)?;
            }
        }
        Ok(())
    }

    pub fn extras_init(&mut self, addon_info: &ExtrasAddonInfo, account_name: Option<&str>) {
        let version = addon_info.version();
        debug!("extras version: {:?}", version);
        debug!("supports chat message2: {:?}", version.supports_chat_message2());
        debug!("supports squad chat message: {:?}", version.supports_squad_chat_message());
        debug!("is compatible: {:?}", version.is_compatible());
        if version.is_compatible() && version.supports_chat_message2() {
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
        event: Option<&Event>,
        src: Option<&Agent>,
        dst: Option<&Agent>,
        _skill_name: Option<&'static str>,
        _id: u64,
        _revision: u64,
    ) {
        if let Some(src) = src {
            match event {
                Some(event) => match event.get_statechange() {
                    StateChange::EnterCombat => {
                        if let Some(player) = self.tracker.get_arc_player(src.id) {
                            let mut parts: Vec<LogPart> = Vec::new();
                            if player.account == self.self_account_name {
                                parts.push(LogPart::new_no_color("You have entered combat as "));
                                parts.push(LogPart::new(
                                    &player.character,
                                    Some(&player.account),
                                    None,
                                    None,
                                ));
                            } else {
                                parts.push(LogPart::new(
                                    &player.character,
                                    Some(&player.account),
                                    None,
                                    None,
                                ));
                                parts.push(LogPart::new_no_color(" has entered combat"));
                            }
                            self.log_ui.buffer.insert_combat_update_parts(&mut parts);
                        }
                    }
                    StateChange::ExitCombat => {
                        if let Some(player) = self.tracker.get_arc_player(src.id) {
                            let mut parts: Vec<LogPart> = Vec::new();
                            if player.account == self.self_account_name {
                                parts.push(LogPart::new_no_color("You have left combat as "));
                                parts.push(LogPart::new(
                                    &player.character,
                                    Some(&player.account),
                                    None,
                                    None,
                                ));
                            } else {
                                parts.push(LogPart::new(
                                    &player.character,
                                    Some(&player.account),
                                    None,
                                    None,
                                ));
                                parts.push(LogPart::new_no_color(" has left combat"));
                            }
                            self.log_ui.buffer.insert_combat_update_parts(&mut parts);
                        }
                    }
                    _ => {}
                },
                None => {
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
                                        None,
                                    ));
                                } else {
                                    parts.push(LogPart::new(
                                        &player.character,
                                        Some(&player.account),
                                        None,
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
                                        None,
                                    ));
                                } else {
                                    parts.push(LogPart::new(
                                        &player.character,
                                        Some(&player.account),
                                        None,
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
            let account_name = match user_update.account_name() {
                Some(x) => arcdps::strip_account_prefix(x),
                None => continue,
            };
            let owned_user = UserInfoOwned {
                account_name: user_update.account_name().map(|x| x.to_string()),
                join_time: user_update.join_time,
                role: user_update.role,
                subgroup: user_update.subgroup,
                ready_status: user_update.ready_status,
            };
            match user_update.role {
                UserRole::SquadLeader | UserRole::Lieutenant | UserRole::Member => {
                    let old_info = self.tracker.add_extras_player(&owned_user);
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
                        let _result = self.tracker.remove_extras_player(&owned_user);
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
