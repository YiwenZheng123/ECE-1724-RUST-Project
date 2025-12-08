use axum::{
    routing::post,
    Router,
};
use crate::backend::{handlers, AppState};

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/api/sync", post(handlers::sync_handler))
}