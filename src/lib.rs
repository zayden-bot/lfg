mod components;
mod error;
mod lfg_post_manager;
mod modal;
mod slash_command;

use std::collections::HashSet;

pub use components::{ActivityComponent, PostComponents};
pub use error::Error;
use error::Result;
pub use lfg_post_manager::LfgPostManager;
pub use modal::LfgCreateModal;
use serenity::all::{CreateEmbed, CreateEmbedFooter, Mentionable, UserId};
pub use slash_command::LfgCommand;

fn create_lfg_embed(
    activity: &str,
    timestamp: i64,
    description: &str,
    fireteam: &HashSet<UserId>,
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
