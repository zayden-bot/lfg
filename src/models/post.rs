use async_trait::async_trait;
use chrono::DateTime;
use chrono_tz::Tz;
use serenity::all::{ChannelId, UserId};
use sqlx::{Database, Pool, any::AnyQueryResult};

use crate::templates::TemplateInfo;
use crate::{Join, Leave};

pub struct PostBuilder {
    id: ChannelId,
    owner: UserId,
    activity: String,
    start_time: DateTime<Tz>,
    description: String,
    fireteam_size: i16,
    fireteam: Vec<UserId>,
    alternatives: Vec<UserId>,
}

impl PostBuilder {
    pub fn new(
        owner: impl Into<UserId>,
        activity: impl Into<String>,
        start: DateTime<Tz>,
        desc: impl Into<String>,
        fireteam_size: i16,
    ) -> Self {
        let owner = owner.into();

        Self {
            id: ChannelId::default(),
            owner,
            activity: activity.into(),
            start_time: start,
            description: desc.into(),
            fireteam_size,
            fireteam: vec![owner],
            alternatives: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl Into<ChannelId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn activity(mut self, activity: impl Into<String>) -> Self {
        self.activity = activity.into();
        self
    }

    pub fn fireteam_size(mut self, size: i16) -> Self {
        self.fireteam_size = size;
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn start(mut self, start: DateTime<Tz>) -> Self {
        self.start_time = start;
        self
    }

    pub fn build(self) -> PostRow {
        PostRow {
            id: self.id.get() as i64,
            owner_id: self.owner.get() as i64,
            activity: self.activity,
            start_time: self.start_time,
            description: self.description,
            fireteam_size: self.fireteam_size,
            fireteam: self
                .fireteam
                .into_iter()
                .map(|user| user.get() as i64)
                .collect(),
            alternatives: self
                .alternatives
                .into_iter()
                .map(|user| user.get() as i64)
                .collect(),
        }
    }
}

impl TemplateInfo for PostBuilder {
    fn activity(&self) -> &str {
        &self.activity
    }

    fn timestamp(&self) -> i64 {
        self.start_time.timestamp()
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn fireteam_size(&self) -> i16 {
        self.fireteam_size
    }

    fn fireteam(&self) -> impl Iterator<Item = UserId> {
        self.fireteam.iter().copied()
    }

    fn alternatives(&self) -> impl Iterator<Item = UserId> {
        self.alternatives.iter().copied()
    }
}

impl From<PostRow> for PostBuilder {
    fn from(value: PostRow) -> Self {
        Self {
            id: ChannelId::new(value.id as u64),
            owner: UserId::new(value.owner_id as u64),
            activity: value.activity,
            start_time: value.start_time,
            description: value.description,
            fireteam_size: value.fireteam_size,
            fireteam: value
                .fireteam
                .into_iter()
                .map(|id| UserId::new(id as u64))
                .collect(),
            alternatives: value
                .alternatives
                .into_iter()
                .map(|id| UserId::new(id as u64))
                .collect(),
        }
    }
}

#[async_trait]
pub trait PostManager<Db: Database> {
    async fn owner(pool: &Pool<Db>, id: impl Into<ChannelId> + Send) -> sqlx::Result<UserId>;

    async fn row(pool: &Pool<Db>, id: impl Into<ChannelId> + Send) -> sqlx::Result<PostRow>;

    async fn save(pool: &Pool<Db>, row: PostRow) -> sqlx::Result<AnyQueryResult>;

    async fn delete(
        pool: &Pool<Db>,
        id: impl Into<ChannelId> + Send,
    ) -> sqlx::Result<AnyQueryResult>;
}

pub struct PostRow {
    pub id: i64,
    pub owner_id: i64,
    pub activity: String,
    pub start_time: DateTime<Tz>,
    pub description: String,
    pub fireteam_size: i16,
    pub fireteam: Vec<i64>,
    pub alternatives: Vec<i64>,
}

impl Leave for PostRow {
    fn fireteam_mut(&mut self) -> &mut Vec<i64> {
        &mut self.fireteam
    }

    fn alternatives_mut(&mut self) -> &mut Vec<i64> {
        &mut self.alternatives
    }
}

impl Join for PostRow {
    fn fireteam_size(&self) -> i16 {
        self.fireteam_size
    }

    fn fireteam(&self) -> impl Iterator<Item = UserId> {
        self.fireteam.iter().map(|&id| UserId::new(id as u64))
    }

    fn fireteam_len(&self) -> i16 {
        self.fireteam.len() as i16
    }

    fn alternatives(&self) -> impl Iterator<Item = UserId> {
        self.alternatives.iter().map(|&id| UserId::new(id as u64))
    }
}

impl TemplateInfo for PostRow {
    fn activity(&self) -> &str {
        &self.activity
    }

    fn timestamp(&self) -> i64 {
        self.start_time.timestamp()
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn fireteam_size(&self) -> i16 {
        self.fireteam_size
    }

    fn fireteam(&self) -> impl Iterator<Item = UserId> {
        self.fireteam.iter().map(|&id| UserId::new(id as u64))
    }

    fn alternatives(&self) -> impl Iterator<Item = UserId> {
        self.alternatives.iter().map(|&id| UserId::new(id as u64))
    }
}
