use std::collections::HashMap;

use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use serenity::all::{
    ChannelId, Context, CreateActionRow, CreateForumPost, CreateInputText, CreateMessage,
    CreateModal, InputTextStyle, MessageId, ModalInteraction,
};
use zayden_core::parse_modal_data;

use crate::lfg_post_manager::LfgPostData;
use crate::slash_command::ACTIVITY_MAP;
use crate::{create_lfg_embed, create_main_row, LfgPostManager, Result};

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
        let fireteam_size = values[2].parse::<u8>()?;
        let description = match values.get(3) {
            Some(description) => *description,
            None => activity,
        };

        let naive_dt = NaiveDateTime::parse_from_str(values[1], "%Y-%m-%d %H:%M").unwrap();
        let start_time = naive_dt.and_utc();
        let timestamp = start_time.timestamp();

        let embed = create_lfg_embed(
            activity,
            timestamp,
            description,
            &[interaction.user.id].into_iter().collect(),
            fireteam_size,
            &interaction.user.name,
        );

        let row = create_main_row();

        let post = LFG_CHANNEL
            .create_forum_post(
                ctx,
                CreateForumPost::new(
                    format!("{} - {} UTC", activity, start_time.format("%d %b %H:%M")),
                    CreateMessage::new().embed(embed).components(vec![row]),
                ),
            )
            .await?;

        let mut data = ctx.data.write().await;
        let manager = data
            .get_mut::<LfgPostManager>()
            .expect("Expected LfgPostManager in TypeMap");

        manager.insert(
            MessageId::new(post.id.get()),
            LfgPostData::new(
                interaction.user.id,
                activity,
                start_time,
                description,
                fireteam_size,
            ),
        );

        Ok(())
    }
}
