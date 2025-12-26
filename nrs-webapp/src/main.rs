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
use crate::{config::AppConfig, middleware::mw_res_mapper, routes::router};

#[cfg(debug_assertions)]
mod _dev_utils;
pub mod config;
pub mod error;
pub mod extract;
pub mod middleware;
pub mod model;
pub mod routes;

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

    let routes = router();

    let addr = "0.0.0.0:3621";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{addr}");

    axum::serve(listener, routes.into_make_service()).await?;
    Ok(())
}
