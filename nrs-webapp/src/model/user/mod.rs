use sqlbindable::Fields;
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::model::{
    Error, ModelManager, Result,
    entity::{DbBmc, DbBmcWithPkey},
    store::primary_store::PrimaryStore,
};

pub struct UserBmc;

impl DbBmc for UserBmc {
    const TABLE_NAME: &'static str = "app_user";
}

impl DbBmcWithPkey for UserBmc {
    const PRIMARY_KEY: &'static str = "id";
    type PkeyType = String;
}

#[derive(Debug, Clone, FromRow, Fields)]
pub struct UserForCreate {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl UserBmc {
    pub async fn create_dev_user<'p, P: PrimaryStore>(
        mm: &'p mut P,
        create_req: UserForCreate,
    ) -> Result<String>
    where
        P::Executor<'p>: for<'a> sqlx::PgExecutor<'a>,
    {
        <Self as DbBmcWithPkey>::create_returning_pkey(mm, create_req).await
    }
}
