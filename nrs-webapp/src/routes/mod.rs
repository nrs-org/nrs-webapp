#[cfg(debug_assertions)]
mod dev;

mod auth;
mod fallback;
mod static_serve;

use axum::{Router, response::IntoResponse, routing::get};
use axum_htmx::HxRequest;
use nrs_webapp_frontend::{maybe_document, views};

use crate::{
    config::AppConfig,
    extract::doc_props::DocProps,
    middleware::{
        mw_req_session::mw_req_session, mw_req_stamp::mw_req_stamp, mw_res_map::mw_res_mapper,
    },
    model::ModelManager,
    routes::fallback::{fallback_handler, method_not_allowed_fallback_handler},
};

pub fn router(mm: ModelManager) -> Router {
    let mut router = Router::new()
        .route("/", get(home))
        .nest("/auth", auth::router(mm))
        .fallback(fallback_handler)
        .method_not_allowed_fallback(method_not_allowed_fallback_handler)
        .layer(axum::middleware::map_response(mw_res_mapper))
        .layer(axum::middleware::from_fn(mw_req_session))
        .layer(axum::middleware::from_fn(mw_req_stamp))
        .layer(AppConfig::get().IP_SOURCE.clone().into_extension())
        .nest_service("/static", static_serve::service());
    #[cfg(debug_assertions)]
    {
        router = router.nest("/__dev_only", dev::dev_router());
    }
    router
}

async fn home(hx_req: HxRequest, DocProps(doc_props): DocProps) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET home", "ROUTE");
    maybe_document(hx_req, doc_props, views::pages::home::home())
}
