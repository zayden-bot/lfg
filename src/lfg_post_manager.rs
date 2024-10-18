use chrono::{DateTime, FixedOffset, TimeZone};
use serenity::all::{Context, MessageId, User, UserId};
use serenity::async_trait;
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;
use sqlx::types::time::{OffsetDateTime, UtcOffset};
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
        start_time: OffsetDateTime,
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
    pub start_time: OffsetDateTime,
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

        let start_time = {
            let fixed_offset = start_time.offset().utc_minus_local();
            let utc_offset = UtcOffset::from_whole_seconds(fixed_offset).expect("Should be valid");
            OffsetDateTime::from_unix_timestamp(start_time.timestamp())
                .expect("Should be valid")
                .to_offset(utc_offset)
        };

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

    pub fn start_time(&self) -> DateTime<FixedOffset> {
        let naive_dt = DateTime::from_timestamp(
            self.start_time.unix_timestamp(),
            self.start_time.nanosecond(),
        )
        .expect("Should be a valid timestamp");

        let utc_offset = self.start_time.offset().whole_seconds();
        let fixed_offset = FixedOffset::east_opt(utc_offset).expect("Should be a valid UTC offset");

        fixed_offset.from_utc_datetime(&naive_dt.naive_utc())
    }

    pub fn timestamp(&self) -> i64 {
        self.start_time.unix_timestamp()
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
