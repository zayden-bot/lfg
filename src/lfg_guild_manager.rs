use async_trait::async_trait;
use serenity::all::{ChannelId, GuildId, RoleId};
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
        id: impl Into<GuildId> + Send,
        channel: impl Into<ChannelId> + Send,
        role: Option<impl Into<RoleId> + Send>,
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

    pub fn role_id(&self) -> Option<RoleId> {
        self.role_id.map(|id| RoleId::new(id as u64))
    }
}
