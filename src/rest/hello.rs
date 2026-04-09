use axum::{Router, routing::get};

use crate::state::AppState;

// Example

pub fn hello_router() -> Router<AppState> {
    Router::new().route("/", get(hello_handler))
}

async fn hello_handler() -> &'static str {
    "Hello, World!"
}
