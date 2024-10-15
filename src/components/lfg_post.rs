use serenity::all::{
    ComponentInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::{create_lfg_embed, Error, LfgPostManager, Result};

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

        let embed = create_lfg_embed(
            &post.activity,
            post.start_time.timestamp(),
            &post.description,
            &post.fireteam,
            post.fireteam_size,
            &post.owner.to_user(ctx).await?.name,
        );

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn leave(ctx: &Context, interaction: &ComponentInteraction) -> Result<()> {
        let mut data = ctx.data.write().await;
        let manager = data
            .get_mut::<LfgPostManager>()
            .expect("Expected LfgPostManager in TypeMap");

        let post = manager
            .get_mut(&interaction.message.id)
            .ok_or(Error::PostNotFound)?;

        let changed = post.fireteam.remove(&interaction.user.id);

        if !changed {
            interaction
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await?;
            return Ok(());
        }

        let embed = create_lfg_embed(
            &post.activity,
            post.start_time.timestamp(),
            &post.description,
            &post.fireteam,
            post.fireteam_size,
            &post.owner.to_user(ctx).await?.name,
        );

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn alternative(_ctx: &Context, _interaction: &ComponentInteraction) {
        todo!()
    }
}
