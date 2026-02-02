use thiserror::Error;

use crate::model::{entity::id::EntityId, store};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Store error: {0}")]
    Store(#[from] store::Error),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("sqlbindable TryIntoExpr: {0}")]
    TryIntoExpr(#[from] sqlbindable::TryIntoExprError),

    #[error("Entity not found: {name} with ID {id}")]
    EntityNotFound { name: &'static str, id: EntityId },

    // UserBmc
    #[error("User with given email or username already exists")]
    EmailOrUsernameAlreadyExists,

    // Token
    #[error("Token is invalid or has expired")]
    InvalidOrExpiredToken,

    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
