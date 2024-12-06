use chrono::{NaiveDateTime, TimeZone};
use serenity::all::{
    AutoArchiveDuration, Context, CreateForumPost, CreateMessage, Mentionable, ModalInteraction,
};
use sqlx::Pool;
use zayden_core::parse_modal_data;

use crate::{create_lfg_embed, create_main_row, Error, LfgPostManager, LfgPostRow, Result};
use crate::{LfgGuildManager, TimezoneManager};

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
        let guild_id = match interaction.guild_id {
            Some(guild_id) => guild_id,
            None => return Err(Error::GuildRequired),
        };

        let mut inputs = parse_modal_data(&interaction.data.components);

        let activity = inputs
            .remove("activity")
            .expect("Activity should exist as it's required");
        let fireteam_size = inputs
            .remove("fireteam size")
            .expect("Fireteam size should exist as it's required")
            .parse::<u8>()?;
        let description = match inputs.remove("description") {
            Some(description) => description,
            None => activity,
        };
        let start_time_str = inputs
            .remove("start time")
            .expect("Start time should exist as it's required");

        let timezone = TzManager::get(pool, interaction.user.id, &interaction.locale).await?;

        let start_time = {
            let naive_dt = NaiveDateTime::parse_from_str(start_time_str, "%Y-%m-%d %H:%M")?;
            timezone
                .from_local_datetime(&naive_dt)
                .single()
                .expect("Invalid date time")
        };

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

        let lfg_guild = GuildManager::get(pool, guild_id).await?;
        let channel_id = match lfg_guild {
            Some(guild) => guild.channel_id(),
            None => return Err(Error::MissingSetup),
        };

        let thread = channel_id
            .create_forum_post(
                ctx,
                CreateForumPost::new(
                    format!("{} - {}", activity, start_time.format("%d %b %H:%M %Z")),
                    CreateMessage::new().embed(embed).components(vec![row]),
                )
                .auto_archive_duration(AutoArchiveDuration::OneWeek),
            )
            .await?;

        // TODO: Add thread tags based on description

        thread
            .send_message(
                ctx,
                CreateMessage::new().content(interaction.user.mention().to_string()),
            )
            .await?;

        post.id = thread.id.get() as i64;

        post.save::<Db, PostManager>(pool).await?;

        Ok(())
    }
}
