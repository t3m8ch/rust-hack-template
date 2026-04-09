use std::sync::Arc;

use sqlx::{Pool, Postgres};

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pgpool: Pool<Postgres>,
}
