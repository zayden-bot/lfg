use zayden_core::ErrorResponse;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FireteamFull,
    PostNotFound,

    ParseInt(std::num::ParseIntError),
    Serenity(serenity::Error),
    ChronoTzParseError(chrono_tz::ParseError),
}

impl ErrorResponse for Error {
    fn to_response(&self) -> String {
        match self {
            Self::FireteamFull => String::from("Unable to join. Fireteam is full"),
            Self::PostNotFound => String::from(
                "Post not found, please message <@211486447369322506> if the issue persists",
            ),
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

impl From<chrono_tz::ParseError> for Error {
    fn from(error: chrono_tz::ParseError) -> Self {
        Self::ChronoTzParseError(error)
    }
}
