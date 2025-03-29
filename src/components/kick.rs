use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
    EditMessage,
};
use serenity::all::{CreateInteractionResponseMessage, Mentionable};
use sqlx::Database;
use sqlx::Pool;

use crate::Result;
use crate::{create_lfg_embed, LfgPostManager};

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

        let mut post = Manager::get(pool, interaction.channel_id.get())
            .await
            .unwrap();

        if post.kick(user) {
            let embed =
                create_lfg_embed(&post, &interaction.user.name, Some(interaction.channel_id));

            let channel_id = post.channel_id();
            channel_id
                .edit_message(ctx, post.message_id(), EditMessage::new().embed(embed))
                .await
                .unwrap();

            post.save::<Db, Manager>(pool).await.unwrap();

            interaction
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await
                .unwrap();
        } else {
            interaction
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!("{} is not in the fireteam", user.mention()))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }

        Ok(())
    }
}
