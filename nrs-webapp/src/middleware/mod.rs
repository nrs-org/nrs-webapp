use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use axum_htmx::HxRequest;
use nrs_webapp_frontend::views::{self, document::DocumentProps};

use crate::{Error, extract::doc_props::DocProps};

pub struct RequestTitle(pub String);

pub async fn mw_res_mapper(
    hx_request: HxRequest,
    DocProps(doc_props): DocProps,
    resp: Response,
) -> Response {
    tracing::debug!("{:<12} -- mw_res_mapper", "MW_RES_MAP");

    let error = resp.extensions().get::<Arc<Error>>().map(Arc::as_ref);
    let title = resp
        .extensions()
        .get::<RequestTitle>()
        .map(|t| t.0.as_str());
    let client_error_parts = error.map(|e| e.get_client_error(title.map(|t| t.into())));

    let client_error = client_error_parts.as_ref().map(|(_, err)| err);

    // TODO: log line

    // during development, print a newline to separate requests
    #[cfg(debug_assertions)]
    tracing::debug!("DONE-REQUEST");

    let response_error = client_error_parts
        .map(|(code, error)| views::error::error(hx_request, &doc_props, &error).into_response());

    response_error.unwrap_or(resp)
}
