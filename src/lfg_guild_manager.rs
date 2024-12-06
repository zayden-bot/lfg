use async_trait::async_trait;
use serenity::all::{ChannelId, GuildId};
use sqlx::any::AnyQueryResult;
use sqlx::{FromRow, Pool};

#[async_trait]
pub trait LfgGuildManager<Db: sqlx::Database> {
    async fn get(
        pool: &Pool<Db>,
        id: impl Into<GuildId> + Send,
    ) -> sqlx::Result<Option<LfgGuildRow>>;

    async fn save(
        pool: &Pool<Db>,
        id: impl Into<i64> + Send,
        channel: impl Into<i64> + Send,
        role: Option<impl Into<i64> + Send>,
    ) -> sqlx::Result<AnyQueryResult>;
}

#[derive(FromRow)]
pub struct LfgGuildRow {
    pub id: i64,
    pub channel_id: i64,
    pub role_id: Option<i64>,
}

impl LfgGuildRow {
    pub fn channel_id(&self) -> ChannelId {
        ChannelId::new(self.channel_id as u64)
    }
}
