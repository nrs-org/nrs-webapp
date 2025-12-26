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
}

pub type Result<T> = core::result::Result<T, Error>;
