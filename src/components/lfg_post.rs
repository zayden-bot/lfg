use serenity::all::{
    ComponentInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use sqlx::Pool;

use crate::{create_lfg_embed, Error, LfgPostManager, Result};

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
}
