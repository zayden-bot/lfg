use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
};
use sqlx::Database;
use sqlx::Pool;

use crate::modals::create;
use crate::Result;
use crate::TimezoneManager;

pub struct ActivityComponent;

impl ActivityComponent {
    pub async fn run<Db: Database, Manager: TimezoneManager<Db>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        let activity = match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => &values[0],
            _ => unreachable!("Activity is required"),
        };

        let timezone = Manager::get(pool, interaction.user.id, &interaction.locale).await?;

        let modal = create::create_modal(activity, &timezone);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await?;

        Ok(())
    }
}
