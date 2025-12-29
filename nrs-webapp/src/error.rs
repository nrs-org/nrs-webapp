use std::{borrow::Cow, sync::Arc};

use axum::{
    extract::OriginalUri,
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use nrs_webapp_frontend::views::{self, error::ClientError};
use thiserror::Error;

use crate::{auth, crypt, extract::with_rejection::RejectionError, mail, model};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Model error: {0}")]
    Model(#[from] model::Error),

    #[error("Crypt error: {0}")]
    Crypt(#[from] crypt::Error),

    #[error("Auth error: {0}")]
    Auth(#[from] auth::Error),

    #[error("Mail error: {0}")]
    Mailer(#[from] mail::Error),

    #[error(transparent)]
    Rejection(#[from] RejectionError),

    #[error("Rate limit exceeded: {service}")]
    RateLimitExceeded { service: &'static str },

    #[error("Page not found: {uri}")]
    PageNotFound { uri: Uri },
}

impl From<sqlx::Error> for Error {
    /// Convert a `sqlx::Error` into this crate's `Error` by wrapping it in `Error::Model`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use nrs_webapp::error::Error;
    /// let sql_err = sqlx::Error::RowNotFound;
    /// let err = Error::from(sql_err);
    /// assert!(matches!(err, Error::Model(_)));
    /// ```
    fn from(value: sqlx::Error) -> Self {
        Self::Model(model::Error::Sqlx(value))
    }
}

pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    /// Map an `Error` to an HTTP status code and a short client-facing message.
    ///
    /// Returns a tuple with the HTTP `StatusCode` to send and a `Cow<'static, str>` containing a
    /// concise, user-visible description of the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use axum::http::StatusCode;
    /// use axum::http::Uri;
    /// use crate::error::Error;
    ///
    /// let err = Error::PageNotFound { uri: Uri::from_static("/missing") };
    /// let (status, desc) = err.get_client_error_parts();
    /// assert_eq!(status, StatusCode::NOT_FOUND);
    /// assert_eq!(desc, "The page you are looking for does not exist.");
    /// ```
    pub fn get_client_error_parts(&self) -> (StatusCode, Cow<'static, str>) {
        tracing::debug!("{:<12} -- Get client error parts for {self:?}", "ERR_PARTS");
        match self {
            Error::PageNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "The page you are looking for does not exist.".into(),
            ),
            Error::Auth(err) => match err {
                auth::Error::Login(err) => match err {
                    auth::error::LoginError::InvalidCredentials => (
                        StatusCode::BAD_REQUEST,
                        "Invalid credentials provided.".into(),
                    ),
                },
            },
            Error::Model(model::Error::EmailOrUsernameAlreadyExists) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "A user with the given email or username already exists.".into(),
            ),
            Error::Rejection(RejectionError::Validation(err)) => {
                (StatusCode::UNPROCESSABLE_ENTITY, err.to_string().into())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error.".into()),
        }
    }

    pub fn get_client_error(
        &self,
        title: Option<String>,
        req_uuid: String,
    ) -> (StatusCode, ClientError) {
        let title = title.unwrap_or_else(|| "Error".into());
        let (status_code, description) = self.get_client_error_parts();
        (
            status_code,
            ClientError {
                title,
                description: description.into(),
                req_uuid,
            },
        )
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("{:<12} -- Error {self:?}", "INTO_RES");

        let (status_code, _) = self.get_client_error_parts();

        let mut response = status_code.into_response();
        response.extensions_mut().insert(Arc::new(self));

        response
    }
}
