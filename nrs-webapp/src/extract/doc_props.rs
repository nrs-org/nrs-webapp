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
