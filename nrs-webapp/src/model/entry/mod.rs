use sqlbindable::Fields;
use sqlx::{FromRow, PgExecutor};

use crate::model::{
    ModelManager, Result,
    entity::{DbBmc, DbBmcWithPkey},
    store::primary_store::PrimaryStore,
};
use nrs_webapp_core::data::entry::types::idtype::EntryType;

pub struct EntryBmc;

impl DbBmc for EntryBmc {
    const TABLE_NAME: &'static str = "entry";
}

impl DbBmcWithPkey for EntryBmc {
    const PRIMARY_KEY: &'static str = "id";
    type PkeyType = String;
}

#[derive(Debug, Clone, FromRow, Fields)]
pub struct EntryForCreate {
    pub id: String,
    pub title: String,
    pub entry_type: EntryType,
    pub added_by: String,
    pub overall_score: f64,
}

impl EntryBmc {
    pub async fn create_entry(
        mm: &mut impl PrimaryStore,
        create_req: EntryForCreate,
    ) -> Result<()> {
        <Self as DbBmc>::create(mm, create_req).await?;
        Ok(())
    }
}
