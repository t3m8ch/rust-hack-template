use axum::{Router, routing::get};

// Example

pub fn hello_router() -> Router {
    Router::new().route("/", get(hello_handler))
}

async fn hello_handler() -> &'static str {
    "Hello, World!"
}
