mod handlers;
mod routes;

use axum::{
    routing::get,
    Router,
};
use sqlx::{Pool, Sqlite};
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
}

pub async fn run_server(pool: Pool<Sqlite>) -> anyhow::Result<()> {
    let state = AppState { db: pool };

    let app = Router::new()
        .route("/health", get(|| async { "Backend is running" }))
        .merge(routes::api_routes())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}