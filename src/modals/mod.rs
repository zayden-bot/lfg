pub mod create;
pub use create::LfgCreateModal;

mod edit;
pub use edit::LfgEditModal;

use chrono::DateTime;
use chrono_tz::Tz;
use serenity::all::{CreateActionRow, CreateInputText, InputTextStyle};

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
