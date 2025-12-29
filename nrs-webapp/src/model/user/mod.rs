use always_send::FutureExt;
use sea_query::{Expr, ExprTrait};
use sqlbindable::{Fields, HasFields};
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::model::{
    Error, ModelManager, Result, SqlxRow,
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

#[derive(Fields)]
struct UserMarkEmailVerified {
    pub email_verified_at: Expr,
}

#[derive(Fields)]
struct UserResetPassword {
    pub password_hash: String,
}

impl Default for UserMarkEmailVerified {
    fn default() -> Self {
        Self {
            email_verified_at: Expr::current_timestamp(),
        }
    }
}

impl UserBmc {
    pub async fn create_user(
        mm: &mut impl PrimaryStore,
        create_req: UserForCreate,
    ) -> Result<String> {
        <Self as DbBmcWithPkey>::create_returning_pkey(mm, create_req)
            .await
            .map_err(|e| match e {
                Error::Sqlx(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                    Error::EmailOrUsernameAlreadyExists
                }
                _ => e,
            })
    }

    pub async fn get_by_email<E>(mm: &mut impl PrimaryStore, email: &str) -> Result<Option<E>>
    where
        E: for<'r> FromRow<'r, SqlxRow> + Unpin + Send + HasFields,
    {
        <Self as DbBmc>::get_optional_by_expr(mm, Expr::col("email").eq(email)).await
    }

    pub async fn get_by_username<E>(mm: &mut impl PrimaryStore, username: &str) -> Result<Option<E>>
    where
        E: for<'r> FromRow<'r, SqlxRow> + Unpin + Send + HasFields,
    {
        <Self as DbBmc>::get_optional_by_expr(mm, Expr::col("username").eq(username)).await
    }

    pub async fn mark_email_verified(mm: &mut impl PrimaryStore, user_id: &str) -> Result<()> {
        <Self as DbBmcWithPkey>::update(mm, UserMarkEmailVerified::default(), user_id.into()).await
    }

    pub async fn reset_password(
        mm: &mut impl PrimaryStore,
        user_id: String,
        password_hash: String,
    ) -> Result<()> {
        <Self as DbBmcWithPkey>::update(mm, UserResetPassword { password_hash }, user_id).await
    }
}
