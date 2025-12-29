use std::convert::Infallible;

use axum::extract::OriginalUri;

use crate::{Error, Result};

/// Handle requests that do not match any route by producing a `PageNotFound` error for the original request URI.
///
/// Returns an `Error::PageNotFound` containing the captured `uri`.
///
/// # Examples
///
/// ```no_run
/// use axum::http::Uri;
/// use axum::extract::OriginalUri;
/// use futures::executor::block_on;
/// // Replace the path below with the path you want to test.
/// let uri = Uri::from_static("/nonexistent");
/// let res = block_on(nrs_webapp::routes::fallback::fallback_handler(OriginalUri(uri)));
/// assert!(res.is_err());
/// ```
pub async fn fallback_handler(OriginalUri(uri): OriginalUri) -> Result<Infallible> {
    Err(Error::PageNotFound { uri })
}

/// Handle requests with unsupported HTTP methods by returning a page-not-found error for the original URI.
///
/// This fallback always produces an `Err(Error::PageNotFound)` that contains the captured request `URI`.
///
/// # Examples
///
/// ```
/// use axum::extract::OriginalUri;
/// use http::Uri;
///
/// # #[tokio::main]
/// # async fn main() {
/// let uri = Uri::from_static("/unsupported");
/// let res = method_not_allowed_fallback_handler(OriginalUri(uri)).await;
/// assert!(matches!(res, Err(crate::Error::PageNotFound { uri: _ })));
/// # }
/// ```
pub async fn method_not_allowed_fallback_handler(
    OriginalUri(uri): OriginalUri,
) -> Result<Infallible> {
    Err(Error::PageNotFound { uri })
}