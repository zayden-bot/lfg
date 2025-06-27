use serenity::all::{
    Builder, CacheHttp, ChannelId, CommandInteraction, ComponentInteraction, Context,
    EditInteractionResponse, Mentionable, Message, ResolvedValue, UserId,
};
use sqlx::{Database, Pool};
use zayden_core::parse_options;

use crate::models::Savable;
use crate::templates::DefaultTemplate;
use crate::utils::{Announcement, update_embeds};
use crate::{Join, PostManager, PostRow, Result};

pub struct JoinInteraction {
    token: String,
    thread: ChannelId,
    user: UserId,
}

impl JoinInteraction {
    pub async fn edit_response(
        &self,
        cache_http: impl CacheHttp,
        builder: EditInteractionResponse,
    ) -> serenity::Result<Message> {
        builder.execute(cache_http, &self.token).await
    }
}

impl From<&ComponentInteraction> for JoinInteraction {
    fn from(value: &ComponentInteraction) -> Self {
        Self {
            token: value.token.clone(),
            thread: value.channel_id,
            user: value.user.id,
        }
    }
}

impl From<&CommandInteraction> for JoinInteraction {
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
            user,
        }
    }
}

pub async fn join<Db: Database, Manager: PostManager<Db> + Savable<Db, PostRow>>(
    ctx: &Context,
    interaction: impl Into<JoinInteraction>,
    pool: &Pool<Db>,
    alternative: bool,
    owner_name: &str,
) -> Result<()> {
    let interaction = interaction.into();

    let mut row = Manager::row(pool, interaction.thread).await.unwrap();
    row.join(interaction.user, alternative)?;

    update_embeds::<DefaultTemplate>(ctx, &row, owner_name, interaction.thread).await;
    Announcement::Joined {
        user: interaction.user,
        alternative,
    }
    .send(ctx, interaction.thread)
    .await;

    Manager::save(pool, row).await.unwrap();

    interaction
        .edit_response(
            ctx,
            EditInteractionResponse::new()
                .content(format!("You have joined {}", interaction.thread.mention())),
        )
        .await
        .unwrap();

    Ok(())
}
