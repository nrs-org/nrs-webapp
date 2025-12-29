use std::{convert::Infallible, fmt::Arguments, str::FromStr};

use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use nrs_webapp_frontend::views::document::DocumentProps;
use serde::Deserialize;

use crate::{auth::session::Session, toasts::ConstToast};

pub struct DocProps(pub DocumentProps);

#[derive(Default, Deserialize)]
struct GlobalQueryParams {
    toast: Option<String>,
}

#[macro_export]
macro_rules! toast_on_page_load {
    ($toast:expr) => {
        format_args!("toast={}", urlencoding::encode($toast.into()))
    };
}

impl<S> FromRequestParts<S> for DocProps
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    /// Extracts `DocProps` from HTTP request parts, using an optional `toast` query parameter and the request session.
    ///
    /// The extractor:
    /// - sets `logged_in` to `true` when a `Session` is present in `parts.extensions`, `false` otherwise;
    /// - parses an optional `toast` query parameter, converts it to a `ConstToast` via `FromStr`, and places a single converted toast in `toasts` when parsing succeeds (invalid or absent values produce an empty `toasts` vector);
    /// - leaves all other `DocumentProps` fields as their defaults.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use axum::http::request::Parts;
    /// use axum::extract::FromRequestParts;
    /// // let mut parts: Parts = /* built from a request */ todo!();
    /// // let state = /* app state */ todo!();
    /// // let props = DocProps::from_request_parts(&mut parts, &state).await.unwrap();
    /// ```
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        tracing::debug!("{:<12} -- DocProps", "EXTRACTOR");

        let Query(GlobalQueryParams { toast }): Query<GlobalQueryParams> =
            Query::from_request_parts(parts, state)
                .await
                .unwrap_or_default();

        let session = parts.extensions.get::<Session>();

        // TODO: implement this
        Ok(Self(DocumentProps {
            logged_in: session.is_some(),
            toasts: toast
                .and_then(|t| ConstToast::from_str(&t).ok())
                .map(|t| vec![t.into()])
                .unwrap_or_default(),
            ..Default::default()
        }))
    }
}
