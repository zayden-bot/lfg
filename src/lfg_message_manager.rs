use async_trait::async_trait;
use serenity::all::{ChannelId, MessageId};
use sqlx::{FromRow, Pool};

#[async_trait]
pub trait LfgMessageManager<Db: sqlx::Database> {
    async fn get(pool: &Pool<Db>, id: impl Into<MessageId> + Send) -> sqlx::Result<LfgMessageRow>;

    async fn get_by_post_id(
        pool: &Pool<Db>,
        id: impl Into<ChannelId> + Send,
    ) -> sqlx::Result<Vec<LfgMessageRow>>;

    async fn save(pool: &Pool<Db>, row: LfgMessageRow) -> sqlx::Result<()>;
}

#[derive(FromRow)]
pub struct LfgMessageRow {
    pub id: i64,
    pub channel_id: i64,
    pub post_id: i64,
}

impl LfgMessageRow {
    pub fn new(
        id: impl Into<MessageId>,
        channel_id: impl Into<ChannelId>,
        post_id: impl Into<ChannelId>,
    ) -> Self {
        Self {
            id: id.into().get() as i64,
            channel_id: channel_id.into().get() as i64,
            post_id: post_id.into().get() as i64,
        }
    }

    pub fn message_id(&self) -> MessageId {
        MessageId::new(self.id as u64)
    }

    pub fn channel_id(&self) -> ChannelId {
        ChannelId::new(self.channel_id as u64)
    }

    pub fn post_id(&self) -> ChannelId {
        ChannelId::new(self.post_id as u64)
    }
}
