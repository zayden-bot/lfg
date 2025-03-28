use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use serenity::all::{ChannelId, Context, EditThread, MessageId, User, UserId};
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;
use sqlx::{Database, Pool};

#[async_trait]
pub trait LfgPostManager<Db: sqlx::Database> {
    async fn get_past(pool: &Pool<Db>) -> sqlx::Result<Vec<LfgPostRow>>;

    async fn get(pool: &Pool<Db>, id: impl Into<MessageId> + Send) -> sqlx::Result<LfgPostRow>;

    async fn get_upcoming_by_user(
        pool: &Pool<Db>,
        id: impl Into<UserId> + Send,
    ) -> sqlx::Result<Vec<LfgPostRow>>;

    #[allow(clippy::too_many_arguments)]
    async fn save(
        pool: &Pool<Db>,
        id: impl Into<i64> + Send,
        owner: impl Into<i64> + Send,
        activity: &str,
        timestamp: NaiveDateTime,
        timezone: &str,
        description: &str,
        fireteam_size: impl Into<i16> + Send,
        fireteam: &[i64],
        alternatives: &[i64],
    ) -> sqlx::Result<AnyQueryResult>;

    async fn delete(
        pool: &Pool<Db>,
        id: impl Into<MessageId> + Send,
    ) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct LfgPostRow {
    pub id: i64,
    pub owner_id: i64,
    pub activity: String,
    pub timestamp: NaiveDateTime,
    pub timezone: String,
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
        start_time: DateTime<chrono_tz::Tz>,
        description: impl Into<String>,
        fireteam_size: impl Into<u8>,
    ) -> Self {
        let owner_id = owner_id.into().get() as i64;

        Self {
            id: (id.into().get() as i64),
            owner_id,
            activity: activity.into(),
            timestamp: start_time.naive_utc(),
            timezone: start_time.timezone().name().to_string(),
            description: description.into(),
            fireteam_size: (fireteam_size.into() as i16),
            fireteam: vec![owner_id],
            alternatives: Vec::new(),
        }
    }

    pub fn channel_id(&self) -> ChannelId {
        ChannelId::new(self.id as u64)
    }

    pub fn message_id(&self) -> MessageId {
        MessageId::new(self.id as u64)
    }

    pub fn owner_id(&self) -> UserId {
        UserId::new(self.owner_id as u64)
    }

    pub async fn owner(&self, ctx: &Context) -> serenity::Result<User> {
        let owner_id = UserId::new(self.owner_id as u64);
        owner_id.to_user(ctx).await
    }

    pub fn start_time(&self) -> DateTime<chrono_tz::Tz> {
        let timezone: chrono_tz::Tz = self.timezone.parse().expect("Should be a valid timezone");
        timezone.from_utc_datetime(&self.timestamp)
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp.and_utc().timestamp()
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

    pub fn contains(&self, user: impl Into<UserId>) -> bool {
        let user = user.into().get() as i64;

        self.fireteam.contains(&user) || self.alternatives.contains(&user)
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

    pub fn kick(&mut self, user: UserId) -> bool {
        if !self.contains(user) {
            return false;
        }

        self.leave(user);
        true
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
            self.timestamp,
            &self.timezone,
            &self.description,
            self.fireteam_size,
            &self.fireteam,
            &self.alternatives,
        )
        .await
    }

    pub async fn delete<Db: sqlx::Database, Manager: LfgPostManager<Db>>(
        self,
        pool: &Pool<Db>,
    ) -> sqlx::Result<AnyQueryResult> {
        Manager::delete(pool, self.id as u64).await
    }
}

pub async fn close_old_posts<Db: Database, Manager: LfgPostManager<Db>>(
    ctx: &Context,
    pool: &Pool<Db>,
) {
    let rows = Manager::get_past(pool).await.unwrap();

    for row in rows {
        if row
            .channel_id()
            .edit_thread(ctx, EditThread::new().archived(true))
            .await
            .is_err()
        {
            Manager::delete(pool, row.message_id()).await.unwrap();
        }
    }
}
