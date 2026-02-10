use sqlbindable::{FieldNames, Fields};
use sqlx::FromRow;

use crate::model::{
    Error, Result,
    entity::{DbBmc, DbBmcWithPkey},
    store::primary_store::PrimaryStore,
};

pub struct EntryAliasBmc;

impl DbBmc for EntryAliasBmc {
    const TABLE_NAME: &'static str = "entry_alias";
}

impl DbBmcWithPkey for EntryAliasBmc {
    const PRIMARY_KEY: &'static str = "old_id";
    type PkeyType = String;
}

#[derive(Debug, Clone, FromRow, FieldNames, Fields)]
pub struct EntryAliasForCreate {
    pub old_id: String,
    pub new_id: String,
}

#[derive(Debug, Clone, FromRow, FieldNames)]
struct EntryAliasNewId {
    pub new_id: String,
}

impl EntryAliasBmc {
    pub async fn create_entry_alias(
        mm: &mut impl PrimaryStore,
        create_req: EntryAliasForCreate,
    ) -> Result<()> {
        <Self as DbBmc>::create(mm, create_req).await?;
        Ok(())
    }

    pub async fn get_new_id(mm: &mut impl PrimaryStore, old_id: String) -> Result<Option<String>> {
        let entry: Result<EntryAliasNewId> = <Self as DbBmcWithPkey>::get(mm, old_id).await;
        match entry {
            Ok(EntryAliasNewId { new_id }) => Ok(Some(new_id)),
            Err(Error::EntityNotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn resolve_id(mm: &mut impl PrimaryStore, old_id: String) -> Result<String> {
        if let Some(new_id) = Self::get_new_id(mm, old_id.clone()).await? {
            Ok(new_id)
        } else {
            Ok(old_id)
        }
    }
}
