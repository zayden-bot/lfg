use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse,
    EditMessage,
};
use serenity::all::{CreateInteractionResponseMessage, Mentionable};
use sqlx::Database;
use sqlx::Pool;

use crate::templates::{DefaultTemplate, Template};
use crate::{LfgMessageManager, LfgPostManager, LfgPostWithMessages, Result};

pub struct KickComponent;

impl KickComponent {
    pub async fn run<Db, PostManager, MessageManager>(
        ctx: &Context,
        interaction: &ComponentInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: Database,
        PostManager: LfgPostManager<Db> + Send,
        MessageManager: LfgMessageManager<Db>,
    {
        let user = match &interaction.data.kind {
            ComponentInteractionDataKind::UserSelect { values } => values[0],
            _ => unreachable!("User is required"),
        };

        let LfgPostWithMessages { mut post, messages } =
            PostManager::get_with_messages::<MessageManager>(pool, interaction.channel_id.get())
                .await
                .unwrap();

        if post.kick(user) {
            let embed = DefaultTemplate::embed(&post, &interaction.user.name, None);

            post.channel_id()
                .edit_message(ctx, post.message_id(), EditMessage::new().embed(embed))
                .await
                .unwrap();

            let embed =
                DefaultTemplate::embed(&post, &interaction.user.name, Some(post.channel_id()));

            post.save::<Db, PostManager>(pool).await.unwrap();

            for message in messages {
                message
                    .channel_id()
                    .edit_message(
                        ctx,
                        message.message_id(),
                        EditMessage::new().embed(embed.clone()),
                    )
                    .await
                    .unwrap();
            }

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
