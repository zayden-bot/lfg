use serenity::all::CreateInteractionResponseMessage;
use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
};
use sqlx::Database;
use sqlx::Pool;

use crate::LfgPostManager;
use crate::Result;

pub struct KickComponent;

impl KickComponent {
    pub async fn run<Db: Database, Manager: LfgPostManager<Db>>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        let user = match &interaction.data.kind {
            ComponentInteractionDataKind::UserSelect { values } => values[0],
            _ => unreachable!("User is required"),
        };

        let mut post = Manager::get(pool, interaction.message.id).await?;

        if post.kick(user) {
            post.save::<Db, Manager>(pool).await?;

            interaction
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await?;
        } else {
            interaction
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!("{} is not in the fireteam", user))
                            .ephemeral(true),
                    ),
                )
                .await?;
        }

        Ok(())
    }
}
