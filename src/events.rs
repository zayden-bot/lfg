use serenity::all::PartialGuildChannel;
use sqlx::{Database, Pool};

use crate::LfgPostManager;

pub async fn thread_delete<Db: Database, Manager: LfgPostManager<Db>>(
    thread: &PartialGuildChannel,
    pool: &Pool<Db>,
) {
    let _ = Manager::delete(pool, thread.id.get()).await;
}
