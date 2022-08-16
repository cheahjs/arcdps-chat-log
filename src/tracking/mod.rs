use std::collections::{hash_map::Entry, HashMap};

use arc_util::tracking::Player;
use arcdps::extras::UserInfoOwned;

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
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            arc_id_map: HashMap::new(),
        }
    }

    pub fn add_arc_player(&mut self, player: &Player) -> Option<Player> {
        self.arc_id_map.insert(player.id, player.account.to_owned());
        match self.map.entry(player.account.to_owned()) {
            Entry::Occupied(entry) => {
                let entry = entry.into_mut();
                let old_info = entry.arc.as_ref().cloned();
                entry.arc = Some(player.clone());
                return old_info;
            }
            Entry::Vacant(entry) => {
                entry.insert(PlayerInfo::new_from_arc(player));
            }
        }
        None
    }

    pub fn remove_arc_player(&mut self, id: usize) -> Option<Player> {
        if let Some(account_name) = self.arc_id_map.get(&id) {
            if let Entry::Occupied(entry) = self.map.entry(account_name.to_owned()) {
                let info = entry.get();
                let old_info = info.arc.as_ref().cloned();
                if info.extras.is_none() {
                    entry.remove();
                } else {
                    let info = entry.into_mut();
                    info.arc = None;
                }
                return old_info;
            }
        }
        None
    }

    pub fn add_extras_player(&mut self, player: &UserInfoOwned) -> Option<UserInfoOwned> {
        match &player.account_name {
            Some(account_name) => match self.map.entry(account_name.to_owned()) {
                Entry::Occupied(entry) => {
                    let entry = entry.into_mut();
                    let old_info = entry.extras.as_ref().cloned();
                    entry.extras = Some(player.clone());
                    old_info
                }
                Entry::Vacant(entry) => {
                    entry.insert(PlayerInfo::new_from_extras(player));
                    None
                }
            },
            _ => None,
        }
    }

    pub fn remove_extras_player(&mut self, player: &UserInfoOwned) -> Option<UserInfoOwned> {
        match &player.account_name {
            Some(account_name) => match self.map.entry(account_name.to_owned()) {
                Entry::Occupied(entry) => {
                    let info = entry.get();
                    let old_info = info.extras.as_ref().cloned();
                    if info.arc.is_none() {
                        entry.remove();
                    } else {
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

    pub fn clear(&mut self) {
        self.map.clear()
    }
}
