use chrono::{Duration, Utc};
use serenity::all::{Context, EditThread, Guild, PartialGuildChannel};
use sqlx::{Database, Pool};

use crate::{GuildManager, PostManager, actions, cron::create_reminders, templates::TemplateInfo};

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

pub async fn guild_create<
    Db: Database,
    GuildHandler: GuildManager<Db>,
    PostHandler: PostManager<Db>,
>(
    ctx: &Context,
    guild: &Guild,
    pool: &Pool<Db>,
) {
    let Ok(Some(guild_row)) = GuildHandler::row(pool, guild.id).await else {
        return;
    };

    let lfg_channel = guild_row.channel_id();

    let archived_threads = lfg_channel
        .get_archived_public_threads(&ctx, None, None)
        .await
        .unwrap();

    let threads = guild
        .threads
        .iter()
        .filter(|thread| thread.parent_id.is_some_and(|id| id == lfg_channel))
        .chain(archived_threads.threads.iter())
        .cloned();

    let now = Utc::now();
    let week_ago = now - Duration::days(7);
    let month_ago = now - Duration::days(30);

    for mut thread in threads {
        if *thread.last_message_id.unwrap().created_at() < month_ago {
            thread.delete(ctx).await.unwrap();
        }

        if *thread.last_message_id.unwrap().created_at() < week_ago {
            thread
                .edit_thread(ctx, EditThread::new().archived(true))
                .await
                .unwrap();
        }

        let post = match PostHandler::row(pool, thread.id).await {
            Ok(post) => post,
            Err(_) => continue,
        };

        let thread = post.channel();

        if !guild.channels.contains_key(&thread) {
            actions::delete::<Db, PostHandler>(ctx, thread, pool)
                .await
                .unwrap();
        }

        if post.start_time > now {
            create_reminders::<Db, PostHandler>(ctx, &post).await;
        }

        if post.start_time < now {
            post.channel()
                .edit_thread(ctx, EditThread::new().archived(true))
                .await
                .unwrap();

            if let (Some(channel), Some(message)) = (post.alt_channel(), post.alt_message()) {
                channel.delete_message(ctx, message).await.unwrap();
            }
        }
    }
}
