use serenity::all::{
    ButtonStyle, ComponentInteraction, Context, CreateActionRow, CreateButton,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EditMessage,
    Mentionable,
};
use sqlx::{Database, Pool};

use crate::{
    Error, LfgMessageManager, LfgPostManager, LfgPostWithMessages, Result, create_lfg_embed,
    create_main_row,
};

pub struct PostComponents;

impl PostComponents {
    pub async fn join<Db, PostManager, MessageManager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: Database,
        PostManager: LfgPostManager<Db> + Send,
        MessageManager: LfgMessageManager<Db>,
    {
        let LfgPostWithMessages { mut post, messages } =
            PostManager::get_with_messages::<MessageManager>(pool, interaction.channel_id.get())
                .await
                .unwrap();

        post.join(interaction.user.id, false)?;

        let owner_name = &post.owner(ctx).await.unwrap().name;
        let thread_embed = create_lfg_embed(&post, owner_name, None);
        let msg_embed = create_lfg_embed(&post, owner_name, Some(interaction.channel_id));

        post.save::<Db, PostManager>(pool).await.unwrap();

        interaction
            .channel_id
            .send_message(
                ctx,
                CreateMessage::new().content(format!(
                    "{} joined the fireteam",
                    interaction.user.mention()
                )),
            )
            .await
            .unwrap();

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(thread_embed),
                ),
            )
            .await
            .unwrap();

        for message in messages {
            message
                .channel_id()
                .edit_message(
                    ctx,
                    message.message_id(),
                    EditMessage::new().embed(msg_embed.clone()),
                )
                .await
                .unwrap();
        }

        Ok(())
    }

    pub async fn leave<Db, PostManager, MessageManager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: Database,
        PostManager: LfgPostManager<Db> + Send,
        MessageManager: LfgMessageManager<Db>,
    {
        let LfgPostWithMessages { mut post, messages } =
            PostManager::get_with_messages::<MessageManager>(pool, interaction.channel_id.get())
                .await
                .unwrap();

        post.leave(interaction.user.id);

        let owner_name = post.owner(ctx).await.unwrap().name;

        let thread_embed = create_lfg_embed(&post, &owner_name, None);
        let msg_embed = create_lfg_embed(&post, &owner_name, Some(interaction.channel_id));

        post.save::<Db, PostManager>(pool).await.unwrap();

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(thread_embed),
                ),
            )
            .await
            .unwrap();

        interaction
            .channel_id
            .send_message(
                ctx,
                CreateMessage::new()
                    .content(format!("{} left the fireteam", interaction.user.mention())),
            )
            .await
            .unwrap();

        for message in messages {
            message
                .channel_id()
                .edit_message(
                    ctx,
                    message.message_id(),
                    EditMessage::new().embed(msg_embed.clone()),
                )
                .await
                .unwrap();
        }

        Ok(())
    }

    pub async fn alternative<Db, PostManager, MessageManager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        PostManager: LfgPostManager<Db> + Send,
        MessageManager: LfgMessageManager<Db>,
    {
        let LfgPostWithMessages { mut post, messages } =
            PostManager::get_with_messages::<MessageManager>(pool, interaction.channel_id.get())
                .await
                .unwrap();

        post.join(interaction.user.id, true)?;

        let owner_name = &post.owner(ctx).await.unwrap().name;
        let thread_embed = create_lfg_embed(&post, owner_name, None);
        let msg_embed = create_lfg_embed(&post, owner_name, Some(interaction.channel_id));

        post.save::<Db, PostManager>(pool).await.unwrap();

        interaction
            .channel_id
            .send_message(
                ctx,
                CreateMessage::new().content(format!(
                    "{} joined as an alternative",
                    interaction.user.mention()
                )),
            )
            .await
            .unwrap();

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(thread_embed),
                ),
            )
            .await
            .unwrap();

        for message in messages {
            message
                .channel_id()
                .edit_message(
                    ctx,
                    message.message_id(),
                    EditMessage::new().embed(msg_embed.clone()),
                )
                .await
                .unwrap();
        }

        Ok(())
    }

    pub async fn settings<Db, Manager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        let post = Manager::get(pool, interaction.message.id).await.unwrap();

        if interaction.user.id != post.owner_id() {
            return Err(Error::PermissionDenied(post.owner_id()));
        }

        let main_row = create_main_row();
        let settings_row = CreateActionRow::Buttons(vec![
            CreateButton::new("lfg_edit")
                .label("Edit")
                .style(ButtonStyle::Secondary),
            CreateButton::new("lfg_copy")
                .label("Copy")
                .style(ButtonStyle::Secondary),
            CreateButton::new("lfg_kick")
                .label("Kick")
                .style(ButtonStyle::Secondary),
            CreateButton::new("lfg_delete")
                .label("Delete")
                .style(ButtonStyle::Danger),
        ]);

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .components(vec![main_row, settings_row]),
                ),
            )
            .await
            .unwrap();

        Ok(())
    }
}
