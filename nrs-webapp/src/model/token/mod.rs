use always_send::FutureExt as _;
use sea_query::{Expr, ExprTrait, OnConflict, Query, ReturningClause, Value};
use sqlbindable::{BindContext, Fields, HasFields, TryIntoExpr, TryIntoExprError};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

use crate::model::{
    self, Result,
    entity::{DbBmc, DbBmcWithPkey},
    store::primary_store::PrimaryStore,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "USER_ONE_TIME_TOKEN_PURPOSE")]
pub enum TokenPurpose {
    #[sqlx(rename = "EMAIL_VERIFICATION")]
    EmailVerification,
    #[sqlx(rename = "PASSWORD_RESET")]
    PasswordReset,
}

impl TokenPurpose {
    pub fn to_enum_string(&self) -> &'static str {
        match self {
            TokenPurpose::EmailVerification => "EMAIL_VERIFICATION",
            TokenPurpose::PasswordReset => "PASSWORD_RESET",
        }
    }
}

impl From<TokenPurpose> for Expr {
    fn from(purpose: TokenPurpose) -> Self {
        Value::String(Some(purpose.to_enum_string().into())).cast_as("USER_ONE_TIME_TOKEN_PURPOSE")
    }
}

impl TryIntoExpr for TokenPurpose {
    fn into_expr(self) -> core::result::Result<Expr, TryIntoExprError> {
        Ok(self.into())
    }
}

pub struct UserOneTimeTokenBmc;

impl DbBmc for UserOneTimeTokenBmc {
    const TABLE_NAME: &'static str = "user_one_time_token";
}

#[derive(Debug, Clone, Fields)]
pub struct UserOneTimeTokenCreateReq {
    pub user_id: String,
    pub purpose: TokenPurpose,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub request_ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(FromRow)]
struct UserId {
    user_id: String,
}

impl UserOneTimeTokenBmc {
    pub async fn create_token(
        ps: &mut impl PrimaryStore,
        create_req: UserOneTimeTokenCreateReq,
    ) -> Result<()> {
        ps.query_with(
            Query::insert()
                .into_table(Self::TABLE_NAME)
                .bind(create_req.not_none_fields()?)
                .on_conflict(
                    OnConflict::columns(["user_id", "purpose"])
                        .target_and_where(Expr::column("last_used_at").is_null())
                        .update_columns(UserOneTimeTokenCreateReq::field_names().iter().copied())
                        .value("created_at", Expr::keyword_default())
                        .to_owned(),
                ),
        )
        .execute()
        .await?;
        Ok(())
    }

    pub async fn check_and_consume_token(
        ps: &mut impl PrimaryStore,
        token_hash: &str,
        purpose: TokenPurpose,
    ) -> Result<String> {
        let UserId { user_id } = ps
            .query_as_with::<UserId>(
                Query::update()
                    .table(Self::TABLE_NAME)
                    .value("last_used_at", Expr::current_timestamp())
                    .and_where(Expr::col("token_hash").eq(token_hash))
                    .and_where(Expr::col("purpose").eq(purpose))
                    .and_where(Expr::col("expires_at").gt(Expr::current_timestamp()))
                    .and_where(Expr::col("last_used_at").is_null())
                    .returning_col("user_id"),
            )
            .fetch_optional()
            .await?
            .ok_or_else(|| model::Error::InvalidOrExpiredToken)?;
        Ok(user_id)
    }
}
