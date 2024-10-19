pub mod create;
pub use create::LfgCreateModal;

mod edit;
pub use edit::LfgEditModal;

use std::collections::HashMap;

use chrono::DateTime;
use chrono_tz::Tz;
use lazy_static::lazy_static;
use serenity::all::{CreateActionRow, CreateInputText, InputTextStyle};

use crate::slash_command::ACTIVITY_MAP;

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

pub fn modal_components(
    activity: &str,
    start_time: DateTime<Tz>,
    fireteam_size: u8,
    description: Option<&str>,
) -> Vec<CreateActionRow> {
    let mut desc_input =
        CreateInputText::new(InputTextStyle::Paragraph, "Description", "description")
            .required(false);
    desc_input = match description {
        Some(description) => desc_input.value(description),
        None => desc_input.placeholder(activity),
    };

    vec![
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Activity", "activity").value(activity),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Short,
                format!("Start Time ({})", start_time.format("%Z")),
                "start time",
            )
            .value(format!("{}", start_time.format("%Y-%m-%d %H:%M"))),
        ),
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Fireteam Size", "fireteam size")
                .value(fireteam_size.to_string()),
        ),
        CreateActionRow::InputText(desc_input),
    ]
}
//CreateModal::new("lfg_edit", "Edit Event").components(row)
