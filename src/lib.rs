mod components;
mod error;
mod lfg_post_manager;
mod modals;
mod slash_command;
pub mod timezone_manager;

use serenity::all::{
    ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, Mentionable,
};

pub use components::{ActivityComponent, KickComponent, PostComponents, SettingsComponents};
pub use error::Error;
use error::Result;
pub use lfg_post_manager::{LfgPostManager, LfgPostRow};
pub use modals::{LfgCreateModal, LfgEditModal};
pub use slash_command::LfgCommand;
pub use timezone_manager::TimezoneManager;

fn create_lfg_embed(post: &LfgPostRow, owner_name: &str) -> CreateEmbed {
    let timestamp = post.timestamp();
    let fireteam = post.fireteam();
    let alternatives = post.alternatives();

    let fireteam_str = fireteam
        .iter()
        .map(|id| id.mention().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let mut embed = CreateEmbed::new()
        .title(format!("{} - <t:{}>", &post.activity, timestamp))
        .field("Activity", &post.activity, true)
        .field("Start Time", format!("<t:{}:R>", timestamp), true)
        .field("Description", &post.description, false)
        .field(
            format!("Joined: {}/{}", fireteam.len(), post.fireteam_size()),
            fireteam_str,
            false,
        )
        .footer(CreateEmbedFooter::new(format!("Posted by {}", owner_name)));

    if !alternatives.is_empty() {
        let alternatives_str = alternatives
            .iter()
            .map(|id| id.mention().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        embed = embed.field("Alternatives", alternatives_str, true);
    }

    embed
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
            .style(ButtonStyle::Secondary),
        CreateButton::new("lfg_settings")
            .emoji('⚙')
            .style(ButtonStyle::Secondary),
    ])
}
