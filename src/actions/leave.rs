use serenity::all::{
    Builder, CacheHttp, ChannelId, CommandInteraction, ComponentInteraction, Context,
    EditInteractionResponse, Mentionable, Message, ResolvedValue, UserId,
};
use sqlx::{Database, Pool};
use zayden_core::parse_options;

use crate::{
    Leave, PostManager, PostRow, Result,
    models::Savable,
    templates::DefaultTemplate,
    utils::{Announcement, update_embeds},
};

pub struct LeaveInteraction {
    token: String,
    thread: ChannelId,
    author: UserId,
    user: UserId,
}

impl LeaveInteraction {
    pub async fn edit_response(
        &self,
        cache_http: impl CacheHttp,
        builder: EditInteractionResponse,
    ) -> serenity::Result<Message> {
        builder.execute(cache_http, &self.token).await
    }
}

impl From<&CommandInteraction> for LeaveInteraction {
    fn from(value: &CommandInteraction) -> Self {
        let options = value.data.options();
        let mut options = parse_options(options);
        let thread = match options.remove("thread") {
            Some(ResolvedValue::Channel(channel)) => channel.id,
            _ => value.channel_id,
        };
        let user = match options.remove("guardian") {
            Some(ResolvedValue::User(user, _)) => user.id,
            _ => value.user.id,
        };

        Self {
            token: value.token.clone(),
            thread,
            author: value.user.id,
            user,
        }
    }
}

impl From<&ComponentInteraction> for LeaveInteraction {
    fn from(value: &ComponentInteraction) -> Self {
        Self {
            token: value.token.clone(),
            thread: value.channel_id,
            author: value.user.id,
            user: value.user.id,
        }
    }
}

pub async fn leave<Db: Database, Manager: PostManager<Db> + Savable<Db, PostRow>>(
    ctx: &Context,
    interaction: impl Into<LeaveInteraction>,
    pool: &Pool<Db>,
    owner_name: &str,
) -> Result<()> {
    let interaction = interaction.into();

    let mut row = Manager::row(pool, interaction.thread).await.unwrap();
    row.leave(interaction.user);

    update_embeds::<DefaultTemplate>(ctx, &row, owner_name, interaction.thread).await;
    Announcement::Left(interaction.user)
        .send(ctx, interaction.thread)
        .await;

    Manager::save(pool, row).await.unwrap();

    let content = if interaction.author == interaction.user {
        format!("You have left {}", interaction.thread.mention())
    } else {
        format!(
            "{} have left {}",
            interaction.user.mention(),
            interaction.thread.mention()
        )
    };

    interaction
        .edit_response(ctx, EditInteractionResponse::new().content(content))
        .await
        .unwrap();

    Ok(())
}
