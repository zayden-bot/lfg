use serenity::all::{ComponentInteraction, Context, CreateInteractionResponse};
use sqlx::{Database, Pool};

use crate::{PostManager, PostRow, Result, Savable, actions};

use super::Components;

impl Components {
    pub async fn join<Db: Database, Manager: PostManager<Db> + Savable<Db, PostRow>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        actions::join::<Db, Manager>(
            ctx,
            interaction,
            pool,
            false,
            interaction.user.display_name(),
        )
        .await?;

        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
            .unwrap();

        Ok(())
    }
}
