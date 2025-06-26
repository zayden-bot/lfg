use serenity::all::{Context, CreateInteractionResponse, EditThread, ModalInteraction};
use sqlx::{Database, Pool};
use zayden_core::parse_modal_data;

use crate::templates::DefaultTemplate;
use crate::utils::update_embeds;
use crate::{PostBuilder, PostManager, Result, TimezoneManager};

use super::start_time;

pub struct Edit;

impl Edit {
    pub async fn run<Db: Database, Manager: PostManager<Db>, TzManager: TimezoneManager<Db>>(
        ctx: &Context,
        interaction: &ModalInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        let mut inputs = parse_modal_data(&interaction.data.components);

        let activity = inputs
            .remove("activity")
            .expect("Activity should exist as it's required");
        let fireteam_size = inputs
            .remove("fireteam size")
            .expect("Fireteam size should exist as it's required")
            .parse::<i16>()
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

        let post = PostBuilder::from(Manager::row(pool, interaction.channel_id).await.unwrap())
            .activity(activity)
            .fireteam_size(fireteam_size)
            .description(description)
            .timestamp(start_time);

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

        update_embeds::<DefaultTemplate>(
            ctx,
            &post,
            interaction.user.display_name(),
            interaction.channel_id,
        )
        .await;

        Manager::save(pool, post.build()).await.unwrap();

        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
            .unwrap();

        Ok(())
    }
}
