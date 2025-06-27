use std::collections::HashMap;

use serenity::all::{CommandInteraction, Context, ResolvedValue};
use sqlx::{Database, Pool};

use crate::{PostManager, PostRow, Result, Savable, actions};

use super::Command;

impl Command {
    pub async fn join<Db: Database, Manager: PostManager<Db> + Savable<Db, PostRow>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        mut options: HashMap<&str, ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let user = match options.remove("guardian") {
            Some(ResolvedValue::User(user, _)) => user,
            _ => &interaction.user,
        };

        let alternative = match options.remove("alternative") {
            Some(ResolvedValue::Boolean(alt)) => alt,
            _ => false,
        };

        actions::join::<Db, Manager>(ctx, interaction, pool, alternative, user.display_name())
            .await
            .unwrap();

        Ok(())
    }
}
