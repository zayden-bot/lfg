use std::collections::HashMap;

use chrono::{NaiveDateTime, TimeZone, Utc};
use lazy_static::lazy_static;
use serenity::all::{
    AutoArchiveDuration, ChannelId, Context, CreateActionRow, CreateForumPost, CreateInputText,
    CreateMessage, CreateModal, InputTextStyle, Mentionable, ModalInteraction,
};
use sqlx::Pool;
use zayden_core::parse_modal_data;

use crate::slash_command::ACTIVITY_MAP;
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
    static ref LOCALE_TO_TIMEZONE: HashMap<&'static str, chrono_tz::Tz> = {
        let mut m = HashMap::new();
        m.insert("id", chrono_tz::Asia::Jakarta);
        m.insert("da", chrono_tz::Europe::Copenhagen);
        m.insert("de", chrono_tz::Europe::Berlin);
        m.insert("en-GB", chrono_tz::Europe::London);
        // m.insert("en-US", chrono_tz::America::New_York);
        m.insert("es-ES", chrono_tz::Europe::Madrid);
        m.insert("es-419", chrono_tz::America::Mexico_City);
        m.insert("fr", chrono_tz::Europe::Paris);
        m.insert("hr", chrono_tz::Europe::Zagreb);
        m.insert("it", chrono_tz::Europe::Rome);
        m.insert("lt", chrono_tz::Europe::Vilnius);
        m.insert("hu", chrono_tz::Europe::Budapest);
        m.insert("nl", chrono_tz::Europe::Amsterdam);
        m.insert("no", chrono_tz::Europe::Oslo);
        m.insert("pl", chrono_tz::Europe::Warsaw);
        m.insert("pt-BR", chrono_tz::America::Sao_Paulo);
        m.insert("ro", chrono_tz::Europe::Bucharest);
        m.insert("fi", chrono_tz::Europe::Helsinki);
        m.insert("sv-SE", chrono_tz::Europe::Stockholm);
        m.insert("vi", chrono_tz::Asia::Ho_Chi_Minh);
        m.insert("tr", chrono_tz::Europe::Istanbul);
        m.insert("cs", chrono_tz::Europe::Prague);
        m.insert("el", chrono_tz::Europe::Athens);
        m.insert("bg", chrono_tz::Europe::Sofia);
        m.insert("ru", chrono_tz::Europe::Moscow);
        m.insert("uk", chrono_tz::Europe::Kiev);
        m.insert("hi", chrono_tz::Asia::Kolkata);
        m.insert("th", chrono_tz::Asia::Bangkok);
        m.insert("zh-CN", chrono_tz::Asia::Shanghai);
        m.insert("ja", chrono_tz::Asia::Tokyo);
        m.insert("zh-TW", chrono_tz::Asia::Taipei);
        m.insert("ko", chrono_tz::Asia::Seoul);
        m
    };
}

pub fn create_modal(activity: &str, locale: &str) -> CreateModal {
    let fireteam_size = match MAX_FIRETEAM_SIZE.get(activity) {
        Some(fireteam_size) => *fireteam_size,
        None => 3,
    };

    let timezone = LOCALE_TO_TIMEZONE.get(locale).unwrap_or(&chrono_tz::UTC);
    let now_timezone = timezone.from_utc_datetime(&Utc::now().naive_utc());
    let tz_abbr = now_timezone.format("%Z").to_string();

    let row = vec![
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Activity", "activity").value(activity),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Short,
                format!("Start Time ({})", tz_abbr),
                format!("start time:{}", timezone),
            )
            .value(format!("{}", now_timezone.format("%Y-%m-%d %H:%M"))),
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

        let (start_time_id, start_time_value) = inputs
            .into_iter()
            .find(|(key, _)| key.starts_with("start time:"))
            .expect("Start time should exist as it's required");

        let timezone = {
            let start = start_time_id
                .find(':')
                .expect("Start time label should have a timezone");
            &start_time_id[(start + 1)..]
        }
        .parse::<chrono_tz::Tz>()?;

        let start_time = {
            let native_time =
                NaiveDateTime::parse_from_str(start_time_value, "%Y-%m-%d %H:%M").unwrap();
            timezone.from_local_datetime(&native_time).single().unwrap()
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
