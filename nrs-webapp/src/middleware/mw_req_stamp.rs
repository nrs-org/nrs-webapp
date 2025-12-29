use crate::Result;
use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ReqStamp {
    pub uuid: Uuid,
    pub time_in: OffsetDateTime,
}

pub async fn mw_req_stamp(mut req: Request, next: Next) -> Result<Response> {
    tracing::debug!("{:<12} -- mw_req_stamp", "MIDDLEWARE");

    let stamp = ReqStamp {
        uuid: Uuid::new_v4(),
        time_in: OffsetDateTime::now_utc(),
    };

    req.extensions_mut().insert(stamp);

    Ok(next.run(req).await)
}

impl<S: Send + Sync> FromRequestParts<S> for ReqStamp {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> core::result::Result<Self, Self::Rejection> {
        parts.extensions.get::<ReqStamp>().cloned().ok_or(())
    }
}
