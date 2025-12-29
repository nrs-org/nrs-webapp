use sea_query::{Expr, ExprTrait};
use sqlbindable::{Fields, HasFields};
use sqlx::FromRow;

use crate::model::{
    Error, Result, SqlxRow,
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
    /// Creates a `UserMarkEmailVerified` where `email_verified_at` is set to the current timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// let mark = UserMarkEmailVerified::default();
    /// // `mark.email_verified_at` is an expression representing the current timestamp.
    /// ```
    fn default() -> Self {
        Self {
            email_verified_at: Expr::current_timestamp(),
        }
    }
}

impl UserBmc {
    /// Creates a new user and returns the newly created user's primary key.
    ///
    /// On database unique-constraint violations for username or email, this maps the error to
    /// `Error::EmailOrUsernameAlreadyExists`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::{UserBmc, UserForCreate, Error};
    /// # async fn example(mm: &mut impl crate::PrimaryStore) -> Result<(), Error> {
    /// let req = UserForCreate {
    ///     username: "alice".into(),
    ///     email: "alice@example.com".into(),
    ///     password_hash: "hash".into(),
    /// };
    /// let user_id = UserBmc::create_user(mm, req).await?;
    /// assert!(!user_id.is_empty());
    /// # Ok(()) }
    /// ```
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

    /// Fetches an optional record matching the given email.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use crate::bmc::UserBmc;
    /// # async fn run(mm: &mut impl crate::store::PrimaryStore) -> crate::Result<()> {
    /// let user: Option<crate::models::User> = UserBmc::get_by_email(mm, "alice@example.com").await?;
    /// if let Some(u) = user {
    ///     assert_eq!(u.email, "alice@example.com");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// @returns `Some(E)` if a record with the given email exists, `None` otherwise.
    pub async fn get_by_email<E>(mm: &mut impl PrimaryStore, email: &str) -> Result<Option<E>>
    where
        E: for<'r> FromRow<'r, SqlxRow> + Unpin + Send + HasFields,
    {
        <Self as DbBmc>::get_optional_by_expr(mm, Expr::col("email").eq(email)).await
    }

    /// Fetches an optional record that matches the given username.
    ///
    /// # Returns
    ///
    /// `Some(E)` containing the matched record when found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(mm: &mut impl crate::PrimaryStore) -> anyhow::Result<()> {
    /// let maybe_user = crate::user::UserBmc::get_by_username::<crate::user::UserForCreate>(mm, "alice").await?;
    /// if let Some(user) = maybe_user {
    ///     // found user
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_by_username<E>(mm: &mut impl PrimaryStore, username: &str) -> Result<Option<E>>
    where
        E: for<'r> FromRow<'r, SqlxRow> + Unpin + Send + HasFields,
    {
        <Self as DbBmc>::get_optional_by_expr(mm, Expr::col("username").eq(username)).await
    }

    /// Marks the user's email as verified by setting the verification timestamp to now.
    ///
    /// Applies the default `UserMarkEmailVerified` update to the user record identified by `user_id`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeded, `Err(_)` if the store operation failed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut mm = /* obtain a PrimaryStore implementor */ unimplemented!();
    /// mark_email_verified(&mut mm, "user-123").await?;
    /// # Ok(()) }
    /// ```
    pub async fn mark_email_verified(mm: &mut impl PrimaryStore, user_id: &str) -> Result<()> {
        <Self as DbBmcWithPkey>::update(mm, UserMarkEmailVerified::default(), user_id.into()).await
    }

    /// Update a user's stored password hash.
    ///
    /// Updates the `password_hash` field for the user identified by `user_id`.
    ///
    /// # Parameters
    ///
    /// - `user_id`: the primary key of the user to update (string).
    /// - `password_hash`: the new password hash to store (already hashed).
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeds, `Err` with the underlying database error otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(mm: &mut impl PrimaryStore) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id = "user-123".to_string();
    /// let password_hash = "new_hashed_password".to_string();
    /// UserBmc::reset_password(mm, user_id, password_hash).await?;
    /// # Ok(()) }
    /// ```
    pub async fn reset_password(
        mm: &mut impl PrimaryStore,
        user_id: String,
        password_hash: String,
    ) -> Result<()> {
        <Self as DbBmcWithPkey>::update(mm, UserResetPassword { password_hash }, user_id).await
    }
}
