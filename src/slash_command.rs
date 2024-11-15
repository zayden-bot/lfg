use chrono_tz::Tz;
use lazy_static::lazy_static;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, CommandOptionType, Context, CreateAutocompleteResponse,
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditInteractionResponse, ResolvedValue,
};
use sqlx::{Database, Pool};
use std::collections::HashMap;
use std::str::FromStr;
use zayden_core::parse_options;

use crate::modals::create;
use crate::timezone_manager::TimezoneManager;
use crate::Result;

lazy_static! {
    pub static ref ACTIVITY_MAP: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert(
            "raid",
            vec![
                "Salvation's Edge",
                "Crota's End",
                "Root of Nightmares",
                "King's Fall",
                "Vow of the Disciple",
                "Vault of Glass",
                "Deep Stone Crypt",
                "Garden of Salvation",
                "Last Wish",
            ],
        );
        m.insert(
            "dungeon",
            vec![
                "Vesper's Host",
                "Warlord's Ruin",
                "Ghosts of the Deep",
                "Spire of the Watcher",
                "Duality",
                "Grasp of Avarice",
                "Prophecy",
                "Pit of Heresy",
                "Shattered Throne",
            ],
        );
        m.insert(
            "exotic mission",
            vec![
                "The Whisper",
                "Zero Hour",
                "Harbinger",
                "Presage",
                "Vox Obscura",
                "Operation: Seraph's Shield",
                "Node.Ovrd.Avalon",
                "Starcrossed",
            ],
        );
        m.insert(
            "vanguard",
            vec!["Vanguard Ops", "Nightfall", "Grandmaster", "Onslaught"],
        );
        m.insert(
            "crucible",
            vec!["Crucible", "Competitive", "Iron Banner", "Trials of Osiris"],
        );
        m
    };
}

pub struct LfgCommand;

impl LfgCommand {
    pub async fn run<Db: Database, Manager: TimezoneManager<Db>>(
        ctx: &Context,
        interaction: &CommandInteraction,
        pool: &Pool<Db>,
    ) -> Result<()> {
        let command = &interaction.data.options()[0];

        let options = match &command.value {
            ResolvedValue::SubCommand(options) => options,
            _ => unreachable!("Subcommand is required"),
        };
        let options = parse_options(options);

        match command.name {
            "create" => Self::create::<Db, Manager>(ctx, interaction, pool, options).await?,
            "join" => Self::join(ctx, interaction, options).await,
            "leave" => Self::leave(ctx, interaction, options).await,
            "joined" => Self::joined(ctx, interaction).await,
            "timezone" => Self::timezone(ctx, interaction, options).await?,
            _ => unreachable!("Invalid subcommand"),
        }

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

        if let Some(sub_activity) = ACTIVITY_MAP.get(activity.split_whitespace().next().unwrap()) {
            let menu = CreateSelectMenu::new(
                "lfg_activity",
                CreateSelectMenuKind::String {
                    options: sub_activity
                        .iter()
                        .map(|a| CreateSelectMenuOption::new(*a, *a))
                        .collect(),
                },
            );

            interaction
                .edit_response(
                    ctx,
                    EditInteractionResponse::new()
                        .select_menu(menu)
                        .content("Select the activity you are looking to do"),
                )
                .await?;
        } else {
            interaction.delete_response(ctx).await?;

            let timezone = Manager::get(pool, interaction.user.id, &interaction.locale).await?;

            let modal = create::create_modal(activity, &timezone);

            interaction
                .create_response(ctx, CreateInteractionResponse::Modal(modal))
                .await?;
        }

        Ok(())
    }

    async fn join(
        ctx: &Context,
        interaction: &CommandInteraction,
        options: HashMap<&str, &ResolvedValue<'_>>,
    ) {
        todo!()
    }

    async fn leave(
        ctx: &Context,
        interaction: &CommandInteraction,
        options: HashMap<&str, &ResolvedValue<'_>>,
    ) {
        todo!()
    }

    async fn joined(ctx: &Context, interaction: &CommandInteraction) {
        todo!()
    }

    async fn timezone(
        ctx: &Context,
        interaction: &CommandInteraction,
        options: HashMap<&str, &ResolvedValue<'_>>,
    ) -> Result<()> {
        let timezone = match options.get("region") {
            Some(ResolvedValue::String(region)) => *region,
            _ => unreachable!("Region is required"),
        };

        let tz = Tz::from_str(timezone).unwrap();

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
        CreateCommand::new("lfg")
            .description("Create a looking for group post")
            .add_option(
                CreateCommandOption::new(
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
                    .add_string_choice("Raid", "raid")
                    .add_string_choice("Dungeon", "dungeon")
                    .add_string_choice("Exotic Mission", "exotic mission")
                    .add_string_choice("Vangard", "vanguard")
                    .add_string_choice("Gambit", "gambit")
                    .add_string_choice("Crucible", "crucible")
                    .add_string_choice("Seasonal", "seasonal")
                    .add_string_choice("Other", "other"),
                ),
            )
            // .add_option(CreateCommandOption::new(
            //     CommandOptionType::SubCommand,
            //     "join",
            //     "Join a looking for group post",
            // ))
            // .add_option(CreateCommandOption::new(
            //     CommandOptionType::SubCommand,
            //     "leave",
            //     "Leave a looking for group post",
            // ))
            // .add_option(CreateCommandOption::new(
            //     CommandOptionType::SubCommand,
            //     "joined",
            //     "View all the posts you have joined",
            // ))
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "timezone",
                    "Set your timezone",
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::String, "region", "Your region")
                        .required(true)
                        .add_string_choice("Africa", "Africa")
                        .add_string_choice("America", "America")
                        .add_string_choice("Antarctica", "Antarctica")
                        .add_string_choice("Arctic", "Arctic")
                        .add_string_choice("Asia", "Asia")
                        .add_string_choice("Atlantic", "Atlantic")
                        .add_string_choice("Australia", "Australia")
                        .add_string_choice("Brazil", "Brazil")
                        .add_string_choice("Canada", "Canada")
                        .add_string_choice("Chile", "Chile")
                        .add_string_choice("Europe", "Europe")
                        .add_string_choice("Indian", "Indian")
                        .add_string_choice("Mexico", "Mexico")
                        .add_string_choice("Pacific", "Pacific")
                        .add_string_choice("US", "US")
                        .add_string_choice("Other", "Other"),
                ),
            )
    }

    pub async fn autocomplete(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
        let command = &interaction.data.options()[0];

        if command.name != "timezone" {
            return Ok(());
        }

        let option = interaction.data.autocomplete().unwrap();

        let filtered = chrono_tz::TZ_VARIANTS
            .iter()
            .filter(|tz| {
                let name = tz.name();
                name.starts_with(option.value)
            })
            .take(25)
            .map(|tz| AutocompleteChoice::new(tz.name(), tz.name()))
            .collect::<Vec<_>>();

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
