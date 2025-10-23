use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{config::Config, state::AppState};

pub mod config;
pub mod rest;
pub mod state;

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    dotenvy::dotenv().ok();
    let config: Config = envy::from_env()?;
    let state = AppState {
        config: Arc::new(config.clone()),
    };

    let app = Router::new()
        .with_state(state)
        .nest("/hello", rest::hello_router())
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", &addr);
    axum::serve(listener, app).await?;

    Ok(())
}
