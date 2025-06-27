use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime};
use chrono_tz::Tz;
use serenity::all::{
    ComponentInteraction, Context, CreateInteractionResponse, CreateModal, MessageId, UserId,
};
use sqlx::prelude::FromRow;
use sqlx::{Database, Pool};

use crate::modals::modal_components;
use crate::{Error, Result};

use super::Components;

#[async_trait]
pub trait EditManager<Db: Database> {
    async fn edit_row(pool: &Pool<Db>, id: impl Into<MessageId> + Send) -> sqlx::Result<EditRow>;
}

#[derive(FromRow)]
pub struct EditRow {
    pub owner_id: i64,
    pub activity: String,
    pub start_time: DateTime<Tz>,
    pub description: String,
    pub fireteam_size: i16,
}

impl EditRow {
    pub fn owner(&self) -> UserId {
        UserId::new(self.owner_id as u64)
    }

    pub fn start_time(&self) -> DateTime<Tz> {
        self.start_time
    }
}

impl Components {
    pub async fn edit<Db: Database, Manager: EditManager<Db>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        let post = Manager::edit_row(pool, interaction.message.id)
            .await
            .unwrap();

        if interaction.user.id != post.owner() {
            return Err(Error::PermissionDenied(post.owner()));
        }

        let row = modal_components(
            &post.activity,
            post.start_time(),
            post.fireteam_size,
            Some(&post.description),
        );

        let modal = CreateModal::new("lfg_edit", "Edit Event").components(row);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await
            .unwrap();

        Ok(())
    }
}
