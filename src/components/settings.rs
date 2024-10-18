use chrono::{DateTime, FixedOffset};
use serenity::all::{
    ComponentInteraction, Context, CreateActionRow, CreateInputText, CreateModal, InputTextStyle,
};
use sqlx::Pool;

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

        let modal = create_edit_modal(
            &post.activity,
            post.start_time,
            post.fireteam_size(),
            &post.description,
        );

        interaction
            .create_response(ctx, serenity::all::CreateInteractionResponse::Modal(modal))
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

fn create_edit_modal(
    activity: &str,
    start_time: DateTime<FixedOffset>,
    fireteam_size: u8,
    description: &str,
) -> CreateModal {
    let row = vec![
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Activity", "activity").value(activity),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Short,
                format!("Start Time ({})", start_time.format("%Z")),
                "start time",
            )
            .value(format!("{}", start_time.format("%Y-%m-%d %H:%M"))),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Fireteam Size", "fireteam size")
                .value(fireteam_size.to_string()),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Paragraph, "Description", "description")
                .value(description)
                .required(false),
        ),
    ];

    CreateModal::new("lfg_edit", "Edit Event").components(row)
}
