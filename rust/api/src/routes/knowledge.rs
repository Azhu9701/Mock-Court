use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use foundation::KnowledgeResult;
use serde::Deserialize;

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search", get(search))
        .route("/rebuild", post(rebuild))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize { 20 }

async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<KnowledgeResult>>, (axum::http::StatusCode, Json<ApiError>)> {
    state
        .archive
        .search_knowledge(&query.q, query.limit)
        .await
        .map(Json)
        .map_err(map_api_error)
}

async fn rebuild(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ApiError>)> {
    let count = state.archive.rebuild_fts().await.map_err(map_api_error)?;
    Ok(Json(serde_json::json!({ "indexed": count })))
}
