#[cfg(debug_assertions)]
mod dev;

mod fallback;
mod static_serve;

use axum::Router;

use crate::{middleware::mw_res_mapper, routes::fallback::fallback_handler};

pub fn router() -> Router {
    let mut router = Router::new()
        .fallback(fallback_handler)
        .layer(axum::middleware::map_response(mw_res_mapper))
        .nest_service("/static", static_serve::service());
    #[cfg(debug_assertions)]
    {
        router = router.nest("/__dev_only", dev::dev_router());
    }
    router
}
