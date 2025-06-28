use serenity::all::{ChannelId, Context};
use sqlx::{Database, Pool};

use crate::{PostManager, Result, templates::TemplateInfo};

pub async fn delete<Db: Database, Manager: PostManager<Db>>(
    ctx: &Context,
    channel: ChannelId,
    pool: &Pool<Db>,
) -> Result<()> {
    let Ok(post) = Manager::row(pool, channel).await else {
        return Ok(());
    };

    post.channel().delete(ctx).await.unwrap();

    if let (Some(channel), Some(message)) = (post.alt_channel(), post.alt_message()) {
        channel.delete_message(ctx, message).await.unwrap();
    }

    Manager::delete(pool, channel).await.unwrap();

    Ok(())
}
