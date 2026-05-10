use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use archive::{ArchiveVerification, ExportStatus};
use serde::Serialize;

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/verify/:session_id", get(verify_archive))
        .route("/export", post(export_archive))
        .route("/export/:task_id", get(export_status))
}

async fn verify_archive(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<ArchiveVerification>, (axum::http::StatusCode, Json<ApiError>)> {
    state.archive.verify_archive(&session_id).await.map(Json).map_err(map_api_error)
}

#[derive(Debug, Serialize)]
struct ExportResponse {
    task_id: String,
    status: String,
}

async fn export_archive(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ExportResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let task_id = state.archive.export_archive().await.map_err(map_api_error)?;
    Ok(Json(ExportResponse {
        task_id,
        status: "started".into(),
    }))
}

#[derive(Debug, Serialize)]
struct ExportStatusResponse {
    task_id: String,
    status: ExportStatus,
}

async fn export_status(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> Json<ExportStatusResponse> {
    let status = state.archive.export_status(&task_id).unwrap_or(ExportStatus::Failed("not found".into()));
    Json(ExportStatusResponse { task_id, status })
}
