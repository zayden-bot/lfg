use chrono::{NaiveDateTime, TimeZone};
use serenity::all::{
    Context, CreateInteractionResponse, EditMessage, EditThread, ModalInteraction,
};
use sqlx::Pool;
use zayden_core::parse_modal_data;

use crate::{create_lfg_embed, LfgPostManager, Result, TimezoneManager};

pub struct LfgEditModal;

impl LfgEditModal {
    pub async fn run<Db, PostManager, TzManager>(
        ctx: &Context,
        interaction: &ModalInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        PostManager: LfgPostManager<Db>,
        TzManager: TimezoneManager<Db>,
    {
        let mut inputs = parse_modal_data(&interaction.data.components);

        let activity = inputs
            .remove("activity")
            .expect("Activity should exist as it's required");
        let fireteam_size = inputs
            .remove("fireteam size")
            .expect("Fireteam size should exist as it's required")
            .parse::<u8>()?;
        let description = match inputs.remove("description") {
            Some(description) => description,
            None => activity,
        };
        let start_time_str = inputs
            .remove("start time")
            .expect("Start time should exist as it's required");

        let timezone = TzManager::get(pool, interaction.user.id, &interaction.locale).await?;

        let start_time = {
            let naive_dt = NaiveDateTime::parse_from_str(start_time_str, "%Y-%m-%d %H:%M")?;
            timezone
                .from_local_datetime(&naive_dt)
                .single()
                .expect("Invalid date time")
        };

        let channel_id = interaction.channel_id;

        let mut post = PostManager::get(pool, interaction.message.as_ref().unwrap().id).await?;
        post.activity = activity.to_string();
        post.fireteam_size = fireteam_size as i16;
        post.description = description.to_string();
        post.timestamp = start_time.naive_utc();
        post.timezone = timezone.name().to_string();

        let embed = create_lfg_embed(&post, &interaction.user.name);

        channel_id
            .edit_thread(
                ctx,
                EditThread::new().name(format!(
                    "{} - {}",
                    activity,
                    start_time.format("%d %b %H:%M %Z")
                )),
            )
            .await?;

        channel_id
            .edit_message(ctx, channel_id.get(), EditMessage::new().embed(embed))
            .await?;

        post.save::<Db, PostManager>(pool).await?;

        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await?;

        Ok(())
    }
}
