use std::convert::Infallible;

use axum::extract::OriginalUri;

use crate::{Error, Result};

pub async fn fallback_handler(OriginalUri(uri): OriginalUri) -> Result<Infallible> {
    Err(Error::PageNotFound { uri })
}

pub async fn method_not_allowed_fallback_handler(
    OriginalUri(uri): OriginalUri,
) -> Result<Infallible> {
    Err(Error::PageNotFound { uri })
}
