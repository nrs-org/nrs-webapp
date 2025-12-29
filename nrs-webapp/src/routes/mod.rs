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

/// Builds and returns the application's HTTP router with routes, middleware, and static services configured.
///
/// The returned router mounts the root home handler at `/`, nests the authentication router under `/auth` (using
/// the provided `ModelManager`), serves static assets under `/static`, and applies response mapping and request
/// middleware. In debug builds an additional dev-only router is nested at `/__dev_only`. A fallback handler and a
/// method-not-allowed handler are also registered.
///
/// # Parameters
///
/// * `mm` — the `ModelManager` instance used to configure the nested authentication router.
///
/// # Returns
///
/// A fully configured `axum::Router` ready to be served.
///
/// # Examples
///
/// ```no_run
/// use nrs_webapp::routes::router;
/// use nrs_webapp::model::ModelManager;
///
/// // Construct or obtain a ModelManager from your application setup.
/// let mm: ModelManager = unsafe { std::mem::zeroed() };
/// let app = router(mm);
/// // `app` can now be converted into a hyper service and served.
/// ```
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

/// Render the application's home page, returning either a full document or an HTMX partial update.
///
/// This handler produces the appropriate response for the root GET route: a complete HTML document for normal requests
/// or an HTMX-compatible partial response when the request indicates an HTMX interaction.
///
/// # Returns
///
/// An HTTP response containing either the full page document or an HTMX partial update.
///
/// # Examples
///
/// ```no_run
/// use axum::response::IntoResponse;
/// # async fn example(hx_req: nrs_webapp_frontend::HxRequest, props: nrs_webapp_frontend::DocProps) {
/// let response = home(hx_req, props).await;
/// let _ = response.into_response();
/// # }
/// ```
async fn home(hx_req: HxRequest, DocProps(doc_props): DocProps) -> impl IntoResponse {
    tracing::debug!("{:<12} -- GET home", "ROUTE");
    maybe_document(hx_req, doc_props, views::pages::home::home())
}
