use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::sync::Arc;

use arc_util::tracking::Player;
use arcdps::{
    extras::{message::Message, UserInfo},
    strip_account_prefix,
};
use log::debug;

#[derive(Debug)]
pub struct PlayerInfo {
    pub character_name: Option<String>,
    pub extras: Option<Arc<UserInfo>>,
}

impl PlayerInfo {
    pub fn new(character_name: Option<String>, extras: Option<Arc<UserInfo>>) -> Self {
        Self {
            character_name,
            extras,
        }
    }

    pub fn new_from_extras(player: &UserInfo) -> Self {
        Self::new(None, Some(Arc::new(player.clone())))
    }
}

#[derive(Debug)]
pub struct Tracker {
    pub map: HashMap<String, PlayerInfo>,
    pub self_account_name: String,
}

impl Tracker {
    pub fn new(self_account_name: String) -> Self {
        Self {
            map: HashMap::new(),
            self_account_name,
        }
    }

    pub fn add_arc_player(&mut self, player: &Player) -> Option<Player> {
        debug!("adding arc player: {:?}", player);
        self.insert_name_into_cache(&player.account, Some(&player.character));
        match self.map.entry(player.account.to_owned()) {
            Entry::Occupied(entry) => {
                debug!("adding arc player into existing entry");
                let entry = entry.into_mut();
                let old_info = entry.character_name.as_ref().cloned();
                entry.character_name = Some(player.character.to_owned());
                return old_info;
            }
            Entry::Vacant(entry) => {
                debug!("adding arc player into new entry");
                entry.insert(PlayerInfo::new(Some(player.character.to_owned()), None));
            }
        }
        None
    }

    pub fn remove_arc_player(&mut self, id: usize) -> Option<Player> {
        if let Some(account_name) = self.map.get_key_value(&id.to_string()) {
            debug!("removing arc player: {}", account_name.0);
            let old_info = account_name.1.character_name.as_ref().cloned();
            self.map.remove_entry(account_name.0);
            old_info
        } else {
            None
        }
    }

    pub fn get_arc_player(&mut self, id: usize) -> Option<Player> {
        self.map.get_key_value(&id.to_string()).map(|(_, info)| {
            info.character_name.as_ref().cloned().unwrap()
        })
    }

    pub fn add_extras_player(&mut self, player: &UserInfo) {
        let account_name = strip_account_prefix(player.account_name());
        self.insert_name_into_cache(account_name, None);

        match self.map.entry(account_name.to_owned()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().extras = Some(Arc::new(player.clone()));
            }
            Entry::Vacant(entry) => {
                entry.insert(PlayerInfo::new_from_extras(player));
            }
        }
    }

    pub fn remove_extras_player(&mut self, player: &UserInfo) -> Option<Arc<UserInfo>> {
        let account_name = strip_account_prefix(player.account_name());
        self.map.remove_entry(account_name).map(|(_, info)| info.extras.take())
    }

    pub fn add_player_from_message(&mut self, message: &Message) {
        self.insert_name_into_cache(message.account_name(), Some(message.character_name()));
    }

    fn insert_name_into_cache(&mut self, account_name: &str, character_name: Option<&str>) {
        if let Some(entry) = self.map.get_mut(account_name) {
            if let Some(character_name) = character_name {
                entry.character_name = Some(character_name.to_owned());
            }
        } else {
            self.map.insert(
                account_name.to_owned(),
                PlayerInfo::new(character_name.map(|s| s.to_owned()), None),
            );
        }
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }
}
