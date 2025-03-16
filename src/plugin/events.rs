use std::sync::Arc;

use arcdps::{
    extras::{
        ExtrasAddonInfo,
        message::SquadMessage,
        user::{UserInfo, UserInfoIter, UserRole},
    },
    Event as CombatEvent,
    Agent, StateChange,
};

use crate::logui::buffer::LogPart;
use crate::plugin::Plugin;

pub fn extras_init(addon_info: &ExtrasAddonInfo) -> bool {
    if addon_info.version() >= 0x02 {
        true
    } else {
        log::error!("Extras version too old");
        false
    }
}

pub fn extras_chat_callback(msg: &SquadMessage) {
    if let Some(plugin) = Plugin::get() {
        if let Some(tracking) = &plugin.tracking {
            tracking.process_message(msg);
        }
        if let Some(notifications) = &plugin.notifications {
            notifications.process_message(msg);
        }
    }
}

pub fn combat(ev: Option<&CombatEvent>, src: Option<&Agent>, dst: Option<&Agent>, skill_name: Option<&str>, id: u64, revision: u64) {
    if let Some(plugin) = Plugin::get() {
        if let Some(tracking) = &plugin.tracking {
            if let Some(ev) = ev {
                if ev.is_statechange == StateChange::EnterCombat as i32 {
                    if let Some(src) = src {
                        if src.is_self == 1 {
                            tracking.log_ui.buffer.push(LogPart::EnterCombat);
                        }
                    }
                }
            }
        }
    }
}

pub fn squad_update(user_update: &UserInfo, self_update: bool) {
    if let Some(plugin) = Plugin::get() {
        if let Some(tracking) = &plugin.tracking {
            let mut log_parts = Vec::new();

            if self_update {
                if let Some(role) = user_update.role() {
                    log_parts.push(LogPart::RoleChange {
                        account_name: user_update.account_name().to_string(),
                        role,
                    });
                }

                if let Some(ready) = user_update.ready() {
                    log_parts.push(LogPart::ReadyStatus {
                        account_name: user_update.account_name().to_string(),
                        ready,
                    });
                }

                if let Some(subgroup) = user_update.subgroup() {
                    log_parts.push(LogPart::SubgroupMove {
                        account_name: user_update.account_name().to_string(),
                        subgroup,
                    });
                }
            }

            for part in log_parts {
                tracking.log_ui.buffer.push(part);
            }
        }
    }
}
