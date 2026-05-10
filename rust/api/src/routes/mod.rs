mod analytics;
mod apikey;
mod archive;
mod config;
mod knowledge;
mod possess;
mod searxng;
mod sessions;
mod souls;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::state::AppState;

pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .nest("/souls", souls::router())
        .nest("/possess", possess::router())
        .nest("/sessions", sessions::router())
        .nest("/analytics", analytics::router())
        .nest("/archive", archive::router())
        .nest("/apikey", apikey::router())
        .nest("/knowledge", knowledge::router())
        .nest("/config", config::router())
        .nest("/searxng", searxng::router())
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}
