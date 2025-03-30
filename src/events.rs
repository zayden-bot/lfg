use serenity::all::{MessageDeleteEvent, PartialGuildChannel};
use sqlx::{Database, Pool};

use crate::{LfgMessageManager, LfgPostManager};

pub async fn thread_delete<Db: Database, Manager: LfgPostManager<Db>>(
    thread: &PartialGuildChannel,
    pool: &Pool<Db>,
) {
    let _ = Manager::delete(pool, thread.id.get()).await;
}

pub async fn message_delete<Db: Database, Manager: LfgMessageManager<Db>>(
    event: MessageDeleteEvent,
    pool: &Pool<Db>,
) {
    let _ = Manager::delete(pool, event.message_id).await;
}
