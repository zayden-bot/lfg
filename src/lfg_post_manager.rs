use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serenity::all::{MessageId, UserId};
use serenity::prelude::TypeMapKey;

pub struct LfgPostManager;

impl TypeMapKey for LfgPostManager {
    type Value = HashMap<MessageId, LfgPostData>;
}

pub struct LfgPostData {
    pub owner: UserId,
    pub activity: String,
    pub start_time: DateTime<Utc>,
    pub description: String,
    pub fireteam_size: u8,
    pub fireteam: HashSet<UserId>,
}

impl LfgPostData {
    pub fn new(
        owner: UserId,
        activity: String,
        start_time: DateTime<Utc>,
        description: String,
        fireteam_size: u8,
    ) -> Self {
        let mut fireteam = HashSet::with_capacity(6);
        fireteam.insert(owner);

        Self {
            owner,
            activity,
            start_time,
            description,
            fireteam_size,
            fireteam,
        }
    }
}
