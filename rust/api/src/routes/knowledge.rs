use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use foundation::{KnowledgeCard, KnowledgeCardFilter, KnowledgeResult, KnowledgeTopic};
use serde::Deserialize;

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search", get(search))
        .route("/rebuild", post(rebuild))
        .route("/cards", get(list_cards))
        .route("/topics", get(list_topics))
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

#[derive(Debug, Deserialize)]
struct CardsQuery {
    soul: Option<String>,
    #[serde(default = "default_limit_u32")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

fn default_limit_u32() -> u32 { 20 }

async fn list_cards(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CardsQuery>,
) -> Result<Json<Vec<KnowledgeCard>>, (axum::http::StatusCode, Json<ApiError>)> {
    let filter = KnowledgeCardFilter {
        soul_name: query.soul,
        tags: None,
        limit: Some(query.limit),
        offset: Some(query.offset),
    };
    state
        .archive
        .get_knowledge_cards_list(&filter)
        .await
        .map(Json)
        .map_err(map_api_error)
}

#[derive(Debug, Deserialize)]
struct TopicsQuery {
    mode: Option<String>,
    #[serde(default = "default_limit_usize")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_limit_usize() -> usize { 20 }

async fn list_topics(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TopicsQuery>,
) -> Result<Json<Vec<KnowledgeTopic>>, (axum::http::StatusCode, Json<ApiError>)> {
    state
        .archive
        .list_knowledge_topics(query.mode.as_deref(), query.limit, query.offset)
        .await
        .map(Json)
        .map_err(map_api_error)
}
