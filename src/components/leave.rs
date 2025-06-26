use serenity::all::{ComponentInteraction, Context};
use sqlx::{Database, Pool};

use crate::{PostManager, Result, actions};

use super::Components;

impl Components {
    pub async fn leave<Db: Database, Manager: PostManager<Db>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        actions::leave::<Db, Manager>(ctx, interaction, pool, interaction.user.display_name())
            .await
            .unwrap();

        Ok(())
    }
}
