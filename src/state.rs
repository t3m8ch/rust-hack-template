use std::sync::Arc;

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<Config>,
}
