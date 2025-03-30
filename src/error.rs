use serenity::all::{Mentionable, UserId};
use zayden_core::ErrorResponse;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingGuildId,
    MissingSetup,
    FireteamFull,
    PermissionDenied(String),
    InvalidDateTime(String),
    TagRequired,
    AlreadyJoined,
    InvalidChannel,
}

impl Error {
    pub fn permission_denied(owner: UserId) -> Self {
        let response = format!(
            "Permission denied. Only the owner ({}) can use this action.",
            owner.mention()
        );
        
        Self::PermissionDenied(response)
    }

    pub fn invalid_date_time(format: &str) -> Self {
        let response = format!("Invalid date time. Expected format: {}", format);

        Self::InvalidDateTime(response)
    }
}

impl ErrorResponse for Error {
    fn to_response(&self) -> &str {
        match self {
            Self::MissingGuildId => zayden_core::Error::MissingGuildId.to_response(),
            Self::MissingSetup => "Missing setup. If you are the owner, please run `/lfg setup` to set up the bot.",
            Self::FireteamFull => "Unable to join. Fireteam is full.",
            Self::PermissionDenied(msg) => msg, 
            Self::InvalidDateTime(msg) => msg,
            Self::TagRequired => "Unable to parse Activity and apply necessary tags. Please fix the Activity field and use the edit button to update after creating the post.",
            Self::AlreadyJoined => "You have already joined this LFG.",
            Self::InvalidChannel => "Invalid LFG channel."
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
