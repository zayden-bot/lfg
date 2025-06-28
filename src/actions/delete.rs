use serenity::all::{ChannelId, Context};
use sqlx::{Database, Pool};

use crate::{PostManager, Result};

pub async fn delete<Db: Database, Manager: PostManager<Db>>(
    ctx: &Context,
    channel: ChannelId,
    pool: &Pool<Db>,
) -> Result<()> {
    channel.delete(ctx).await.unwrap();

    Manager::delete(pool, channel).await.unwrap();

    Ok(())
}
