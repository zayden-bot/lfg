use async_trait::async_trait;
use serenity::all::{
    AutoArchiveDuration, ChannelId, Context, CreateForumPost, CreateInteractionResponse,
    CreateMessage, DiscordJsonError, ErrorResponse, GuildId, HttpError, Mentionable,
    ModalInteraction,
};
use sqlx::any::AnyQueryResult;
use sqlx::{Database, Pool};
use zayden_core::parse_modal_data;

use crate::templates::{DefaultTemplate, Template};
use crate::{ACTIVITIES, Error, PostBuilder, Result};
use crate::{PostRow, TimezoneManager};

use super::start_time;

#[async_trait]
pub trait CreateManager<Db: Database> {
    async fn guild(pool: &Pool<Db>, id: impl Into<GuildId>) -> sqlx::Result<Option<GuildRow>>;

    async fn save(pool: &Pool<Db>, row: PostRow) -> sqlx::Result<AnyQueryResult>;
}

pub struct GuildRow {
    channel_id: i64,
    scheduled_thread_id: Option<i64>,
}

impl GuildRow {
    pub fn channel_id(&self) -> ChannelId {
        ChannelId::new(self.channel_id as u64)
    }

    pub fn scheduled_thread_id(&self) -> Option<ChannelId> {
        self.scheduled_thread_id.map(|id| ChannelId::new(id as u64))
    }
}

pub struct Create;

impl Create {
    pub async fn run<Db: Database, Manager: CreateManager<Db>, TzManager: TimezoneManager<Db>>(
        ctx: &Context,
        interaction: &ModalInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        let guild_id = interaction.guild_id.ok_or(Error::MissingGuildId)?;

        let mut inputs = parse_modal_data(&interaction.data.components);

        let activity = inputs
            .remove("activity")
            .expect("Activity should exist as it's required");
        let fireteam_size = inputs
            .remove("fireteam size")
            .expect("Fireteam size should exist as it's required")
            .parse::<i16>()
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

        let mut post = PostBuilder::new(
            interaction.user.id,
            activity,
            start_time,
            description,
            fireteam_size as i16,
        );

        let embed = DefaultTemplate::thread_embed(&post, interaction.user.display_name());
        let row = DefaultTemplate::main_row();

        let lfg_guild = Manager::guild(pool, guild_id)
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
                    CreateMessage::new().embed(embed).components(vec![row]),
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

        post = post.id(thread.id);

        let embed =
            DefaultTemplate::message_embed(&post, interaction.user.display_name(), thread.id);

        if let Some(thread_id) = lfg_guild.scheduled_thread_id() {
            let msg = thread_id
                .send_message(ctx, CreateMessage::new().embed(embed))
                .await
                .unwrap();

            post = post.message(thread_id, msg.id)
        }

        Manager::save(pool, post.build()).await.unwrap();

        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await
            .unwrap();

        Ok(())
    }
}
