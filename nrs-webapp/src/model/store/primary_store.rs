use std::marker::PhantomData;

use crate::model::{Error, Result};
use always_send::FutureExt;
use sea_query::PostgresQueryBuilder;
use sea_query_sqlx::{SqlxBinder, SqlxValues};
use sqlx::{FromRow, PgExecutor as ExecutorTrait};

use crate::model::SqlxRow;

// aka main DB, ie the postgres DB that currently has everything
pub trait PrimaryStore: Send {
    type Executor<'a>: ExecutorTrait<'a>
    where
        Self: 'a;

    fn executor(&mut self) -> Self::Executor<'_>;

    fn query_builder(&self) -> PostgresQueryBuilder {
        PostgresQueryBuilder
    }

    fn query_with(&mut self, query: &impl SqlxBinder) -> PrimaryStoreQuery<Self::Executor<'_>> {
        let (sql, args) = query.build_sqlx(self.query_builder());
        PrimaryStoreQuery {
            executor: self.executor(),
            sql,
            args,
        }
    }

    fn query_as_with<T>(
        &mut self,
        query: &impl SqlxBinder,
    ) -> PrimaryStoreQueryAs<Self::Executor<'_>, T>
    where
        T: for<'r> sqlx::FromRow<'r, SqlxRow> + Send + Unpin,
    {
        let (sql, args) = query.build_sqlx(self.query_builder());
        PrimaryStoreQueryAs {
            executor: self.executor(),
            sql,
            args,
            _marker: PhantomData,
        }
    }
}

pub struct PrimaryStoreQuery<E> {
    pub executor: E,
    pub sql: String,
    pub args: SqlxValues,
}

impl<E> PrimaryStoreQuery<E> {
    pub async fn execute<'e>(self) -> Result<u64>
    where
        E: ExecutorTrait<'e>,
    {
        let result = sqlx::query_with(&self.sql, self.args)
            .execute(self.executor)
            .always_send()
            .await?;
        Ok(result.rows_affected())
    }
}

pub struct PrimaryStoreQueryAs<E, T> {
    executor: E,
    sql: String,
    args: SqlxValues,
    _marker: core::marker::PhantomData<T>,
}

/// SAFETY: T is only a marker
unsafe impl<E: Send, T> Send for PrimaryStoreQueryAs<E, T> {}

impl<T, E> PrimaryStoreQueryAs<E, T> {
    pub async fn fetch_optional<'e>(self) -> Result<Option<T>>
    where
        T: Send + Unpin + for<'r> FromRow<'r, SqlxRow>,
        E: ExecutorTrait<'e>,
    {
        let row: Option<T> = sqlx::query_as_with(&self.sql, self.args)
            .fetch_optional(self.executor)
            .await?;
        Ok(row)
    }

    pub async fn fetch_one<'e>(self) -> Result<T>
    where
        T: Send + Unpin + for<'r> FromRow<'r, SqlxRow>,
        E: ExecutorTrait<'e>,
    {
        let row: T = sqlx::query_as_with(&self.sql, self.args)
            .fetch_one(self.executor)
            .await?;
        Ok(row)
    }

    pub async fn fetch_all<'e>(self) -> Result<Vec<T>>
    where
        T: Send + Unpin + for<'r> FromRow<'r, SqlxRow>,
        E: ExecutorTrait<'e>,
    {
        let rows: Vec<T> = sqlx::query_as_with(&self.sql, self.args)
            .fetch_all(self.executor)
            .await?;
        Ok(rows)
    }
}
