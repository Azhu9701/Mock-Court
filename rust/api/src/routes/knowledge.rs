use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use foundation::{CallConfig, KnowledgeCard, KnowledgeCardFilter, KnowledgeResult, KnowledgeTopic, LLMRequest, Provider};
use serde::Deserialize;

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search", get(search))
        .route("/rebuild", post(rebuild))
        .route("/cards", get(list_cards))
        .route("/topics", get(list_topics))
        .route("/distill-reviews", post(distill_reviews))
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

/// POST /knowledge/distill-reviews
/// 从近期实践反馈蒸馏元知识："什么视角常被忽视"
async fn distill_reviews(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ApiError>)> {
    let reviews = state.archive.get_recent_reviews(20).await.map_err(map_api_error)?;
    if reviews.len() < 2 {
        return Ok(Json(serde_json::json!({"ok": false, "message": "样本不足，至少需要 2 条反馈"})));
    }

    let triples: Vec<(String, String, String)> = reviews.iter().map(|r| {
        (r.most_unexpected.clone(), r.already_known.clone(), r.self_negation.clone())
    }).collect();

    let pb = ai_gateway::prompt::PromptBuilder::new();
    let prompt = pb.build_review_distill_prompt(&triples);

    let gateway = state.engine.gateway().clone();
    let provider = gateway.list_providers().into_iter()
        .find(|i| i.available).map(|i| i.provider).unwrap_or(Provider::Claude);

    let req = LLMRequest {
        provider,
        prompt,
        config: CallConfig { temperature: 0.4, max_tokens: 1024, stream: false, ..Default::default() },
    };

    let mut rx = gateway.call(&req).map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
    })?;

    let mut content = String::new();
    while let Some(r) = rx.recv().await {
        if let Ok(c) = r { content.push_str(&c.content); }
    }

    // Store as a knowledge card
    let card = KnowledgeCard {
        id: uuid::Uuid::new_v4().to_string(),
        title: "实践反馈元知识蒸馏".into(),
        content: content.clone(),
        source_soul: None,
        source_session: None,
        tags: vec!["元知识".into(), "反馈蒸馏".into()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let _ = state.archive.insert_knowledge_card(&card).await;

    Ok(Json(serde_json::json!({"ok": true, "card_id": card.id, "content": content})))
}
