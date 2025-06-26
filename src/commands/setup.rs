use std::collections::HashMap;

use async_trait::async_trait;
use serenity::all::{
    ChannelId, CommandInteraction, Context, EditInteractionResponse, GuildId, ResolvedValue, RoleId,
};
use sqlx::any::AnyQueryResult;
use sqlx::{Database, Pool};

use crate::{Error, Result};

use super::Command;

#[async_trait]
pub trait SetupManager<Db: Database> {
    async fn insert(
        pool: &Pool<Db>,
        id: impl Into<GuildId> + Send,
        channel: impl Into<ChannelId> + Send,
        role: Option<impl Into<RoleId> + Send>,
    ) -> sqlx::Result<AnyQueryResult>;
}

impl Command {
    pub async fn setup<Db: Database, Manager: SetupManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        mut options: HashMap<&str, ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let guild_id = interaction.guild_id.ok_or(Error::MissingGuildId)?;

        let Some(ResolvedValue::Channel(channel)) = options.remove("channel") else {
            unreachable!("Channel is required");
        };

        let role = match options.remove("role") {
            Some(ResolvedValue::Role(role)) => Some(role.id),
            _ => None,
        };

        Manager::insert(pool, guild_id, channel.id, role)
            .await
            .unwrap();

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new().content("LFG plugin has been setup"),
            )
            .await
            .unwrap();

        Ok(())
    }
}
