use serenity::all::{ComponentInteraction, Context, CreateInteractionResponse, CreateModal};
use sqlx::Pool;

use crate::modals::modal_components;
use crate::{LfgPostManager, Result};

pub struct SettingsComponents;

impl SettingsComponents {
    pub async fn edit<Db, Manager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        let post = Manager::get(pool, interaction.message.id).await?;

        let row = modal_components(
            &post.activity,
            post.start_time(),
            post.fireteam_size(),
            Some(&post.description),
        );

        let modal = CreateModal::new("lfg_edit", "Edit Event").components(row);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await?;

        Ok(())
    }

    pub async fn copy<Db, Manager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        Ok(())
    }

    pub async fn kick<Db, Manager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        Ok(())
    }

    pub async fn delete<Db, Manager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        Ok(())
    }
}
