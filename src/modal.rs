use std::collections::HashMap;

use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use serenity::all::{
    AutoArchiveDuration, ChannelId, Context, CreateActionRow, CreateForumPost, CreateInputText,
    CreateMessage, CreateModal, InputTextStyle, Mentionable, ModalInteraction,
};
use sqlx::Pool;
use zayden_core::parse_modal_data;

use crate::slash_command::ACTIVITY_MAP;
use crate::TimezoneManager;
use crate::{create_lfg_embed, create_main_row, LfgPostManager, LfgPostRow, Result};

const LFG_CHANNEL: ChannelId = ChannelId::new(1091736203029659728);

lazy_static! {
    static ref MAX_FIRETEAM_SIZE: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();
        for activity in ACTIVITY_MAP["raid"].iter() {
            m.insert(*activity, 6);
        }
        m.insert("Crucible", 6);
        m.insert("Iron Banner", 6);
        m
    };
}

pub fn create_modal(activity: &str, timezone: &Tz) -> CreateModal {
    let fireteam_size = match MAX_FIRETEAM_SIZE.get(activity) {
        Some(fireteam_size) => *fireteam_size,
        None => 3,
    };

    let now = timezone.from_utc_datetime(&Utc::now().naive_utc());

    let row = vec![
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Activity", "activity").value(activity),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Short,
                format!("Start Time ({})", now.format("%Z")),
                "start time",
            )
            .value(format!("{}", now.format("%Y-%m-%d %H:%M"))),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Fireteam Size", "fireteam size")
                .value(fireteam_size.to_string()),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Paragraph, "Description", "description")
                .placeholder(activity)
                .required(false),
        ),
    ];

    CreateModal::new("lfg_create", "Create Event").components(row)
}

pub struct LfgCreateModal;

impl LfgCreateModal {
    pub async fn run<Db, Manager>(
        ctx: &Context,
        interaction: &ModalInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        Manager: LfgPostManager<Db>,
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

        let timezone = {
            let mut data = ctx.data.write().await;
            let manager = data
                .get_mut::<TimezoneManager>()
                .expect("TimezoneManager should exist");
            manager
                .remove(&interaction.user.id)
                .expect("User should have a timezone")
        };

        let start_time = {
            let naive_dt = NaiveDateTime::parse_from_str(start_time_str, "%Y-%m-%d %H:%M")?;
            timezone.from_utc_datetime(&naive_dt)
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

        post.save::<Db, Manager>(pool).await?;

        Ok(())
    }
}
