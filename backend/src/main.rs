mod api;
mod db;
mod domain;
mod middleware;

use crate::api::{get_breweries, get_schedules, get_vendors};
use crate::db::Db;
use crate::middleware::request_id_middleware;
use axum::routing::get;
use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Db::init().await?;

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/schedules", get(get_schedules))
        .route("/api/breweries", get(get_breweries))
        .route("/api/vendors", get(get_vendors))
        .layer(axum::middleware::from_fn(request_id_middleware))
        .with_state(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
