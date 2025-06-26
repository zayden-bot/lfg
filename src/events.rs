use async_trait::async_trait;
use serenity::all::{MessageDeleteEvent, MessageId, PartialGuildChannel};
use sqlx::{Database, Pool, any::AnyQueryResult};

#[async_trait]
pub trait EventManager<Db: Database> {
    async fn delete(
        pool: &Pool<Db>,
        id: impl Into<MessageId> + Send,
    ) -> sqlx::Result<AnyQueryResult>;
}

pub async fn thread_delete<Db: Database, Manager: EventManager<Db>>(
    thread: &PartialGuildChannel,
    pool: &Pool<Db>,
) {
    Manager::delete(pool, thread.id.get()).await.unwrap();
}

pub async fn message_delete<Db: Database, Manager: EventManager<Db>>(
    event: &MessageDeleteEvent,
    pool: &Pool<Db>,
) {
    Manager::delete(pool, event.message_id).await.unwrap();
}
