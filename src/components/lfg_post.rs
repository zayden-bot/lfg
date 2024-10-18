use serenity::all::{
    ComponentInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::{create_lfg_embed, Error, LfgPostManager, Result};

pub struct PostComponents;

impl PostComponents {
    pub async fn join<Db, Manager>(ctx: &Context, interaction: &ComponentInteraction) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        let mut post = Manager::get(&interaction.message.id).await?;

        if post.is_full() {
            return Err(Error::FireteamFull);
        }

        post.join(interaction.user.id);

        let embed = create_lfg_embed(
            &post.activity,
            post.start_time.timestamp(),
            &post.description,
            &post.fireteam,
            post.fireteam_size,
            &post.owner.to_user(ctx).await?.name,
        );

        Manager::save(post).await?;

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

    pub async fn leave<Db, Manager>(ctx: &Context, interaction: &ComponentInteraction) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
    {
        let mut post = Manager::get(&interaction.message.id).await?;

        post.leave(interaction.user.id);

        let embed = create_lfg_embed(
            &post.activity,
            post.start_time.timestamp(),
            &post.description,
            &post.fireteam,
            post.fireteam_size,
            &post.owner.to_user(ctx).await?.name,
        );

        Manager::save(post).await?;

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
