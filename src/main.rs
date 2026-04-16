use rust_hack_template::{build_router, build_state, config::Config, connect_migrated_pgpool, run};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

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

    let pgpool = connect_migrated_pgpool(&config.database_url).await?;
    let state = build_state(config.clone(), pgpool);
    let app = build_router(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", &addr);
    run(listener, app).await?;

    Ok(())
}
