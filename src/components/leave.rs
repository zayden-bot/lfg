use serenity::all::{ComponentInteraction, Context, EditInteractionResponse};
use sqlx::{Database, Pool};

use crate::{PostManager, PostRow, Result, Savable, actions};

use super::Components;

impl Components {
    pub async fn leave<Db: Database, Manager: PostManager<Db> + Savable<Db, PostRow>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let content =
            actions::leave::<Db, Manager>(ctx, interaction, pool, interaction.user.display_name())
                .await
                .unwrap();

        interaction
            .edit_response(ctx, EditInteractionResponse::new().content(content))
            .await
            .unwrap();

        Ok(())
    }
}
