#![allow(unused)]

use std::net::SocketAddr;

use axum::{Router, response::Html, routing::get};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(debug_assertions)]
mod _dev_utils;
pub mod config;
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

    let routes = Router::new().route(
        "/test",
        get(|| async { Html("<h1>con so cua su rau ma</h1>") }),
    );

    let addr = "0.0.0.0:3621";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{addr}");

    axum::serve(listener, routes.into_make_service()).await?;
    Ok(())
}
