use serenity::all::{
    AutoArchiveDuration, Context, CreateForumPost, CreateInteractionResponse, CreateMessage,
    DiscordJsonError, ErrorResponse, HttpError, Mentionable, ModalInteraction,
};
use sqlx::Pool;
use zayden_core::parse_modal_data;

use crate::{
    create_lfg_embed, create_main_row, Error, LfgPostManager, LfgPostRow, Result, ACTIVITIES,
};
use crate::{LfgGuildManager, TimezoneManager};

use super::start_time;

pub struct LfgCreateModal;

impl LfgCreateModal {
    pub async fn run<Db, GuildManager, PostManager, TzManager>(
        ctx: &Context,
        interaction: &ModalInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        GuildManager: LfgGuildManager<Db>,
        PostManager: LfgPostManager<Db>,
        TzManager: TimezoneManager<Db>,
    {
        let guild_id = interaction.guild_id.ok_or(Error::MissingGuildId)?;

        let mut inputs = parse_modal_data(&interaction.data.components);

        let activity = inputs
            .remove("activity")
            .expect("Activity should exist as it's required");
        let fireteam_size = inputs
            .remove("fireteam size")
            .expect("Fireteam size should exist as it's required")
            .parse::<u8>()
            .unwrap();
        let description = match inputs.remove("description") {
            Some(description) => &description.chars().take(1024).collect::<String>(),
            None => activity,
        };
        let start_time_str = inputs
            .remove("start time")
            .expect("Start time should exist as it's required");

        let timezone = TzManager::get(pool, interaction.user.id, &interaction.locale)
            .await
            .unwrap();

        let start_time = start_time(timezone, start_time_str)?;

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

        let lfg_guild = GuildManager::get(pool, guild_id)
            .await
            .unwrap()
            .ok_or(Error::MissingSetup)?;

        let channel = lfg_guild
            .channel_id()
            .to_channel(ctx)
            .await
            .unwrap()
            .guild()
            .unwrap();

        let tags = channel
            .available_tags
            .iter()
            .filter(|tag| {
                tag.name.to_lowercase()
                    == ACTIVITIES
                        .iter()
                        .find(|a| activity.to_lowercase().contains(&a.name.to_lowercase()))
                        .map(|a| a.category.to_string())
                        .unwrap_or_default()
                        .to_lowercase()
            })
            .map(|tag| tag.id);

        let thread = match channel
            .create_forum_post(
                ctx,
                CreateForumPost::new(
                    format!("{} - {}", activity, start_time.format("%d %b %H:%M %Z")),
                    CreateMessage::new()
                        .embed(embed.clone())
                        .components(vec![row]),
                )
                .auto_archive_duration(AutoArchiveDuration::OneWeek)
                .set_applied_tags(tags),
            )
            .await
        {
            Ok(thread) => thread,
            // A tag is required to create a thread
            Err(serenity::Error::Http(HttpError::UnsuccessfulRequest(ErrorResponse {
                error: DiscordJsonError { code: 40067, .. },
                ..
            }))) => {
                return Err(Error::TagRequired);
            }
            r => r.unwrap(),
        };

        thread
            .send_message(
                ctx,
                CreateMessage::new().content(interaction.user.mention().to_string()),
            )
            .await
            .unwrap();

        post.id = thread.id.get() as i64;

        post.clone().save::<Db, PostManager>(pool).await.unwrap();

        if let Some(thread_id) = lfg_guild.scheduled_thread_id() {
            thread_id
                .send_message(ctx, CreateMessage::new().embed(embed))
                .await
                .unwrap();
        }

        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
            .unwrap();

        Ok(())
    }
}
