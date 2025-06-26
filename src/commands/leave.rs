use serenity::all::{CommandInteraction, Context};
use sqlx::{Database, Pool};

use crate::{PostManager, Result, actions};

use super::Command;

impl Command {
    pub async fn leave<Db: Database, Manager: PostManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        actions::leave::<Db, Manager>(ctx, interaction, pool, interaction.user.display_name())
            .await
            .unwrap();

        Ok(())
    }
}
