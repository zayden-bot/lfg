use serenity::all::{ComponentInteraction, Context};
use sqlx::{Database, Pool};

use crate::{PostManager, Result, actions};

use super::Components;

impl Components {
    pub async fn alternative<Db: Database, Manager: PostManager<Db>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        actions::join::<Db, Manager>(
            ctx,
            interaction,
            pool,
            true,
            interaction.user.display_name(),
        )
        .await
        .unwrap();

        Ok(())
    }
}
