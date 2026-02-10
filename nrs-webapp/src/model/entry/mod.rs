pub mod alias;

use sea_query::{Expr, ExprTrait, JoinType, Query, SelectExpr, SelectStatement};
use sqlbindable::{FieldNames, Fields, HasFieldNames};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use crate::model::{
    Result,
    entity::{ApplyExt, DbBmc, DbBmcWithPkey, ListPayload},
    store::primary_store::PrimaryStore,
    user::UserBmc,
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

#[derive(Debug, Clone, FromRow, FieldNames, Fields)]
pub struct EntryForCreate {
    pub id: String,
    pub title: String,
    pub entry_type: EntryType,
    pub added_by: Uuid,
}

#[derive(Debug, Clone, FromRow, FieldNames)]
pub struct EntryAddedBy {
    #[sqlx(rename = "added_by.id")]
    pub id: Uuid,
    #[sqlx(rename = "added_by.username")]
    pub username: String,
}

#[derive(Debug, Clone, FromRow, FieldNames)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub entry_type: EntryType,
    #[sqlx(flatten)]
    pub added_by: EntryAddedBy,
    pub entry_info: Json<serde_json::Value>,
}

impl EntryBmc {
    pub async fn create_entry(
        mm: &mut impl PrimaryStore,
        create_req: EntryForCreate,
    ) -> Result<()> {
        <Self as DbBmc>::create(mm, create_req).await?;
        Ok(())
    }

    fn select_entry() -> SelectStatement {
        let mut query = Query::select();
        query
            .from(Self::TABLE_NAME)
            .exprs(
                Entry::field_names()
                    .iter()
                    .copied()
                    .filter(|col| *col != "added_by")
                    .map(|col| SelectExpr {
                        expr: Expr::col((Self::TABLE_NAME, col)),
                        alias: Some(col.into()),
                        window: None,
                    }),
            )
            .exprs(
                EntryAddedBy::field_names()
                    .iter()
                    .copied()
                    .map(|col| SelectExpr {
                        expr: Expr::col((UserBmc::TABLE_NAME, col)),
                        alias: Some(format!("added_by.{col}").into()),
                        window: None,
                    }),
            )
            .join(
                JoinType::Join,
                UserBmc::TABLE_NAME,
                Expr::col((Self::TABLE_NAME, "added_by")).equals((UserBmc::TABLE_NAME, "id")),
            );
        query
    }

    pub async fn list_entries(
        ps: &mut impl PrimaryStore,
        payload: ListPayload,
    ) -> Result<Vec<Entry>> {
        let entities = ps
            .query_as_with(Self::select_entry().apply_alias(payload, Self::TABLE_NAME))
            .fetch_all()
            .await?;
        Ok(entities)
    }

    pub async fn get_details(ps: &mut impl PrimaryStore, id: String) -> Result<Entry> {
        let maybe_entity = ps
            .query_as_with::<Entry>(Self::select_entry().and_where(Self::cond_pkey(id)))
            .fetch_one()
            .await?;
        Ok(maybe_entity)
    }
}
