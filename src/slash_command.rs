use std::collections::HashMap;
use std::str::FromStr;

use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use serenity::all::{
    AutocompleteChoice, AutocompleteOption, CommandInteraction, CommandOptionType, Context,
    CreateAutocompleteResponse, CreateCommand, CreateCommandOption, CreateEmbed,
    CreateInteractionResponse, CreateModal, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, EditInteractionResponse, EditMessage, GuildChannel, Mentionable,
    ResolvedValue,
};
use sqlx::{Database, Pool};
use zayden_core::parse_options;

use crate::modals::modal_components;
use crate::timezone_manager::TimezoneManager;
use crate::{
    ACTIVITIES, Error, LfgGuildManager, LfgMessageManager, LfgPostManager, LfgPostWithMessages,
    Result, create_lfg_embed,
};

pub struct LfgCommand;

impl LfgCommand {
    pub async fn run<Db, GuildManager, PostManager, MessageManager, TzManager>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        GuildManager: LfgGuildManager<Db>,
        PostManager: LfgPostManager<Db> + Send,
        MessageManager: LfgMessageManager<Db>,
        TzManager: TimezoneManager<Db>,
    {
        let command = interaction.data.options().pop().unwrap();

        let options = match command.value {
            ResolvedValue::SubCommand(options) => options,
            ResolvedValue::SubCommandGroup(options) => options,
            _ => unreachable!("Subcommand is required"),
        };
        let options = parse_options(options);

        match command.name {
            "setup" => Self::setup::<Db, GuildManager>(ctx, interaction, pool, options).await?,
            "create" => Self::create::<Db, TzManager>(ctx, interaction, pool, options).await?,
            "tags" => Self::tags::<Db, PostManager>(ctx, interaction, pool, options).await?,
            "join" => {
                Self::join::<Db, PostManager, MessageManager>(ctx, interaction, pool, options)
                    .await?
            }
            "leave" => {
                Self::leave::<Db, PostManager, MessageManager>(ctx, interaction, pool, options)
                    .await?
            }
            "joined" => Self::joined::<Db, PostManager>(ctx, interaction, pool).await,
            "timezone" => Self::timezone::<Db, TzManager>(ctx, interaction, pool, options).await?,
            _ => unreachable!("Invalid subcommand"),
        }

        Ok(())
    }

    async fn setup<Db: Database, Manager: LfgGuildManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        mut options: HashMap<&str, ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let guild_id = interaction.guild_id.ok_or(Error::MissingGuildId)?;

        let Some(ResolvedValue::Channel(channel)) = options.remove("channel") else {
            unreachable!("Channel is required");
        };

        let role = match options.remove("role") {
            Some(ResolvedValue::Role(role)) => Some(role.id),
            _ => None,
        };

        Manager::save(pool, guild_id, channel.id, role)
            .await
            .unwrap();

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new().content("LFG plugin has been setup"),
            )
            .await
            .unwrap();

        Ok(())
    }

    async fn create<Db: Database, Manager: TimezoneManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        mut options: HashMap<&str, ResolvedValue<'_>>,
    ) -> Result<()> {
        let Some(ResolvedValue::String(activity)) = options.remove("activity") else {
            unreachable!("Activity is required");
        };

        let timezone = Manager::get(pool, interaction.user.id, &interaction.locale)
            .await
            .unwrap();
        let now = timezone.from_utc_datetime(&Utc::now().naive_utc());

        let fireteam_size = match ACTIVITIES.iter().find(|a| a.name == activity) {
            Some(activity) => activity.fireteam_size,
            None => 3,
        };

        let row = modal_components(activity, now, fireteam_size, None);

        let modal = CreateModal::new("lfg_create", "Create Event").components(row);

        interaction
            .create_response(ctx, CreateInteractionResponse::Modal(modal))
            .await
            .unwrap();

        Ok(())
    }

    async fn tags<Db: Database, Manager: LfgPostManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        options: HashMap<&str, ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let post = Manager::get(pool, interaction.channel_id.get())
            .await
            .unwrap();

        if post.owner_id() != interaction.user.id {
            return Err(Error::PermissionDenied(post.owner_id()));
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

        if options.contains_key("add") {
            Self::add_tags(ctx, interaction, forum_channel, thread_channel)
                .await
                .unwrap();
        } else if options.contains_key("remove") {
            Self::remove_tags(ctx, interaction, forum_channel, thread_channel)
                .await
                .unwrap();
        }

        Ok(())
    }

    async fn add_tags(
        ctx: &Context,
        interaction: &CommandInteraction,
        forum_channel: GuildChannel,
        thread_channel: GuildChannel,
    ) -> Result<()> {
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
                    CreateSelectMenu::new("lfg_tags_add", CreateSelectMenuKind::String { options })
                        .max_values(max_values),
                ),
            )
            .await
            .unwrap();

        Ok(())
    }

    async fn remove_tags(
        ctx: &Context,
        interaction: &CommandInteraction,
        forum_channel: GuildChannel,
        thread_channel: GuildChannel,
    ) -> Result<()> {
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
                    CreateSelectMenu::new(
                        "lfg_tags_remove",
                        CreateSelectMenuKind::String { options },
                    )
                    .max_values(max_values),
                ),
            )
            .await
            .unwrap();

        Ok(())
    }

    async fn join<
        Db: Database,
        PostManager: LfgPostManager<Db> + Send,
        MessageManager: LfgMessageManager<Db>,
    >(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        mut options: HashMap<&str, ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let thread = match options.remove("thread") {
            Some(ResolvedValue::Channel(thread)) => thread,
            _ => interaction.channel.as_ref().unwrap(),
        };

        let user_id = match options.remove("guardian") {
            Some(ResolvedValue::User(user, _)) => user.id,
            _ => interaction.user.id,
        };

        let alternative = match options.remove("alternate") {
            Some(ResolvedValue::Boolean(alternative)) => alternative,
            _ => false,
        };

        let LfgPostWithMessages { mut post, messages } =
            PostManager::get_with_messages::<MessageManager>(pool, interaction.channel_id.get())
                .await
                .unwrap();

        post.join(user_id, alternative)?;

        let owner_name = &post.owner(ctx).await.unwrap().name;
        let thread_embed = create_lfg_embed(&post, owner_name, None);
        let msg_embed = create_lfg_embed(&post, owner_name, Some(thread.id));

        post.save::<Db, PostManager>(pool).await.unwrap();

        thread
            .id
            .edit_message(ctx, thread.id.get(), EditMessage::new().embed(thread_embed))
            .await
            .unwrap();

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .content(format!("You have joined {}", thread.id.mention())),
            )
            .await
            .unwrap();

        for message in messages {
            message
                .channel_id()
                .edit_message(
                    ctx,
                    message.message_id(),
                    EditMessage::new().embed(msg_embed.clone()),
                )
                .await
                .unwrap();
        }

        Ok(())
    }

    async fn leave<Db, PostManager, MessageManager>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        mut options: HashMap<&str, ResolvedValue<'_>>,
    ) -> Result<()>
    where
        Db: sqlx::Database,
        PostManager: LfgPostManager<Db> + Send,
        MessageManager: LfgMessageManager<Db>,
    {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let Some(ResolvedValue::Channel(thread)) = options.remove("channel") else {
            unreachable!("Thread is required");
        };

        let LfgPostWithMessages { mut post, messages } =
            PostManager::get_with_messages::<MessageManager>(pool, interaction.channel_id.get())
                .await
                .unwrap();

        post.leave(interaction.user.id);

        let owner_name = &post.owner(ctx).await.unwrap().name;
        let thread_embed = create_lfg_embed(&post, owner_name, None);
        let msg_embed = create_lfg_embed(&post, owner_name, Some(thread.id));

        post.save::<Db, PostManager>(pool).await.unwrap();

        thread
            .id
            .edit_message(ctx, thread.id.get(), EditMessage::new().embed(thread_embed))
            .await
            .unwrap();

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .content(format!("You have left {}", thread.id.mention())),
            )
            .await
            .unwrap();

        for message in messages {
            message
                .channel_id()
                .edit_message(
                    ctx,
                    message.message_id(),
                    EditMessage::new().embed(msg_embed.clone()),
                )
                .await
                .unwrap();
        }

        Ok(())
    }

    async fn joined<Db: Database, Manager: LfgPostManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let posts = Manager::get_upcoming_by_user(pool, interaction.user.id)
            .await
            .unwrap();

        let (joined, alternative) = posts
            .into_iter()
            .partition::<Vec<_>, _>(|row| row.fireteam().contains(&interaction.user.id));

        let mut embed = CreateEmbed::new().title("Joined LFG Events");

        if !joined.is_empty() {
            let values = joined
                .into_iter()
                .map(|row| {
                    format!(
                        "{0}\n<t:{1}> (<t:{1}:R>)\n{2}",
                        row.activity,
                        row.timestamp(),
                        row.channel_id().mention()
                    )
                })
                .collect::<Vec<_>>();

            embed = embed.field("Joined Posts", values.join("\n\n"), false)
        }

        if !alternative.is_empty() {
            let values = alternative
                .into_iter()
                .map(|row| {
                    format!(
                        "{0}\n<t:{1}> (<t:{1}:R>)\n{2}",
                        row.activity,
                        row.timestamp(),
                        row.channel_id().mention()
                    )
                })
                .collect::<Vec<_>>();

            embed = embed.field("Alternative Posts", values.join("\n\n"), false)
        }

        interaction
            .edit_response(ctx, EditInteractionResponse::new().embed(embed))
            .await
            .unwrap();
    }

    async fn timezone<Db: Database, Manager: TimezoneManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
        mut options: HashMap<&str, ResolvedValue<'_>>,
    ) -> Result<()> {
        interaction.defer_ephemeral(ctx).await.unwrap();

        let Some(ResolvedValue::String(region)) = options.remove("region") else {
            unreachable!("Region is required");
        };

        let tz = Tz::from_str(region).unwrap();

        Manager::save(pool, interaction.user.id, tz).await.unwrap();

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .content(format!("Your timezone has been set to {}", tz.name())),
            )
            .await
            .unwrap();

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
        )
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::User,
            "guardian",
            "The guardian you want to join",
        ))
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "alternate",
            "Join as an alternate",
        ));

        let leave = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "leave",
            "Leave a looking for group post",
        )
        .add_sub_option(
            CreateCommandOption::new(CommandOptionType::Channel, "thread", "The LFG thread")
                .required(true),
        );

        let joined = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "joined",
            "View all the posts you have joined",
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
            .add_option(joined)
            .add_option(timezone)
    }

    pub async fn autocomplete(
        ctx: &Context,
        interaction: &CommandInteraction,
        option: AutocompleteOption<'_>,
    ) -> Result<()> {
        let command = &interaction.data.options().remove(0);

        let filtered = match command.name {
            "create" => ACTIVITIES
                .iter()
                .filter(|activity| {
                    activity
                        .name
                        .to_lowercase()
                        .starts_with(&option.value.to_lowercase())
                })
                .take(25)
                .map(|activity| AutocompleteChoice::new(activity.name, activity.name))
                .collect::<Vec<_>>(),

            "timezone" => chrono_tz::TZ_VARIANTS
                .iter()
                .filter(|tz| {
                    let name = tz.name().to_lowercase();
                    name.contains(&option.value.to_lowercase())
                })
                .take(25)
                .map(|tz| AutocompleteChoice::new(tz.name(), tz.name()))
                .collect::<Vec<_>>(),
            _ => return Ok(()),
        };

        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Autocomplete(
                    CreateAutocompleteResponse::new().set_choices(filtered),
                ),
            )
            .await
            .unwrap();

        Ok(())
    }
}
