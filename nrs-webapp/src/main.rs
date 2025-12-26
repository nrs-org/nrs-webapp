#![allow(unused)]

use std::{convert::Infallible, net::SocketAddr};

use axum::{
    Router,
    extract::{MatchedPath, OriginalUri},
    http::StatusCode,
    response::Html,
    routing::get,
};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::error::{Error, Result};
use crate::{config::AppConfig, middleware::mw_res_mapper};

#[cfg(debug_assertions)]
mod _dev_utils;
pub mod config;
pub mod error;
pub mod extract;
pub mod middleware;
pub mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()?)
        .with(
            // Disable timestamps and targets for cleaner output during development
            // TODO: Adjust this for production use
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time(),
        )
        .init();

    #[cfg(debug_assertions)]
    _dev_utils::init_dev().await;

    let mut routes = Router::new()
        .route(
            "/test",
            get(|| async { Html("<h1>con so cua su rau ma</h1>") }),
        )
        .fallback(fallback_handler)
        .layer(axum::middleware::map_response(mw_res_mapper))
        .nest_service("/static", ServeDir::new(&AppConfig::get().STATIC_SERVE_DIR));

    #[cfg(debug_assertions)]
    {
        let live_reload_router = Router::new().route(
            "/",
            get(|| async {
                use std::time::Duration;

                use axum::response::{Sse, sse::Event};
                use tokio::time::interval;
                use tokio_stream::{StreamExt, wrappers::IntervalStream};

                let interval = interval(Duration::from_secs(20));
                let stream = IntervalStream::new(interval)
                    .map(|_| Ok::<_, Infallible>(Event::default().event("ping")));

                Sse::new(stream)
            }),
        );

        routes = routes.nest("/__watch", live_reload_router);
    }

    let addr = "0.0.0.0:3621";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{addr}");

    axum::serve(listener, routes.into_make_service()).await?;
    Ok(())
}

#[axum::debug_handler]
async fn fallback_handler(OriginalUri(uri): OriginalUri) -> Result<Infallible> {
    Err(Error::PageNotFound { uri })
}
