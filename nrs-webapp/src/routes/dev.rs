use axum::{Router, routing::get};
use std::{convert::Infallible, time::Duration};

use axum::response::{Sse, sse::Event};
use tokio::time::interval;
use tokio_stream::{StreamExt, wrappers::IntervalStream};

pub fn dev_router() -> Router {
    Router::new().route(
        "/livereload",
        get(|| async {
            let interval = interval(Duration::from_secs(20));
            let stream = IntervalStream::new(interval)
                .map(|_| Ok::<_, Infallible>(Event::default().event("ping")));

            Sse::new(stream)
        }),
    )
}
