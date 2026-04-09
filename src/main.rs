use std::sync::Arc;

use axum::Router;
use socketioxide::{SocketIo, extract::SocketRef};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{config::Config, state::AppState};

pub mod auth;
pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod extractors;
pub mod rest;
pub mod state;
pub mod ws;

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

    let pgpool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    let state = AppState {
        config: Arc::new(config.clone()),
        pgpool,
    };

    let (ws_layer, ws_io) = SocketIo::new_layer();
    ws_io.ns("/", |s: SocketRef| {
        ws::hello(&s);
    });

    let app = Router::new()
        .nest("/auth", rest::auth_router())
        .nest("/hello", rest::hello_router())
        .layer(ws_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", &addr);
    axum::serve(listener, app).await?;

    Ok(())
}
