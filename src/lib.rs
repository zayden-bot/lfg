mod components;
mod error;
mod lfg_post_manager;
mod modal;
mod slash_command;

pub use components::{ActivityComponent, PostComponents};
pub use error::Error;
use error::Result;
pub use lfg_post_manager::LfgPostManager;
pub use lfg_post_manager::LfgPostRow;
pub use modal::LfgCreateModal;
use serenity::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, Mentionable, UserId,
};
pub use slash_command::LfgCommand;

fn create_lfg_embed(
    activity: &str,
    timestamp: i64,
    description: &str,
    fireteam: &[UserId],
    fireteam_size: u8,
    owner_name: &str,
) -> CreateEmbed {
    let fireteam_str = fireteam
        .iter()
        .map(|id| id.mention().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    CreateEmbed::new()
        .title(format!("{} - <t:{}>", activity, timestamp))
        .field("Activity", activity, true)
        .field("Start Time", format!("<t:{}:R>", timestamp), true)
        .field("Description", description, false)
        .field(
            format!("Joined: {}/{}", fireteam.len(), fireteam_size),
            fireteam_str,
            false,
        )
        .footer(CreateEmbedFooter::new(format!("Posted by {}", owner_name)))
}

fn create_main_row() -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new("lfg_join")
            .emoji('➕')
            .style(ButtonStyle::Success),
        CreateButton::new("lfg_leave")
            .emoji('➖')
            .style(ButtonStyle::Danger),
        CreateButton::new("lfg_alternative")
            .emoji('❔')
            .style(ButtonStyle::Secondary)
            .disabled(true),
        CreateButton::new("lfg_settings")
            .emoji('⚙')
            .style(ButtonStyle::Secondary)
            .disabled(true),
    ])
}
