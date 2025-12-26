use std::sync::Arc;

use axum::{
    http::{Method, Uri},
    response::{IntoResponse, Response},
};
use axum_htmx::HxRequest;
use nrs_webapp_frontend::views::{self, document::DocumentProps, error::ClientError};
use serde_json::{Value, json};

use crate::{Error, extract::doc_props::DocProps, middleware::mw_req_stamp::ReqStamp};

pub struct RequestTitle(pub String);

pub async fn mw_res_mapper(
    hx_request: HxRequest,
    DocProps(doc_props): DocProps,
    uri: Uri,
    method: Method,
    req_stamp: ReqStamp,
    resp: Response,
) -> Response {
    tracing::debug!("{:<12} -- mw_res_mapper", "MW_RES_MAP");

    let error = resp.extensions().get::<Arc<Error>>().map(Arc::as_ref);
    let title = resp
        .extensions()
        .get::<RequestTitle>()
        .map(|t| t.0.as_str());
    let client_error_parts =
        error.map(|e| e.get_client_error(title.map(|t| t.into()), req_stamp.uuid.to_string()));

    let client_error = client_error_parts.as_ref().map(|(_, err)| err);

    // TODO: log line
    println!(
        "{:<12} -- {}",
        "REQ-LOG-LINE",
        to_log_line(method, uri, req_stamp, error, client_error)
    );

    // during development, print a newline to separate requests
    #[cfg(debug_assertions)]
    tracing::debug!("DONE-REQUEST");

    let response_error = client_error_parts
        .map(|(code, error)| views::error::error(hx_request, &doc_props, &error).into_response());

    response_error.unwrap_or(resp)
}

fn to_log_line(
    method: Method,
    uri: Uri,
    req_stamp: ReqStamp,
    error: Option<&Error>,
    client_error: Option<&ClientError>,
) -> Value {
    json!({
        "uri": uri.to_string(),
        "uuid": req_stamp.uuid,
        "method": method.to_string(),
        "error_type": error.map(|e| e.to_string()),
        "client_error": client_error,
    })
}
