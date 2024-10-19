use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
};

use crate::modals::create;
use crate::Result;
use crate::TimezoneManager;

pub struct ActivityComponent;

impl ActivityComponent {
    pub async fn run(ctx: &Context, interaction: &ComponentInteraction) -> Result<()> {
        let activity = match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => &values[0],
            _ => unreachable!("Activity is required"),
        };

        let timezone = TimezoneManager::get(&interaction.locale).await;

        let modal = create::create_modal(activity, timezone);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await?;

        Ok(())
    }
}
