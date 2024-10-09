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
        owner: impl Into<UserId>,
        activity: impl Into<String>,
        start_time: DateTime<Utc>,
        description: impl Into<String>,
        fireteam_size: impl Into<u8>,
    ) -> Self {
        let owner = owner.into();

        let mut fireteam = HashSet::with_capacity(6);
        fireteam.insert(owner);

        Self {
            owner,
            activity: activity.into(),
            start_time,
            description: description.into(),
            fireteam_size: fireteam_size.into(),
            fireteam,
        }
    }
}
