use std::collections::{hash_map::Entry, HashMap, HashSet};

use arc_util::tracking::Player;
use arcdps::{
    extras::{message::ChatMessageInfo, UserInfoOwned},
    strip_account_prefix,
};
use log::debug;

#[derive(Debug)]
pub struct PlayerInfo {
    pub arc: Option<Player>,
    pub extras: Option<UserInfoOwned>,
}

impl PlayerInfo {
    pub fn new(arc: Option<Player>, extras: Option<UserInfoOwned>) -> Self {
        Self { arc, extras }
    }

    pub fn new_from_arc(player: &Player) -> Self {
        Self::new(Some(player.clone()), None)
    }

    pub fn new_from_extras(player: &UserInfoOwned) -> Self {
        Self::new(None, Some(player.clone()))
    }
}

#[derive(Debug)]
pub struct Tracker {
    pub map: HashMap<String, PlayerInfo>,
    pub arc_id_map: HashMap<usize, String>,
    pub seen_users: HashMap<String, HashSet<String>>,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            arc_id_map: HashMap::new(),
            seen_users: HashMap::new(),
        }
    }

    pub fn add_arc_player(&mut self, player: &Player) -> Option<Player> {
        debug!("adding arc player: {:?}", player);
        self.insert_name_into_cache(&player.account, Some(&player.character));
        self.arc_id_map.insert(player.id, player.account.to_owned());
        match self.map.entry(player.account.to_owned()) {
            Entry::Occupied(entry) => {
                debug!("adding arc player into existing entry");
                let entry = entry.into_mut();
                let old_info = entry.arc.as_ref().cloned();
                entry.arc = Some(player.clone());
                return old_info;
            }
            Entry::Vacant(entry) => {
                debug!("adding arc player into new entry");
                entry.insert(PlayerInfo::new_from_arc(player));
            }
        }
        None
    }

    pub fn remove_arc_player(&mut self, id: usize) -> Option<Player> {
        if let Some(account_name) = self.arc_id_map.get(&id) {
            debug!("removing arc player: {}", account_name);
            if let Entry::Occupied(entry) = self.map.entry(account_name.to_owned()) {
                let info = entry.get();
                let old_info = info.arc.as_ref().cloned();
                if info.extras.is_none() {
                    debug!("player has no extras, removing from map");
                    entry.remove();
                } else {
                    let info = entry.into_mut();
                    debug!("player has extras, removing arc");
                    info.arc = None;
                }
                return old_info;
            }
        }
        None
    }

    pub fn get_arc_player(&mut self, id: usize) -> Option<Player> {
        if let Some(account_name) = self.arc_id_map.get(&id) {
            if let Entry::Occupied(entry) = self.map.entry(account_name.to_owned()) {
                return entry.get().arc.as_ref().cloned();
            }
        }
        None
    }

    pub fn add_extras_player(&mut self, player: &UserInfoOwned) -> Option<UserInfoOwned> {
        debug!("adding extras player {:?}", player);
        match &player.account_name {
            Some(account_name) => {
                self.insert_name_into_cache(strip_account_prefix(account_name), None);
                match self.map.entry(account_name.to_owned()) {
                    Entry::Occupied(entry) => {
                        debug!("adding extras player into existing entry");
                        let entry = entry.into_mut();
                        let old_info = entry.extras.as_ref().cloned();
                        entry.extras = Some(player.clone());
                        old_info
                    }
                    Entry::Vacant(entry) => {
                        debug!("adding extras player into new entry");
                        entry.insert(PlayerInfo::new_from_extras(player));
                        None
                    }
                }
            }
            _ => None,
        }
    }

    pub fn remove_extras_player(&mut self, player: &UserInfoOwned) -> Option<UserInfoOwned> {
        debug!("removing extras player {:?}", player);
        match &player.account_name {
            Some(account_name) => match self.map.entry(account_name.to_owned()) {
                Entry::Occupied(entry) => {
                    let info = entry.get();
                    let old_info = info.extras.as_ref().cloned();
                    if info.arc.is_none() {
                        debug!("player has no arc, removing from map");
                        entry.remove();
                    } else {
                        debug!("player has arc, removing extras");
                        let info = entry.into_mut();
                        info.extras = None;
                    }
                    old_info
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub fn add_player_from_message(&mut self, message: &ChatMessageInfo) {
        self.insert_name_into_cache(message.account_name, Some(message.character_name));
    }

    fn insert_name_into_cache(&mut self, account_name: &str, character_name: Option<&str>) {
        match self.seen_users.entry(account_name.to_owned()) {
            Entry::Occupied(entry) => {
                if let Some(character_name) = character_name {
                    entry.into_mut().insert(character_name.to_owned());
                }
            }
            Entry::Vacant(entry) => {
                let mut set = HashSet::new();
                if let Some(character_name) = character_name {
                    set.insert(character_name.to_owned());
                }
                entry.insert(set);
            }
        };
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }
}
