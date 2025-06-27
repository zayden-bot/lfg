use serenity::all::{MessageDeleteEvent, PartialGuildChannel};
use sqlx::{Database, Pool};

use crate::PostManager;

pub async fn thread_delete<Db: Database, Manager: PostManager<Db>>(
    thread: &PartialGuildChannel,
    pool: &Pool<Db>,
) {
    Manager::delete(pool, thread.id).await.unwrap();
}

pub async fn message_delete<Db: Database, Manager: PostManager<Db>>(
    event: &MessageDeleteEvent,
    pool: &Pool<Db>,
) {
    Manager::delete(pool, event.message_id.get()).await.unwrap();
}
