use std::marker::PhantomData;

use crate::model::store::{Db, new_db_pool, primary_store::PrimaryStore};
use sea_query::{PostgresQueryBuilder, QueryStatementWriter};
use sea_query_sqlx::{SqlxBinder, SqlxValues};

pub mod entity;
pub mod entry;
mod error;
mod store;
pub mod user;

pub use error::{Error, Result};
use sqlx::{Connection, Database, FromRow, Transaction};

type SqlxDatabase = sqlx::Postgres;
type SqlxRow = sqlx::postgres::PgRow;

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(Self { db })
    }

    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }

    pub async fn tx(&self) -> Result<Transaction<'_, SqlxDatabase>> {
        let tx = self.db.begin().await?;
        Ok(tx)
    }
}

impl PrimaryStore for ModelManager {
    type Executor<'a> = &'a Db;

    fn executor(&mut self) -> Self::Executor<'_> {
        self.db()
    }
}

impl<'t> PrimaryStore for Transaction<'t, SqlxDatabase> {
    fn executor(&mut self) -> Self::Executor<'_> {
        &mut *self
    }

    type Executor<'a>
        = &'a mut <SqlxDatabase as Database>::Connection
    where
        Self: 'a;
}
