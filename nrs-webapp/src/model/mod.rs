use crate::model::store::{Db, new_db_pool, primary_store::PrimaryStore};

pub mod entity;
pub mod entry;
mod error;
mod store;
pub mod token;
pub mod user;

pub use error::{Error, Result};
use sqlx::{Database, Transaction};

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

    /// Get a reference to the manager's database pool.
    ///
    /// Returns a reference to the internal `Db`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let manager: crate::model::ModelManager = todo!();
    /// let db_ref = manager.db();
    /// let _ = db_ref; // use the Db reference
    /// ```
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }

    /// Begins a new database transaction from the manager's connection pool.
    ///
    /// # Returns
    ///
    /// A `Transaction<'static, SqlxDatabase>` wrapped in `Result` on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(mgr: &crate::model::ModelManager) -> Result<(), crate::model::Error> {
    /// let mut tx = mgr.tx().await?;
    /// // use `tx`...
    /// Ok(())
    /// # }
    /// ```
    pub async fn tx(&self) -> Result<Transaction<'static, SqlxDatabase>> {
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
