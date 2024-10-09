use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Mentionable,
};

use crate::modal::create_modal;
use crate::{Error, LfgPostManager, Result};

pub struct PostComponents;

impl PostComponents {
    pub async fn join(ctx: &Context, interaction: &ComponentInteraction) -> Result<()> {
        let mut data = ctx.data.write().await;
        let manager = data
            .get_mut::<LfgPostManager>()
            .expect("Expected LfgPostManager in TypeMap");

        let post = manager.get_mut(&interaction.message.id).unwrap();

        if (post.fireteam.len() as u8) == post.fireteam_size {
            return Err(Error::FireteamFull);
        }

        post.fireteam.insert(interaction.user.id);

        let mut embed = interaction.message.embeds[0].clone();
        let field = embed.fields.last_mut().unwrap();
        field.value = post
            .fireteam
            .iter()
            .map(|id| id.mention().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(embed.into()),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn leave(ctx: &Context, interaction: &ComponentInteraction) {}

    pub async fn alternative(ctx: &Context, interaction: &ComponentInteraction) {}
}
