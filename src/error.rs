use serenity::all::{Mentionable, UserId};
use zayden_core::ErrorResponse;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GuildRequired,
    MissingSetup,
    FireteamFull,
    PostNotFound,
    PermissionDenied { owner: UserId },
    InvalidDateTime { format: String },

    ParseInt(std::num::ParseIntError),
    Serenity(serenity::Error),
    ChronoParseError(chrono::ParseError),
    ChronoTzParseError(chrono_tz::ParseError),
    Sqlx(sqlx::Error),
}

impl ErrorResponse for Error {
    fn to_response(&self) -> String {
        match self {
            Self::GuildRequired => String::from("Guild required to use this command."),
            Self::MissingSetup => String::from(
                "Missing setup. If you are the owner, please run `/lfg setup` to set up the bot.",
            ),
            Self::FireteamFull => String::from("Unable to join. Fireteam is full."),
            Self::PostNotFound => String::from(
                "Post not found, please message <@211486447369322506> if the issue persists.",
            ),
            Self::PermissionDenied { owner } => format!(
                "Permission denied. Only the owner ({}) can use this action.",
                owner.mention()
            ),
            Self::InvalidDateTime { format } => {
                format!("Invalid date time. Expected format: {}", format)
            }
            _ => String::new(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

impl From<serenity::Error> for Error {
    fn from(error: serenity::Error) -> Self {
        Self::Serenity(error)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(error: chrono::ParseError) -> Self {
        Self::ChronoParseError(error)
    }
}

impl From<chrono_tz::ParseError> for Error {
    fn from(error: chrono_tz::ParseError) -> Self {
        Self::ChronoTzParseError(error)
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error)
    }
}
