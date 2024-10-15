use lazy_static::lazy_static;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    EditInteractionResponse, ResolvedValue,
};
use std::collections::HashMap;
use zayden_core::parse_options;

use crate::modal::create_modal;
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
        m.insert("vanguard", vec!["Strike", "Nightfall", "Grandmaster"]);
        m.insert(
            "crucible",
            vec!["Crucible", "Iron Banner", "Trials of Osiris"],
        );
        m
    };
}

pub struct LfgCommand;

impl LfgCommand {
    pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
        let command = &interaction.data.options()[0];

        let options = match &command.value {
            ResolvedValue::SubCommand(options) => options,
            _ => unreachable!("Subcommand is required"),
        };
        let options = parse_options(options);

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

            let modal = create_modal(activity, &interaction.locale);

            interaction
                .create_response(ctx, CreateInteractionResponse::Modal(modal))
                .await?;
        }
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
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "edit",
                    "Edit an existing looking for group post",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Channel,
                        "channel",
                        "The LFG post to edit",
                    )
                    .required(true),
                ),
            )
    }
}
// /event join
// /event joined
// /event leave
