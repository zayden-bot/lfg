use serenity::all::{
    ComponentInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateModal, CreateSelectMenu, CreateSelectMenuKind,
};
use sqlx::Pool;

use crate::modals::modal_components;
use crate::{Error, LfgPostManager, Result};

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
        let post = Manager::get(pool, interaction.message.id).await.unwrap();

        if interaction.user.id != post.owner_id() {
            return Err(Error::PermissionDenied(post.owner_id()));
        }

        let row = modal_components(
            &post.activity,
            post.start_time(),
            post.fireteam_size(),
            Some(&post.description),
        );

        let modal = CreateModal::new("lfg_edit", "Edit Event").components(row);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await
            .unwrap();

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
        let post = Manager::get(pool, interaction.message.id).await.unwrap();

        if interaction.user.id != post.owner_id() {
            return Err(Error::PermissionDenied(post.owner_id()));
        }

        let row = modal_components(
            &post.activity,
            post.start_time(),
            post.fireteam_size(),
            Some(&post.description),
        );

        let modal = CreateModal::new("lfg_create", "Copy Event").components(row);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await
            .unwrap();

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
        let post = Manager::get(pool, interaction.message.id).await.unwrap();

        if interaction.user.id != post.owner_id() {
            return Err(Error::PermissionDenied(post.owner_id()));
        }

        let select_menu = CreateSelectMenu::new(
            "lfg_kick_menu",
            CreateSelectMenuKind::User {
                default_users: None,
            },
        );

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Select the user you want to kick")
                        .select_menu(select_menu)
                        .ephemeral(true),
                ),
            )
            .await
            .unwrap();

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
        let post = Manager::get(pool, interaction.message.id).await.unwrap();

        if interaction.user.id != post.owner_id() {
            return Err(Error::PermissionDenied(post.owner_id()));
        }

        post.delete::<Db, Manager>(pool).await.unwrap();

        interaction.channel_id.delete(ctx).await.unwrap();

        Ok(())
    }
}
