use std::convert::Infallible;

use axum::extract::FromRequestParts;
use nrs_webapp_frontend::views::document::DocumentProps;

pub struct DocProps(pub DocumentProps);

impl<S> FromRequestParts<S> for DocProps
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // TODO: implement this
        Ok(Self(DocumentProps {
            logged_in: false,
            ..Default::default()
        }))
    }
}
