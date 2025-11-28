#![allow(unused)]

use std::net::SocketAddr;

use axum::{Router, response::Html, routing::get};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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
