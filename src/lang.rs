//
use std::fmt;

pub enum Language {
    EN,
    ENUS,
    ES,
    DE,
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        match self {
            Self::EN => "EN",
            Self::ENUS => "EN-US",
            Self::ES => "ES",
            Self::DE => "DE",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}