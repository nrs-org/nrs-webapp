use std::sync::Arc;

use axum::{
    extract::OriginalUri,
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use nrs_webapp_frontend::views::{self, error::ClientError};
use thiserror::Error;

use crate::model;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Model error: {0}")]
    Model(#[from] model::Error),

    #[error("Page not found: {uri}")]
    PageNotFound { uri: Uri },
}

pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    pub fn get_client_error_parts(&self) -> (StatusCode, &'static str) {
        match self {
            Error::PageNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "The page you are looking for does not exist.",
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error"),
        }
    }

    pub fn get_client_error(&self, title: Option<String>) -> (StatusCode, ClientError) {
        let title = title.unwrap_or_else(|| "Error".into());
        let (status_code, description) = self.get_client_error_parts();
        (
            status_code,
            ClientError {
                title,
                description: description.into(),
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
