use zayden_core::ErrorResponse;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FireteamFull,

    Serenity(serenity::Error),
}

impl ErrorResponse for Error {
    fn to_response(&self) -> String {
        match self {
            Self::FireteamFull => String::from("Unable to join. Fireteam is full"),
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

impl From<serenity::Error> for Error {
    fn from(error: serenity::Error) -> Self {
        Self::Serenity(error)
    }
}
