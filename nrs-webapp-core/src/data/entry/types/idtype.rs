use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
#[cfg_attr(feature = "sql", sqlx(type_name = "ENTRYTYPE"))]
#[derive(Default)]
pub enum EntryType {
    // DAH_entry_id_impl
    Anime,
    Manga,
    LightNovel,
    VisualNovel,
    MusicArtist,
    MusicAlbum,
    MusicTrack,
    MusicAlbumTrack,
    Franchise,
    Game,
    #[default]
    Other,
    // Non-standard
    GenericPerson,
    GenericOrganization,
}


#[cfg(feature = "sql")]
pub mod sql {
    use sea_query::{Expr, ExprTrait, Nullable, Value};
    use sqlbindable::{TryIntoExpr, TryIntoExprError};

    use super::EntryType;

    impl TryIntoExpr for EntryType {
        fn into_expr(self) -> Result<Expr, TryIntoExprError> {
            Ok(Value::String(Some(self.to_enum_string())).cast_as("ENTRYTYPE"))
        }
    }

    impl Nullable for EntryType {
        fn null() -> Value {
            Value::String(None)
        }
    }
}

impl Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Error)]
pub enum EntryTypeParseError {
    #[error("Invalid entry type")]
    InvalidEntryType,
}

impl FromStr for EntryType {
    type Err = EntryTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_enum_string(s).ok_or(EntryTypeParseError::InvalidEntryType)
    }
}

impl EntryType {
    pub fn to_display_string(&self) -> &'static str {
        match self {
            EntryType::Anime => "Anime",
            EntryType::Manga => "Manga",
            EntryType::LightNovel => "Light Novel",
            EntryType::VisualNovel => "Visual Novel",
            EntryType::MusicArtist => "Music Artist",
            EntryType::MusicAlbum => "Music Album",
            EntryType::MusicTrack => "Music Track",
            EntryType::MusicAlbumTrack => "Music Album Track",
            EntryType::Franchise => "Franchise",
            EntryType::Game => "Game",
            EntryType::Other => "Other",
            EntryType::GenericPerson => "Generic Person",
            EntryType::GenericOrganization => "Generic Organization",
        }
    }

    pub fn all() -> impl IntoIterator<Item = EntryType> {
        [
            EntryType::Anime,
            EntryType::Manga,
            EntryType::LightNovel,
            EntryType::VisualNovel,
            EntryType::MusicArtist,
            EntryType::MusicAlbum,
            EntryType::MusicTrack,
            EntryType::MusicAlbumTrack,
            EntryType::Franchise,
            EntryType::Game,
            EntryType::Other,
            EntryType::GenericPerson,
            EntryType::GenericOrganization,
        ]
    }

    pub fn to_enum_string(&self) -> String {
        format!("{:?}", self)
    }

    pub fn from_enum_string(s: &str) -> Option<EntryType> {
        match s {
            "Anime" => Some(EntryType::Anime),
            "Manga" => Some(EntryType::Manga),
            "LightNovel" => Some(EntryType::LightNovel),
            "VisualNovel" => Some(EntryType::VisualNovel),
            "MusicArtist" => Some(EntryType::MusicArtist),
            "MusicAlbum" => Some(EntryType::MusicAlbum),
            "MusicTrack" => Some(EntryType::MusicTrack),
            "MusicAlbumTrack" => Some(EntryType::MusicAlbumTrack),
            "Franchise" => Some(EntryType::Franchise),
            "Game" => Some(EntryType::Game),
            "Other" => Some(EntryType::Other),
            "GenericPerson" => Some(EntryType::GenericPerson),
            "GenericOrganization" => Some(EntryType::GenericOrganization),
            _ => None,
        }
    }
}
