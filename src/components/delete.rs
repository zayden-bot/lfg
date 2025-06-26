use serenity::all::{ComponentInteraction, Context, CreateInteractionResponse};
use sqlx::{Database, Pool};

use crate::{Error, PostManager, Result};

use super::Components;

impl Components {
    pub async fn delete<Db: Database, Manager: PostManager<Db>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        let owner = Manager::owner(pool, interaction.channel_id).await?;

        if interaction.user.id != owner {
            return Err(Error::PermissionDenied(owner));
        }

        interaction.channel_id.delete(ctx).await.unwrap();

        Manager::delete(pool, interaction.channel_id).await.unwrap();

        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
            .unwrap();

        Ok(())
    }
}
