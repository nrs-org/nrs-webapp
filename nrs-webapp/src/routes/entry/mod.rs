use crate::Result;
use crate::extract::doc_props::DocProps;
use crate::model::entity::ListPayload;
use crate::model::entry::alias::EntryAliasBmc;
use crate::model::entry::{Entry, EntryBmc};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Router, extract::Path, routing::get};
use axum_htmx::{HxRedirect, HxRequest};
use nrs_webapp_frontend::maybe_document;
use nrs_webapp_frontend::views::pages::entry::details::{EntryDetails, entry_details_page};
use nrs_webapp_frontend::views::pages::entry::list::{EntryListEntry, entry_list_page};
use reqwest::StatusCode;

use crate::model::ModelManager;

pub fn router() -> Router<ModelManager> {
    Router::new()
        .route("/", get(get_all))
        .route("/{id}", get(get_by_id))
}

pub async fn get_all(
    hx_request: HxRequest,
    DocProps(props): DocProps,
    State(mut mm): State<ModelManager>,
) -> Result<Response> {
    tracing::debug!("{:<12} -- GET entry::get_all", "ROUTE");

    let payload = ListPayload {
        offset: Some(0),
        limit: Some(10),
        ..Default::default()
    };

    let entries: Vec<Entry> = EntryBmc::list_entries(&mut mm, payload).await?;
    let entries = entries
        .into_iter()
        .map(|e| EntryListEntry {
            id: e.id,
            title: e.title,
            entry_type: e.entry_type,
            added_by: e.added_by.username,
        })
        .collect::<Vec<_>>();

    Ok(maybe_document(hx_request, props, entry_list_page(&entries)).into_response())
}

pub async fn get_by_id(
    hx_request: HxRequest,
    DocProps(props): DocProps,
    Path(id): Path<String>,
    State(mut mm): State<ModelManager>,
) -> Result<Response> {
    tracing::debug!("{:<12} -- GET entry::get_by_id {}", "ROUTE", id);

    if let Some(id) = EntryAliasBmc::get_new_id(&mut mm, id.clone()).await? {
        let redirect_uri = format!("/entry/{}", id);
        // redirect to the new id
        match hx_request {
            HxRequest(true) => {
                return Ok((
                    HxRedirect(format!("/entry/{}", id)),
                    StatusCode::PERMANENT_REDIRECT,
                )
                    .into_response());
            }
            HxRequest(false) => {
                return Ok(Redirect::permanent(&redirect_uri).into_response());
            }
        }
    }

    let entry = EntryBmc::get_details(&mut mm, id).await?;
    let entry_details = EntryDetails {
        id: entry.id,
        title: entry.title,
        entry_type: entry.entry_type,
        added_by_id: entry.added_by.id.to_string(),
        added_by_username: entry.added_by.username,
        info_json: format!("{:#}", entry.entry_info.0),
    };

    Ok(maybe_document(hx_request, props, entry_details_page(&entry_details)).into_response())
}
