use std::collections::HashMap;
use std::str::FromStr;

use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, CommandOptionType, Context, CreateAutocompleteResponse,
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateModal, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditInteractionResponse, EditMessage,
    Mentionable, ResolvedValue,
};
use sqlx::{Database, Pool};
use zayden_core::parse_options;

use crate::modals::modal_components;
use crate::timezone_manager::TimezoneManager;
use crate::{join_post, leave_post, Error, LfgGuildManager, LfgPostManager, Result, ACTIVITIES};

pub struct LfgCommand;

impl LfgCommand {
    pub async fn run<Db, GuildManager, PostManager, TzManager>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        GuildManager: LfgGuildManager<Db>,
        PostManager: LfgPostManager<Db>,
        TzManager: TimezoneManager<Db>,
    {
        let command = &interaction.data.options()[0];

        let options = match &command.value {
            ResolvedValue::SubCommand(options) => options,
            ResolvedValue::SubCommandGroup(options) => options,
            _ => unreachable!("Subcommand is required"),
        };
        let options = parse_options(options);

        match command.name {
            "setup" => Self::setup::<Db, GuildManager>(ctx, interaction, pool, options).await?,
            "create" => Self::create::<Db, TzManager>(ctx, interaction, pool, options).await?,
            "tags" => Self::tags::<Db, PostManager>(ctx, interaction, pool).await,
            "join" => Self::join::<Db, PostManager>(ctx, interaction, pool, options).await?,
            "leave" => Self::leave::<Db, PostManager>(ctx, interaction, pool, options).await?,
            "joined" => Self::joined(ctx, interaction).await,
            "timezone" => Self::timezone::<Db, TzManager>(ctx, interaction, pool, options).await?,
            _ => unreachable!("Invalid subcommand"),
        }

        Ok(())
    }

    async fn setup<Db: Database, Manager: LfgGuildManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        options: HashMap<&str, &ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await?;

        let guild_id = match interaction.guild_id {
            Some(guild_id) => guild_id,
            None => return Err(Error::GuildRequired),
        };

        let channel = match options.get("channel") {
            Some(ResolvedValue::Channel(channel)) => channel.id,
            _ => unreachable!("Channel is required"),
        };

        let role = match options.get("role") {
            Some(ResolvedValue::Role(role)) => Some(role.id),
            _ => None,
        };

        Manager::save(pool, guild_id, channel, role).await?;

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new().content("LFG plugin has been setup"),
            )
            .await?;

        Ok(())
    }

    async fn create<Db: Database, Manager: TimezoneManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        options: HashMap<&str, &ResolvedValue<'_>>,
    ) -> Result<()> {
        let activity = match options.get("activity") {
            Some(ResolvedValue::String(activity)) => *activity,
            _ => unreachable!("Activity is required"),
        };

        let timezone = Manager::get(pool, interaction.user.id, &interaction.locale).await?;
        let now = timezone.from_utc_datetime(&Utc::now().naive_utc());

        let fireteam_size = match ACTIVITIES.iter().find(|a| a.name == activity) {
            Some(activity) => activity.fireteam_size,
            None => 3,
        };

        let row = modal_components(activity, now, fireteam_size, None);

        let modal = CreateModal::new("lfg_create", "Create Event").components(row);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await?;

        Ok(())
    }

    async fn tags<Db: Database, Manager: LfgPostManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) {
        let command = &interaction.data.options();
        println!("{:?}", command);
    }

    async fn add_tags<Db: Database, Manager: LfgPostManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let post = Manager::get(pool, interaction.channel_id.get())
            .await
            .unwrap();

        if post.owner_id() != interaction.user.id {
            return Err(Error::PermissionDenied {
                owner: post.owner_id(),
            });
        }

        let thread_channel = interaction
            .channel_id
            .to_channel(ctx)
            .await
            .unwrap()
            .guild()
            .unwrap();

        let forum_channel = thread_channel
            .parent_id
            .unwrap()
            .to_channel(ctx)
            .await
            .unwrap()
            .guild()
            .unwrap();

        let options = forum_channel
            .available_tags
            .into_iter()
            .filter(|tag| !thread_channel.applied_tags.contains(&tag.id))
            .map(|tag| CreateSelectMenuOption::new(tag.name, tag.id.to_string()))
            .collect::<Vec<_>>();

        let max_values = options.len() as u8;

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new().select_menu(
                    CreateSelectMenu::new("lfg_tags", CreateSelectMenuKind::String { options })
                        .max_values(max_values),
                ),
            )
            .await
            .unwrap();

        Ok(())
    }

    async fn remove_tags<Db: Database, Manager: LfgPostManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let post = Manager::get(pool, interaction.channel_id.get())
            .await
            .unwrap();

        if post.owner_id() != interaction.user.id {
            return Err(Error::PermissionDenied {
                owner: post.owner_id(),
            });
        }

        let thread_channel = interaction
            .channel_id
            .to_channel(ctx)
            .await
            .unwrap()
            .guild()
            .unwrap();

        let forum_channel = thread_channel
            .parent_id
            .unwrap()
            .to_channel(ctx)
            .await
            .unwrap()
            .guild()
            .unwrap();

        let options = forum_channel
            .available_tags
            .into_iter()
            .filter(|tag| thread_channel.applied_tags.contains(&tag.id))
            .map(|tag| CreateSelectMenuOption::new(tag.name, tag.id.to_string()))
            .collect::<Vec<_>>();

        let max_values = options.len() as u8;

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new().select_menu(
                    CreateSelectMenu::new("lfg_tags", CreateSelectMenuKind::String { options })
                        .max_values(max_values),
                ),
            )
            .await
            .unwrap();

        Ok(())
    }

    async fn join<Db: Database, Manager: LfgPostManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        options: HashMap<&str, &ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await?;

        let thread_id = match options.get("channel") {
            Some(ResolvedValue::Channel(channel)) => channel.id,
            _ => unreachable!("Thread is required"),
        };

        let post = Manager::get(pool, thread_id.get()).await?;

        let embed = join_post::<Db, Manager>(ctx, pool, post, interaction.user.id).await?;

        thread_id
            .edit_message(ctx, thread_id.get(), EditMessage::new().embed(embed))
            .await?;

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .content(format!("You have joined {}", thread_id.mention())),
            )
            .await?;

        Ok(())
    }

    async fn leave<Db: Database, Manager: LfgPostManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        options: HashMap<&str, &ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await?;

        let thread_id = match options.get("channel") {
            Some(ResolvedValue::Channel(channel)) => channel.id,
            _ => unreachable!("Thread is required"),
        };

        let post = Manager::get(pool, thread_id.get()).await?;

        let embed = leave_post::<Db, Manager>(ctx, pool, post, interaction.user.id).await?;

        thread_id
            .edit_message(ctx, thread_id.get(), EditMessage::new().embed(embed))
            .await?;

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .content(format!("You have left {}", thread_id.mention())),
            )
            .await?;

        Ok(())
    }

    async fn joined(ctx: &Context, interaction: &CommandInteraction) {
        todo!()
    }

    async fn timezone<Db: Database, Manager: TimezoneManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        options: HashMap<&str, &ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await?;

        let timezone = match options.get("region") {
            Some(ResolvedValue::String(region)) => *region,
            _ => unreachable!("Region is required"),
        };

        let tz = Tz::from_str(timezone)?;

        Manager::save(pool, interaction.user.id, tz).await?;

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .content(format!("Your timezone has been set to {}", tz.name())),
            )
            .await?;

        Ok(())
    }

    pub fn register() -> CreateCommand {
        let setup = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "setup",
            "Setup the lfg plugin",
        )
        .add_sub_option(
            CreateCommandOption::new(
                CommandOptionType::Channel,
                "channel",
                "The channel to create the lfg threads in",
            )
            .required(true),
        )
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::Role,
            "role",
            "The role to mention when a new lfg thread is created",
        ));

        let create = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "create",
            "Create a new looking for group post",
        )
        .add_sub_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "activity",
                "The activity you are looking to do",
            )
            .required(true)
            .set_autocomplete(true),
        );

        let tags = CreateCommandOption::new(
            CommandOptionType::SubCommandGroup,
            "tags",
            "Edit the tags for the lfg post",
        )
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "add",
            "Add tags to the lfg post",
        ))
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "remove",
            "Remove tags from the lfg post",
        ));

        let join = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "join",
            "Join a looking for group post",
        )
        .add_sub_option(
            CreateCommandOption::new(CommandOptionType::Channel, "thread", "The LFG thread")
                .required(true),
        );

        let leave = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "leave",
            "Leave a looking for group post",
        )
        .add_sub_option(
            CreateCommandOption::new(CommandOptionType::Channel, "thread", "The LFG thread")
                .required(true),
        );

        let timezone = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "timezone",
            "Set your timezone",
        )
        .add_sub_option(
            CreateCommandOption::new(CommandOptionType::String, "region", "Your region")
                .required(true)
                .set_autocomplete(true),
        );

        CreateCommand::new("lfg")
            .description("Create a looking for group post")
            .add_option(setup)
            .add_option(create)
            .add_option(tags)
            .add_option(join)
            .add_option(leave)
            // .add_option(CreateCommandOption::new(
            //     CommandOptionType::SubCommand,
            //     "joined",
            //     "View all the posts you have joined",
            // ))
            .add_option(timezone)
    }

    pub async fn autocomplete(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
        let command = &interaction.data.options()[0];

        let filtered = match command.name {
            "create" => {
                let option = interaction.data.autocomplete().unwrap();

                ACTIVITIES
                    .iter()
                    .filter(|activity| {
                        activity
                            .name
                            .to_lowercase()
                            .starts_with(&option.value.to_lowercase())
                    })
                    .take(25)
                    .map(|activity| AutocompleteChoice::new(activity.name, activity.name))
                    .collect::<Vec<_>>()
            }

            "timezone" => {
                let option = interaction.data.autocomplete().unwrap();

                chrono_tz::TZ_VARIANTS
                    .iter()
                    .filter(|tz| {
                        let name = tz.name().to_lowercase();
                        name.contains(&option.value.to_lowercase())
                    })
                    .take(25)
                    .map(|tz| AutocompleteChoice::new(tz.name(), tz.name()))
                    .collect::<Vec<_>>()
            }
            _ => return Ok(()),
        };

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Autocomplete(
                    CreateAutocompleteResponse::new().set_choices(filtered),
                ),
            )
            .await?;

        Ok(())
    }
}
