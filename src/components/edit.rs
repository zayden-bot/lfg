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
    async fn row(pool: &Pool<Db>, id: impl Into<MessageId>) -> sqlx::Result<EditRow>;
}

#[derive(FromRow)]
pub struct EditRow {
    pub owner_id: i64,
    pub activity: String,
    pub timestamp: NaiveDateTime,
    pub timezone: String,
    pub description: String,
    pub fireteam_size: i16,
}

impl EditRow {
    pub fn owner(&self) -> UserId {
        UserId::new(self.owner_id as u64)
    }

    pub fn start_time(&self) -> DateTime<Tz> {
        let timezone = self
            .timezone
            .parse::<Tz>()
            .expect("Should be a valid timezone");

        self.timestamp
            .and_local_timezone(timezone)
            .single()
            .unwrap()
    }
}

impl Components {
    pub async fn edit<Db: Database, Manager: EditManager<Db>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        let post = Manager::row(pool, interaction.message.id).await.unwrap();

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
