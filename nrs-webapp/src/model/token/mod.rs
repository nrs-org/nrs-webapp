use sea_query::{Expr, ExprTrait, OnConflict, Query, Value};
use sqlbindable::{
    BindContext, FieldNames, Fields, HasFieldNames, HasFields, TryIntoExpr, TryIntoExprError,
};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{self, Result, entity::DbBmc, store::primary_store::PrimaryStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "USER_ONE_TIME_TOKEN_PURPOSE")]
pub enum TokenPurpose {
    #[sqlx(rename = "EMAIL_VERIFICATION")]
    EmailVerification,
    #[sqlx(rename = "PASSWORD_RESET")]
    PasswordReset,
}

impl TokenPurpose {
    /// Get the SQL enum string corresponding to this token purpose.
    ///
    /// Returns the uppercase enum identifier used in the database: `"EMAIL_VERIFICATION"` or `"PASSWORD_RESET"`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(TokenPurpose::EmailVerification.to_enum_string(), "EMAIL_VERIFICATION");
    /// assert_eq!(TokenPurpose::PasswordReset.to_enum_string(), "PASSWORD_RESET");
    /// ```
    pub fn to_enum_string(&self) -> &'static str {
        match self {
            TokenPurpose::EmailVerification => "EMAIL_VERIFICATION",
            TokenPurpose::PasswordReset => "PASSWORD_RESET",
        }
    }
}

impl From<TokenPurpose> for Expr {
    /// Converts a `TokenPurpose` into an SQL expression of type `USER_ONE_TIME_TOKEN_PURPOSE`.
    ///
    /// The produced expression is a string literal of the enum variant cast to the database enum type.
    ///
    /// # Examples
    ///
    /// ```
    /// let _expr = Expr::from(TokenPurpose::EmailVerification);
    /// ```
    fn from(purpose: TokenPurpose) -> Self {
        Value::String(Some(purpose.to_enum_string().into())).cast_as("USER_ONE_TIME_TOKEN_PURPOSE")
    }
}

impl TryIntoExpr for TokenPurpose {
    /// Convert this `TokenPurpose` into an SQL expression representing that enum value.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::model::token::TokenPurpose;
    ///
    /// let expr = TokenPurpose::EmailVerification.into_expr().unwrap();
    /// ```
    fn into_expr(self) -> core::result::Result<Expr, TryIntoExprError> {
        Ok(self.into())
    }
}

pub struct UserOneTimeTokenBmc;

impl DbBmc for UserOneTimeTokenBmc {
    const TABLE_NAME: &'static str = "user_one_time_token";
}

#[derive(Debug, Clone, FieldNames, Fields)]
pub struct UserOneTimeTokenCreateReq {
    pub user_id: Uuid,
    pub purpose: TokenPurpose,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub request_ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(FromRow)]
struct UserId {
    user_id: Uuid,
}

impl UserOneTimeTokenBmc {
    /// Inserts a new one-time token row or updates an existing unused token for the same user and purpose.
    ///
    /// If a row with the same `(user_id, purpose)` exists and `last_used_at` is NULL, the existing row is updated
    /// with the provided fields and `created_at` is set to the default timestamp. Otherwise a new row is inserted.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example_usage(ps: &mut impl crate::store::PrimaryStore, req: crate::model::token::UserOneTimeTokenCreateReq) -> anyhow::Result<()> {
    /// crate::model::token::UserOneTimeTokenBmc::create_token(ps, req).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_token(
        ps: &mut impl PrimaryStore,
        create_req: UserOneTimeTokenCreateReq,
    ) -> Result<()> {
        ps.query_with(
            Query::insert()
                .into_table(Self::TABLE_NAME)
                .bind(create_req.not_none_fields()?)
                .on_conflict(
                    // Intentional refresh-on-conflict: unused tokens are rotated and their
                    // expires_at is extended when re-created for the same (user_id, purpose).
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

    /// Validate and consume a one-time token and return the associated user ID.
    ///
    /// Attempts to mark the token identified by `token_hash` and `purpose` as used (sets `last_used_at`)
    /// only if the token exists, matches the purpose, has not expired, and has not been used yet.
    ///
    /// # Returns
    ///
    /// `String` containing the `user_id` associated with the consumed token.
    ///
    /// # Errors
    ///
    /// Returns `model::Error::InvalidOrExpiredToken` if no matching unused and unexpired token is found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use nrs_webapp::model::token::{UserOneTimeTokenBmc, TokenPurpose};
    /// # async fn example(ps: &mut impl nrs_webapp::store::PrimaryStore) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id = UserOneTimeTokenBmc::check_and_consume_token(ps, "some_hash", TokenPurpose::EmailVerification).await?;
    /// println!("consumed token for user: {}", user_id);
    /// # Ok(()) }
    /// ```
    pub async fn check_and_consume_token(
        ps: &mut impl PrimaryStore,
        token_hash: &str,
        purpose: TokenPurpose,
    ) -> Result<Uuid> {
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
