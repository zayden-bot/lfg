use serenity::all::{
    Context, CreateInteractionResponse, EditMessage, EditThread, ModalInteraction,
};
use sqlx::Pool;
use zayden_core::parse_modal_data;

use crate::{
    create_lfg_embed, Error, LfgMessageManager, LfgPostManager, LfgPostWithMessages, Result,
    TimezoneManager,
};

use super::start_time;

pub struct LfgEditModal;

impl LfgEditModal {
    pub async fn run<Db, PostManager, MessageManager, TzManager>(
        ctx: &Context,
        interaction: &ModalInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        PostManager: LfgPostManager<Db> + Send,
        MessageManager: LfgMessageManager<Db>,
        TzManager: TimezoneManager<Db>,
    {
        let mut inputs = parse_modal_data(&interaction.data.components);

        let activity = inputs
            .remove("activity")
            .expect("Activity should exist as it's required");
        let fireteam_size = inputs
            .remove("fireteam size")
            .expect("Fireteam size should exist as it's required")
            .parse::<u8>()
            .unwrap();
        let description = match inputs.remove("description") {
            Some(description) => description,
            None => activity,
        };
        let start_time_str = inputs
            .remove("start time")
            .expect("Start time should exist as it's required");

        let timezone = TzManager::get(pool, interaction.user.id, &interaction.locale)
            .await
            .unwrap();

        let start_time = start_time(timezone, start_time_str)?;

        let post = match PostManager::get_with_messages::<MessageManager>(
            pool,
            interaction.channel_id.get(),
        )
        .await
        {
            Ok(post) => post,
            Err(sqlx::Error::RowNotFound) => return Err(Error::InvalidChannel),
            r => r.unwrap(),
        };

        let LfgPostWithMessages { mut post, messages } = post;

        post.activity = activity.to_string();
        post.fireteam_size = fireteam_size as i16;
        post.description = description.to_string();
        post.timestamp = start_time.naive_utc();
        post.timezone = timezone.name().to_string();

        let embed = create_lfg_embed(&post, &interaction.user.name, None);

        interaction
            .channel_id
            .edit_thread(
                ctx,
                EditThread::new().name(format!(
                    "{} - {}",
                    activity,
                    start_time.format("%d %b %H:%M %Z")
                )),
            )
            .await
            .unwrap();

        interaction
            .channel_id
            .edit_message(
                ctx,
                interaction.channel_id.get(),
                EditMessage::new().embed(embed),
            )
            .await
            .unwrap();

        let embed = create_lfg_embed(&post, &interaction.user.name, Some(interaction.channel_id));

        post.save::<Db, PostManager>(pool).await.unwrap();

        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
            .unwrap();

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

        Ok(())
    }
}
