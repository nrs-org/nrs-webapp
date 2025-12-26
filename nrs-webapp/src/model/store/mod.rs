use sqlx::postgres::PgPoolOptions;

use crate::config::AppConfig;

mod error;
pub mod primary_store;
pub use error::{Error, Result};

pub type Db = sqlx::PgPool;

pub async fn new_db_pool() -> Result<Db> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&AppConfig::get().SERVICE_DB_URL)
        .await
        .map_err(|e| Error::FailedCreatingDbPool(e.to_string()))?;
    Ok(pool)
}
