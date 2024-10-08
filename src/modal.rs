use std::collections::HashMap;

use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use serenity::all::{
    ButtonStyle, ChannelId, Context, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateForumPost, CreateInputText, CreateMessage, CreateModal, InputTextStyle, Mentionable,
    ModalInteraction,
};
use zayden_core::parse_modal_data;

use crate::slash_command::ACTIVITY_MAP;
use crate::Result;

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

pub fn create_modal(activity: &str) -> CreateModal {
    let fireteam_size = match MAX_FIRETEAM_SIZE.get(activity) {
        Some(fireteam_size) => *fireteam_size,
        None => 3,
    };

    let row = vec![
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Activity", "activity").value(activity),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Start Time", "start time")
                .placeholder("YYYY-MM-DD HH:MM"),
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
    pub async fn run(ctx: &Context, interaction: &ModalInteraction) -> Result<()> {
        let values = parse_modal_data(&interaction.data.components);

        let activity = values[0];
        let description = match values.get(3) {
            Some(description) => *description,
            None => activity,
        };

        let naive_dt = NaiveDateTime::parse_from_str(values[1], "%Y-%m-%d %H:%M").unwrap();
        let timestamp = naive_dt.and_utc().timestamp();

        let embed = CreateEmbed::new()
            .title(format!("{} - <t:{}>", activity, timestamp))
            .field("Activity", activity, true)
            .field("Start Time", format!("<t:{}:R>", timestamp), true)
            .field("Description", description, false)
            .field(
                format!("Joined: 1/{}", values[2]),
                interaction.user.mention().to_string(),
                false,
            )
            .footer(CreateEmbedFooter::new(format!(
                "Posted by {}",
                interaction.user.name
            )));

        let buttons = vec![
            CreateButton::new("lfg_join")
                .emoji('➕')
                .style(ButtonStyle::Success),
            CreateButton::new("lfg_leave")
                .emoji('➖')
                .style(ButtonStyle::Danger),
            CreateButton::new("lfg_alternative")
                .emoji('🔄')
                .style(ButtonStyle::Secondary),
            CreateButton::new("lfg_settings")
                .emoji('⚙')
                .style(ButtonStyle::Primary),
        ];

        let row = vec![CreateActionRow::Buttons(buttons)];

        LFG_CHANNEL
            .create_forum_post(
                ctx,
                CreateForumPost::new(
                    format!(
                        "{} - {} UTC",
                        activity,
                        naive_dt.and_utc().format("%d %b %H:%M")
                    ),
                    CreateMessage::new().embed(embed).components(row),
                ),
            )
            .await?;

        Ok(())
    }
}
