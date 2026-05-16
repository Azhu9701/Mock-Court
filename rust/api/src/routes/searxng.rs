use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search", get(search))
        .route("/topic-search", get(topic_search))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_page")]
    pageno: u32,
    #[serde(default = "default_lang")]
    language: String,
    categories: Option<String>,
}

fn default_page() -> u32 { 1 }
fn default_lang() -> String { "zh".into() }

#[derive(Debug, Serialize)]
struct SearxngResponse {
    query: String,
    number_of_results: u64,
    results: Vec<SearxngResult>,
    suggestions: Vec<String>,
    unresponsive_engines: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct SearxngResult {
    title: String,
    url: String,
    content: String,
    engine: String,
    engines: Vec<String>,
    score: f64,
    category: String,
}

#[derive(Debug, Deserialize)]
struct SearxngRawResponse {
    query: Option<String>,
    number_of_results: Option<u64>,
    results: Option<Vec<SearxngRawResult>>,
    suggestions: Option<Vec<String>>,
    unresponsive_engines: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct SearxngRawResult {
    title: String,
    url: String,
    content: Option<String>,
    engine: String,
    engines: Option<Vec<String>>,
    score: Option<f64>,
    category: Option<String>,
}

async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearxngResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let searxng_url = &state.config.searxng_url;
    let mut url = format!("{}/search", searxng_url.trim_end_matches('/'));

    url.push_str(&format!("?format=json&q={}", urlencoding(&query.q)));
    url.push_str(&format!("&pageno={}", query.pageno));
    url.push_str(&format!("&language={}", query.language));
    if let Some(ref categories) = query.categories {
        url.push_str(&format!("&categories={}", urlencoding(categories)));
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| {
            let api_err = ApiError { error: format!("SearXNG 请求失败: {e}") };
            (axum::http::StatusCode::BAD_GATEWAY, Json(api_err))
        })?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| {
        let api_err = ApiError { error: format!("SearXNG 响应读取失败: {e}") };
        (axum::http::StatusCode::BAD_GATEWAY, Json(api_err))
    })?;

    if !status.is_success() {
        let api_err = ApiError { error: format!("SearXNG 返回错误 ({}): {body}", status.as_u16()) };
        return Err((axum::http::StatusCode::BAD_GATEWAY, Json(api_err)));
    }

    let raw: SearxngRawResponse = serde_json::from_str(&body).map_err(|e| {
        let api_err = ApiError { error: format!("SearXNG 响应解析失败: {e}") };
        (axum::http::StatusCode::BAD_GATEWAY, Json(api_err))
    })?;

    let results = raw.results.unwrap_or_default().into_iter().map(|r| SearxngResult {
        title: r.title,
        url: r.url,
        content: r.content.unwrap_or_default(),
        engine: r.engine,
        engines: r.engines.unwrap_or_default(),
        score: r.score.unwrap_or(0.0),
        category: r.category.unwrap_or_else(|| "general".into()),
    }).collect();

    Ok(Json(SearxngResponse {
        query: raw.query.unwrap_or(query.q),
        number_of_results: raw.number_of_results.unwrap_or(0),
        results,
        suggestions: raw.suggestions.unwrap_or_default(),
        unresponsive_engines: raw.unresponsive_engines.unwrap_or_default(),
    }))
}

#[derive(Debug, Deserialize)]
struct TopicSearchQuery {
    q: String,
    #[serde(default = "default_engine")]
    engine: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_engine() -> String { "bing".into() }
fn default_limit() -> usize { 5 }

#[derive(Debug, Serialize)]
struct TopicSearchResponse {
    query: String,
    engine: String,
    markdown: String,
}

async fn topic_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TopicSearchQuery>,
) -> Result<Json<TopicSearchResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let markdown = state.collector.search_topic(&query.q, Some(&query.engine), query.limit).await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, Json(ApiError { error: e })))?;
    Ok(Json(TopicSearchResponse {
        query: query.q,
        engine: query.engine,
        markdown,
    }))
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
