use serenity::all::{Context, PartialGuildChannel};
use sqlx::{Database, Pool};

use crate::{PostManager, actions};

pub async fn thread_delete<Db: Database, Manager: PostManager<Db>>(
    ctx: &Context,
    thread: &PartialGuildChannel,
    pool: &Pool<Db>,
) {
    if Manager::exists(pool, thread.id).await.unwrap() {
        actions::delete::<Db, Manager>(ctx, thread.id, pool)
            .await
            .unwrap();
    }
}
