use chrono::{DateTime, FixedOffset};
use chrono_tz::Tz;
use serenity::all::{Context, MessageId, User, UserId};
use serenity::async_trait;
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;
use sqlx::Pool;

#[async_trait]
pub trait LfgPostManager<Db: sqlx::Database> {
    async fn get(pool: &Pool<Db>, id: impl Into<MessageId> + Send) -> sqlx::Result<LfgPostRow>;

    #[allow(clippy::too_many_arguments)]
    async fn save(
        pool: &Pool<Db>,
        id: impl Into<i64> + Send,
        owner: impl Into<i64> + Send,
        activity: &str,
        start_time: DateTime<FixedOffset>,
        description: &str,
        fireteam_size: impl Into<i16> + Send,
        fireteam: &[i64],
        alternatives: &[i64],
    ) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct LfgPostRow {
    pub id: i64,
    pub owner_id: i64,
    pub activity: String,
    pub start_time: DateTime<FixedOffset>,
    pub description: String,
    pub fireteam_size: i16,
    pub fireteam: Vec<i64>,
    pub alternatives: Vec<i64>,
}

impl LfgPostRow {
    pub fn new(
        id: impl Into<MessageId>,
        owner_id: impl Into<UserId>,
        activity: impl Into<String>,
        start_time: DateTime<FixedOffset>,
        description: impl Into<String>,
        fireteam_size: impl Into<u8>,
    ) -> Self {
        let owner_id = owner_id.into().get() as i64;

        Self {
            id: (id.into().get() as i64),
            owner_id,
            activity: activity.into(),
            start_time,
            description: description.into(),
            fireteam_size: (fireteam_size.into() as i16),
            fireteam: vec![owner_id],
            alternatives: Vec::new(),
        }
    }

    pub fn owner_id(&self) -> UserId {
        UserId::new(self.owner_id as u64)
    }

    pub async fn owner(&self, ctx: &Context) -> serenity::Result<User> {
        let owner_id = UserId::new(self.owner_id as u64);
        owner_id.to_user(ctx).await
    }

    pub fn timestamp(&self) -> i64 {
        self.start_time.timestamp()
    }

    pub fn fireteam(&self) -> Vec<UserId> {
        self.fireteam
            .iter()
            .map(|id| UserId::new((*id) as u64))
            .collect()
    }

    pub fn fireteam_size(&self) -> u8 {
        self.fireteam_size as u8
    }

    pub fn alternatives(&self) -> Vec<UserId> {
        self.alternatives
            .iter()
            .map(|id| UserId::new((*id) as u64))
            .collect()
    }

    pub fn is_full(&self) -> bool {
        self.fireteam.len() as i16 == self.fireteam_size
    }

    pub fn join(&mut self, user: impl Into<UserId>) {
        let id = user.into().get() as i64;

        self.leave(id as u64);
        self.fireteam.push(id);
    }

    pub fn join_alt(&mut self, id: impl Into<UserId>) {
        let id = id.into().get() as i64;

        self.leave(id as u64);
        self.alternatives.push(id);
    }

    pub fn leave(&mut self, user: impl Into<UserId>) {
        let user = user.into().get() as i64;

        self.fireteam.retain(|&id| id != user);
        self.alternatives.retain(|&id| id != user);
    }

    pub async fn save<Db: sqlx::Database, Manager: LfgPostManager<Db>>(
        self,
        pool: &Pool<Db>,
    ) -> sqlx::Result<AnyQueryResult> {
        Manager::save(
            pool,
            self.id,
            self.owner_id,
            &self.activity,
            self.start_time,
            &self.description,
            self.fireteam_size,
            &self.fireteam,
            &self.alternatives,
        )
        .await
    }
}
