use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serenity::all::{
    AutoArchiveDuration, ChannelId, Context, CreateForumPost, CreateMessage, CreateModal,
    Mentionable, ModalInteraction,
};
use sqlx::Pool;
use zayden_core::parse_modal_data;

use crate::TimezoneManager;
use crate::{create_lfg_embed, create_main_row, LfgPostManager, LfgPostRow, Result};

use super::{modal_components, MAX_FIRETEAM_SIZE};

const LFG_CHANNEL: ChannelId = ChannelId::new(1091736203029659728);

pub struct LfgCreateModal;

impl LfgCreateModal {
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

        let mut post = LfgPostRow::new(
            1,
            interaction.user.id,
            activity,
            start_time,
            description,
            fireteam_size,
        );

        let embed = create_lfg_embed(&post, &interaction.user.name);

        let row = create_main_row();

        let channel = LFG_CHANNEL
            .create_forum_post(
                ctx,
                CreateForumPost::new(
                    format!("{} - {}", activity, start_time.format("%d %b %H:%M %Z")),
                    CreateMessage::new().embed(embed).components(vec![row]),
                )
                .auto_archive_duration(AutoArchiveDuration::OneWeek),
            )
            .await?;

        // TODO: Add thread tags based on description

        channel
            .send_message(
                ctx,
                CreateMessage::new().content(interaction.user.mention().to_string()),
            )
            .await?;

        post.id = channel.id.get() as i64;

        post.save::<Db, PostManager>(pool).await?;

        Ok(())
    }
}

pub fn create_modal(activity: &str, timezone: &Tz) -> CreateModal {
    let fireteam_size = match MAX_FIRETEAM_SIZE.get(activity) {
        Some(fireteam_size) => *fireteam_size,
        None => 3,
    };

    let now = timezone.from_utc_datetime(&Utc::now().naive_utc());

    let row = modal_components(activity, now, fireteam_size, None);

    CreateModal::new("lfg_create", "Create Event").components(row)
}
