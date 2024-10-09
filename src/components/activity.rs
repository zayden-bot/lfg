use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
};

use crate::modal::create_modal;
use crate::Result;

pub struct ActivityComponent;

impl ActivityComponent {
    pub async fn run(ctx: &Context, interaction: &ComponentInteraction) -> Result<()> {
        let activity = match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => &values[0],
            _ => unreachable!("Activity is required"),
        };

        let modal = create_modal(activity);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await?;

        Ok(())
    }
}
