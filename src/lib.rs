mod components;
mod error;
mod lfg_post_manager;
mod modals;
mod slash_command;
pub mod timezone_manager;

pub use components::{KickComponent, PostComponents, SettingsComponents};
pub use error::Error;
use error::Result;
pub use lfg_post_manager::{LfgPostManager, LfgPostRow};
pub use modals::{LfgCreateModal, LfgEditModal};
use serenity::all::{
    ButtonStyle, Context, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    Mentionable, UserId,
};
pub use slash_command::LfgCommand;
use sqlx::{Database, Pool};
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

async fn join_post<Db: Database, Manager: LfgPostManager<Db>>(
    ctx: &Context,
    pool: &Pool<Db>,
    mut post: LfgPostRow,
    user_id: impl Into<UserId>,
) -> Result<CreateEmbed> {
    if post.is_full() {
        return Err(Error::FireteamFull);
    }

    post.join(user_id);

    let embed = create_lfg_embed(&post, &post.owner(ctx).await?.name);

    post.save::<Db, Manager>(pool).await?;

    Ok(embed)
}
