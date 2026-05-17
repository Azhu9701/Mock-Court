use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use archive::{BoundaryReview, EffectivenessTrend, Period, SoulAlert, SummonStats};
use foundation::FailureAlert;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/summon-stats", get(summon_stats))
        .route("/soul-effectiveness/:name", get(soul_effectiveness))
        .route("/mode-distribution", get(mode_distribution))
        .route("/unsummoned", get(unsummoned_souls))
        .route("/low-effectiveness", get(low_effectiveness))
        .route("/audit", get(audit_all))
        .route("/audit/:name", get(audit_soul))
        .route("/pleasure-stats", get(pleasure_stats))
}

#[derive(Debug, Deserialize)]
struct StatsQuery {
    period_start: Option<String>,
    period_end: Option<String>,
}

async fn summon_stats(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<SummonStats>, (axum::http::StatusCode, Json<ApiError>)> {
    let start = query
        .period_start
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let end = query
        .period_end
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let period = Period { start, end };
    state.archive.get_summon_stats(period).await.map(Json).map_err(map_api_error)
}

async fn soul_effectiveness(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<EffectivenessTrend>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.get_soul_effectiveness(&name).await.map(Json).map_err(map_api_error)
}

async fn mode_distribution(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HashMap<String, usize>>, (axum::http::StatusCode, Json<ApiError>)> {
    let dist = state.archive.get_mode_distribution().await.map_err(map_api_error)?;
    let converted: HashMap<String, usize> = dist
        .into_iter()
        .map(|(k, v)| (k.as_str().to_string(), v))
        .collect();
    Ok(Json(converted))
}

#[derive(Debug, Deserialize)]
struct UnsummonedQuery {
    #[serde(default = "default_threshold_days")]
    threshold_days: u32,
}

fn default_threshold_days() -> u32 {
    30
}

async fn unsummoned_souls(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UnsummonedQuery>,
) -> Result<Json<Vec<SoulAlert>>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.detect_unsummoned_souls(query.threshold_days).await.map(Json).map_err(map_api_error)
}

#[derive(Debug, Deserialize)]
struct LowEffectivenessQuery {
    #[serde(default = "default_threshold")]
    threshold: f64,
}

fn default_threshold() -> f64 {
    0.3
}

async fn low_effectiveness(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LowEffectivenessQuery>,
) -> Result<Json<Vec<BoundaryReview>>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.detect_low_effectiveness(query.threshold).await.map(Json).map_err(map_api_error)
}

// ── Pleasure Stats ──

async fn pleasure_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<archive::PleasureStats>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.get_pleasure_stats().await.map(Json).map_err(map_api_error)
}

// ── Audit ──

async fn audit_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FailureAlert>>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.check_failure_conditions().await.map(Json).map_err(map_api_error)
}

async fn audit_soul(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Vec<FailureAlert>>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.check_soul_failure_conditions(&name).await.map(Json).map_err(map_api_error)
}
