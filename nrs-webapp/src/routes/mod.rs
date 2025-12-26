#[cfg(debug_assertions)]
mod dev;

mod fallback;
mod static_serve;

use axum::{Router, response::IntoResponse, routing::get};
use axum_htmx::HxRequest;
use nrs_webapp_frontend::{maybe_document, views};

use crate::{
    extract::doc_props::DocProps, middleware::mw_res_mapper, routes::fallback::fallback_handler,
};

pub fn router() -> Router {
    let mut router = Router::new()
        .route("/", get(home))
        .fallback(fallback_handler)
        .layer(axum::middleware::map_response(mw_res_mapper))
        .nest_service("/static", static_serve::service());
    #[cfg(debug_assertions)]
    {
        router = router.nest("/__dev_only", dev::dev_router());
    }
    router
}

async fn home(hx_req: HxRequest, DocProps(doc_props): DocProps) -> impl IntoResponse {
    maybe_document(hx_req, doc_props, views::pages::home::home())
}
