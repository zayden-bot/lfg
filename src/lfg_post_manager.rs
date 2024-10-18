use chrono::DateTime;
use chrono_tz::Tz;
use serenity::all::{MessageId, UserId};
use serenity::async_trait;
use sqlx::any::AnyQueryResult;
use sqlx::prelude::FromRow;

#[async_trait]
pub trait LfgPostManager<Db: sqlx::Database> {
    async fn create(
        pool: &sqlx::Pool<Db>,
        id: impl Into<MessageId>,
        owner: impl Into<UserId>,
        activity: impl Into<String>,
        start_time: DateTime<Tz>,
        description: impl Into<String>,
        fireteam_size: impl Into<u8>,
    ) -> sqlx::Result<AnyQueryResult>;

    async fn get(pool: &sqlx::Pool<Db>, id: impl Into<MessageId>) -> sqlx::Result<LfgPostRow>;

    async fn save(pool: &sqlx::Pool<Db>, post: LfgPostRow) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct LfgPostRow {
    pub owner: UserId,
    pub activity: String,
    pub start_time: DateTime<Tz>,
    pub description: String,
    pub fireteam_size: u8,
    pub fireteam: Vec<UserId>,
}

impl LfgPostRow {
    pub fn is_full(&self) -> bool {
        self.fireteam.len() as u8 == self.fireteam_size
    }

    pub fn join(&mut self, user: impl Into<UserId>) {
        self.fireteam.push(user.into());
    }

    pub fn leave(&mut self, user: impl Into<UserId>) {
        let user = user.into();

        self.fireteam.retain(|&id| id != user);
    }
}
