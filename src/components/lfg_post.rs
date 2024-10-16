use serenity::all::{
    ButtonStyle, ComponentInteraction, Context, CreateActionRow, CreateButton,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use sqlx::Pool;

use crate::{create_lfg_embed, create_main_row, Error, LfgPostManager, Result};

pub struct PostComponents;

impl PostComponents {
    pub async fn join<Db, Manager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        let mut post = Manager::get(pool, &interaction.message.id).await?;

        if post.is_full() {
            return Err(Error::FireteamFull);
        }

        post.join(interaction.user.id);

        let embed = create_lfg_embed(&post, &post.owner(ctx).await?.name);

        post.save::<Db, Manager>(pool).await?;

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn leave<Db, Manager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        let mut post = Manager::get(pool, interaction.message.id).await?;

        post.leave(interaction.user.id);

        let embed = create_lfg_embed(&post, &post.owner(ctx).await?.name);

        post.save::<Db, Manager>(pool).await?;

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn alternative<Db, Manager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        let mut post = Manager::get(pool, interaction.message.id).await?;

        post.join_alt(interaction.user.id);

        let embed = create_lfg_embed(&post, &post.owner(ctx).await?.name);

        post.save::<Db, Manager>(pool).await?;

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await?;

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
        let post = Manager::get(pool, interaction.message.id).await?;

        if interaction.user.id != post.owner_id() {
            return Err(Error::PermissionDenied {
                owner: post.owner_id(),
            });
        }

        let main_row = create_main_row();
        let settings_row_1 = CreateActionRow::Buttons(vec![
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
                        .components(vec![main_row, settings_row_1]),
                ),
            )
            .await?;

        Ok(())
    }
}
