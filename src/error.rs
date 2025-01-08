use serenity::all::{Mentionable, UserId};
use zayden_core::ErrorResponse;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GuildRequired,
    MissingSetup,
    FireteamFull,
    PermissionDenied { owner: UserId },
    InvalidDateTime { format: String },
}

impl ErrorResponse for Error {
    fn to_response(&self) -> String {
        match self {
            Self::GuildRequired => String::from("Guild required to use this command."),
            Self::MissingSetup => String::from(
                "Missing setup. If you are the owner, please run `/lfg setup` to set up the bot.",
            ),
            Self::FireteamFull => String::from("Unable to join. Fireteam is full."),
            Self::PermissionDenied { owner } => format!(
                "Permission denied. Only the owner ({}) can use this action.",
                owner.mention()
            ),
            Self::InvalidDateTime { format } => {
                format!("Invalid date time. Expected format: {}", format)
            }
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
