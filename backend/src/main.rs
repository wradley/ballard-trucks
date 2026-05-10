mod api;
mod app_state;
mod db;
mod domain;
mod middleware;

use crate::api::{get_breweries, get_schedules, get_vendors, post_ingest_schedules};
use crate::app_state::AppState;
use crate::middleware::request_id_middleware;
use axum::Router;
use axum::routing::get;
use std::env;
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

    let sqlite_path_override = parse_sqlite_path_override(env::args())?;
    let state = AppState::init(sqlite_path_override).await?;

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/schedules", get(get_schedules))
        .route("/api/breweries", get(get_breweries))
        .route("/api/vendors", get(get_vendors))
        .route(
            "/internal/ingest/schedules",
            axum::routing::post(post_ingest_schedules),
        )
        .layer(axum::middleware::from_fn(request_id_middleware))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn parse_sqlite_path_override(
    mut args: impl Iterator<Item = String>,
) -> anyhow::Result<Option<String>> {
    // Skip binary name.
    let _ = args.next();

    let mut sqlite_path_override = None;

    while let Some(arg) = args.next() {
        if arg == "--sqlite-path" {
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("Missing value for --sqlite-path"))?;
            sqlite_path_override = Some(value);
        }
    }

    Ok(sqlite_path_override)
}
