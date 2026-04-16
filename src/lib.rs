use std::sync::Arc;

use axum::Router;
use socketioxide::{SocketIo, extract::SocketRef};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub mod auth;
pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod extractors;
pub mod rest;
pub mod state;
pub mod ws;

use crate::{config::Config, state::AppState};

pub fn build_state(config: Config, pgpool: PgPool) -> AppState {
    AppState {
        config: Arc::new(config),
        pgpool,
    }
}

pub fn build_router(state: AppState) -> Router {
    let (ws_layer, ws_io) = SocketIo::new_layer();
    ws_io.ns("/", |socket: SocketRef| {
        ws::hello(&socket);
    });

    Router::new()
        .nest("/auth", rest::auth_router())
        .nest("/hello", rest::hello_router())
        .layer(ws_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn run(listener: TcpListener, app: Router) -> anyhow::Result<()> {
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn connect_pgpool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn migrate_pgpool(pgpool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pgpool).await
}

pub async fn connect_migrated_pgpool(database_url: &str) -> anyhow::Result<PgPool> {
    let pgpool = connect_pgpool(database_url).await?;
    migrate_pgpool(&pgpool).await?;
    Ok(pgpool)
}
