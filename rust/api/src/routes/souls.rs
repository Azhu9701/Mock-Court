use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use foundation::{IsmismFilter, IsmismSearch, IsmismCode, SoulListEntry, SoulProfile, SoulMatch, LLMRequest, CallConfig};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use ai_gateway::prompt::PromptBuilder;

use crate::error::{map_api_error, ApiError};
use crate::state::{AppState, AutoCreateEvent};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_souls).post(create_soul))
        .route("/search", get(search_souls))
        .route("/collect", post(collect_soul))
        .route("/refine", post(refine_soul))
        .route("/auto-create", post(auto_create_soul))
        .route("/ismism/distribution", get(ismism_distribution))
        .route("/:name", get(get_soul).put(update_soul).delete(delete_soul))
}

#[derive(Debug, Deserialize)]
struct SoulListQuery {
    field: Option<String>,
    nearest: Option<String>,
    limit: Option<usize>,
}

impl From<SoulListQuery> for IsmismFilter {
    fn from(q: SoulListQuery) -> Self {
        let nearest = q.nearest.and_then(|s| {
            let code = IsmismCode::try_from(s.as_str()).ok()?;
            Some(IsmismSearch {
                target: code,
                weights: None,
                limit: q.limit,
            })
        });
        IsmismFilter {
            field: q.field,
            nearest,
            ..Default::default()
        }
    }
}

async fn list_souls(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SoulListQuery>,
) -> Result<Json<Vec<SoulListEntry>>, (axum::http::StatusCode, Json<ApiError>)> {
    let filter: IsmismFilter = query.into();
    state.registry.list_souls(&filter).map(Json).map_err(map_api_error)
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search_souls(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SoulMatch>>, (axum::http::StatusCode, Json<ApiError>)> {
    state.registry.search_souls(&query.q).map(Json).map_err(map_api_error)
}

async fn ismism_distribution(
    State(state): State<Arc<AppState>>,
) -> Result<Json<foundation::IsmismStats>, (axum::http::StatusCode, Json<ApiError>)> {
    state.registry.get_ismism_distribution().map(Json).map_err(map_api_error)
}

async fn get_soul(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<SoulProfile>, (axum::http::StatusCode, Json<ApiError>)> {
    state.registry.get_soul(&name).map(Json).map_err(map_api_error)
}

#[derive(Debug, Deserialize)]
struct CreateSoulRequest {
    name: String,
    ismism_code: String,
    field: String,
    ontology: String,
    epistemology: String,
    teleology: String,
    #[serde(default)]
    domains: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    summon_prompt: String,
}

async fn create_soul(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateSoulRequest>,
) -> Result<(axum::http::StatusCode, Json<SoulListEntry>), (axum::http::StatusCode, Json<ApiError>)> {
    let profile = SoulProfile {
        name: body.name.clone(),
        ismism_code: body.ismism_code,
        field: body.field,
        ontology: body.ontology,
        epistemology: body.epistemology,
        teleology: body.teleology,
        domains: body.domains,
        exclude_scenarios: vec![],
        summon_count: 0,
        effectiveness: foundation::EffectivenessStats::default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        tags: body.tags,
        summon_prompt: body.summon_prompt,
        practice_observations: vec![],
        title: String::new(), description: String::new(), voice: String::new(), mind: String::new(), self_declare: String::new(), skills_expertise: vec![], model: String::new(), tools: String::new(), trigger_keywords: vec![],
        compat: vec![], incompat: vec![],
    };

    state.registry.create_soul(profile).await.map_err(map_api_error)?;
    let entry = SoulListEntry::from(&state.registry.get_soul(&body.name).map_err(map_api_error)?);
    Ok((
        axum::http::StatusCode::CREATED,
        Json(entry),
    ))
}

#[derive(Debug, Deserialize)]
struct UpdateSoulRequest {
    ismism_code: Option<String>,
    field: Option<String>,
    #[serde(default)]
    domains: Option<Vec<String>>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    summon_prompt: Option<String>,
    #[serde(default)]
    model: Option<String>,
}

async fn update_soul(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(body): Json<UpdateSoulRequest>,
) -> Result<Json<SoulListEntry>, (axum::http::StatusCode, Json<ApiError>)> {
    let mut profile = state.registry.get_soul(&name).map_err(map_api_error)?;
    if let Some(code) = body.ismism_code {
        profile.ismism_code = code;
    }
    if let Some(field) = body.field {
        profile.field = field;
    }
    if let Some(domains) = body.domains {
        profile.domains = domains;
    }
    if let Some(tags) = body.tags {
        profile.tags = tags;
    }
    if let Some(prompt) = body.summon_prompt {
        profile.summon_prompt = prompt;
    }
    if let Some(model) = body.model {
        profile.model = model;
    }
    profile.updated_at = chrono::Utc::now();

    state.registry.update_soul(profile).await.map_err(map_api_error)?;
    let entry = SoulListEntry::from(&state.registry.get_soul(&name).map_err(map_api_error)?);
    Ok(Json(entry))
}

async fn delete_soul(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<(axum::http::StatusCode, ()), (axum::http::StatusCode, Json<ApiError>)> {
    state.registry.delete_soul(&name).await.map_err(map_api_error)?;
    Ok((axum::http::StatusCode::NO_CONTENT, ()))
}

// ── 收魂 ──

#[derive(Debug, Deserialize)]
struct CollectRequest {
    name: String,
    #[serde(default = "default_engine")]
    engine: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_engine() -> String { "baidu".into() }
fn default_limit() -> usize { 5 }

#[derive(Debug, Serialize)]
struct CollectResponse {
    name: String,
    raw_material: String,
    web_search: Option<crate::collector::CollectResult>,
}

async fn collect_soul(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CollectRequest>,
) -> Result<Json<CollectResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    // Fire-and-forget web search in background (can take 30-60s)
    let collector = state.collector.clone();
    let name = body.name.clone();
    let engine = body.engine.clone();
    let limit = body.limit;
    tokio::spawn(async move {
        match collector.collect(&name, Some(&engine), limit).await {
            Ok(r) => tracing::info!("Web 收魂完成: {} → {} 条来源 ({})", name, r.sources, r.raw_path),
            Err(e) => tracing::warn!("Web 收魂失败 ({}): {}", name, e),
        }
    });

    // LLM collect returns immediately
    let prompt_builder = ai_gateway::prompt::PromptBuilder::new();
    let prompt = prompt_builder.build_collect_prompt(&body.name);

    let gateway = state.engine.gateway();
    let provider = gateway.list_providers().into_iter().find(|i| i.available).map(|i| i.provider)
        .ok_or_else(|| (axum::http::StatusCode::SERVICE_UNAVAILABLE, Json(ApiError { error: "No LLM provider".into() })))?;

    let req = LLMRequest { provider, prompt, config: CallConfig { temperature: 0.5, max_tokens: 4096, stream: false, model: None, reasoning_effort: None, structured_output: None, thinking_enabled: None, tools: None, tool_choice: None } };
    let mut rx = gateway.call(&req).map_err(|e| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
    })?;

    let mut raw_material = String::new();
    while let Some(r) = rx.recv().await {
        if let Ok(c) = r { raw_material.push_str(&c.content); }
    }

    Ok(Json(CollectResponse {
        name: body.name,
        raw_material,
        web_search: None,
    }))
}

// ── 炼化 ──

#[derive(Debug, Deserialize)]
struct RefineRequest { raw_material: String, #[serde(default)] name_override: Option<String> }

#[derive(Debug, Serialize)]
struct RefineResponse { profile: SoulProfile, rationale: String }

async fn refine_soul(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefineRequest>,
) -> Result<Json<RefineResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let prompt_builder = ai_gateway::prompt::PromptBuilder::new();
    let prompt = prompt_builder.build_refine_prompt(&body.raw_material);

    let gateway = state.engine.gateway();
    let provider = gateway.list_providers().into_iter().find(|i| i.available).map(|i| i.provider)
        .ok_or_else(|| (axum::http::StatusCode::SERVICE_UNAVAILABLE, Json(ApiError { error: "No LLM provider".into() })))?;

    let req = LLMRequest { provider, prompt, config: CallConfig { temperature: 0.3, max_tokens: 4096, stream: false, model: None, reasoning_effort: None, structured_output: None, thinking_enabled: None, tools: None, tool_choice: None } };
    let mut rx = gateway.call(&req).map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let mut resp = String::new();
    while let Some(r) = rx.recv().await {
        if let Ok(c) = r { resp.push_str(&c.content); }
    }

    let json_str = resp.find('{')
        .and_then(|s| resp.rfind('}').map(|e| &resp[s..=e]))
        .unwrap_or(&resp);
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: format!("Parse error: {}", e) })))?;

    let name = body.name_override.as_deref().unwrap_or(parsed["name"].as_str().unwrap_or("unknown"));
    let ismism_code = parsed["ismism_code"].as_str().unwrap_or("0-0-0-0").to_string();

    let profile = SoulProfile {
        name: name.to_string(), ismism_code,
        field: parsed["field"].as_str().unwrap_or("").into(),
        ontology: parsed["ontology"].as_str().unwrap_or("").into(),
        epistemology: parsed["epistemology"].as_str().unwrap_or("").into(),
        teleology: parsed["teleology"].as_str().unwrap_or("").into(),
        domains: parsed["domains"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        exclude_scenarios: vec![],
        summon_count: 0, effectiveness: foundation::EffectivenessStats::default(),
        created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        tags: parsed["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        summon_prompt: parsed["summon_prompt"].as_str().unwrap_or("").into(),
        practice_observations: vec![],
        title: String::new(), description: String::new(), voice: String::new(), mind: String::new(), self_declare: String::new(), skills_expertise: vec![], model: String::new(), tools: String::new(), trigger_keywords: vec![],
        compat: vec![], incompat: vec![],
    };

    // Auto-save to registry
    let _ = state.registry.create_soul(profile.clone()).await;

    Ok(Json(RefineResponse {
        profile,
        rationale: parsed["rationale"].as_str().unwrap_or("").into(),
    }))
}

// ── 一键收魂炼化 ──

#[derive(Debug, Deserialize)]
struct AutoCreateRequest { name: String }

#[derive(Debug, Serialize)]
struct AutoCreateAccepted { task_id: String, soul_name: String }

async fn auto_create_soul(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AutoCreateRequest>,
) -> Result<Json<AutoCreateAccepted>, (axum::http::StatusCode, Json<ApiError>)> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, Json(ApiError { error: "魂名不能为空".into() })));
    }

    // Cap concurrent auto-create tasks to prevent resource exhaustion
    const MAX_CONCURRENT_TASKS: usize = 10;
    if state.auto_create_tasks.len() >= MAX_CONCURRENT_TASKS {
        return Err((axum::http::StatusCode::TOO_MANY_REQUESTS, Json(ApiError { error: format!("收魂炼化任务过多（最多 {} 个并发），请稍后再试", MAX_CONCURRENT_TASKS) })));
    }

    let task_id = Uuid::new_v4().to_string();
    let (tx, _rx) = broadcast::channel::<AutoCreateEvent>(8);
    state.auto_create_tasks.insert(task_id.clone(), tx.clone());

    // Clone Arc fields before spawn (tokio::spawn requires 'static)
    let gateway = state.engine.gateway().clone();
    let registry = state.registry.clone();
    let tasks = state.auto_create_tasks.clone();

    // Kickoff event
    let _ = tx.send(AutoCreateEvent {
        task_id: task_id.clone(), soul_name: name.clone(),
        phase: "collecting".into(), message: Some("正在搜索资料…".into()), profile: None,
    });
    let task_id_clone = task_id.clone();
    let name_clone = name.clone();
    tracing::info!("auto_create task spawned: task={}, name={}", task_id_clone, name_clone);

    // RAII guard: always clean up DashMap entry even on panic
    struct TaskGuard {
        tasks: Arc<dashmap::DashMap<String, broadcast::Sender<AutoCreateEvent>>>,
        task_id: String,
    }
    impl Drop for TaskGuard {
        fn drop(&mut self) {
            self.tasks.remove(&self.task_id);
        }
    }

    tokio::spawn(async move {
        let _guard = TaskGuard { tasks: tasks.clone(), task_id: task_id_clone.clone() };

        let send = |phase: &str, msg: Option<&str>, p: Option<SoulProfile>| {
            let _ = tx.send(AutoCreateEvent {
                task_id: task_id_clone.clone(), soul_name: name_clone.clone(),
                phase: phase.into(), message: msg.map(String::from), profile: p,
            });
        };

        macro_rules! fail {
            ($msg:expr) => {{
                tracing::error!("auto_create FAILED: task={} name={} reason={}", task_id_clone, name_clone, $msg);
                send("error", Some($msg), None);
                return;
            }};
        }

        tracing::info!("auto_create task started: task={}, name={}", task_id_clone, name_clone);

        let prompt_builder = PromptBuilder::new();
        let providers = gateway.list_providers();
        let first_available = providers.iter().find(|i| i.available);
        let provider = match first_available {
            Some(p) => {
                tracing::info!("auto_create using provider: {:?}", p.provider);
                p.provider.clone()
            }
            None => {
                tracing::error!("auto_create: no LLM provider available. Providers: {:?}", providers.iter().map(|p| format!("{:?}-avail={}", p.provider, p.available)).collect::<Vec<_>>());
                fail!("没有可用的 LLM provider")
            }
        };

        // Step 1: 收魂
        let collect_prompt = prompt_builder.build_collect_prompt(&name_clone);
        tracing::info!("auto_create collecting for: {}", name_clone);
        let collect_req = LLMRequest {
            provider: provider.clone(),
            prompt: collect_prompt,
            config: CallConfig { temperature: 0.5, max_tokens: 4096, stream: false, model: None, reasoning_effort: None, structured_output: None, thinking_enabled: None, tools: None, tool_choice: None },
        };
        let mut collect_rx = match gateway.call(&collect_req) {
            Ok(rx) => rx,
            Err(e) => fail!(&format!("收魂 LLM 调用失败: {}", e)),
        };
        let mut raw_material = String::new();
        while let Some(r) = collect_rx.recv().await {
            if let Ok(c) = r { raw_material.push_str(&c.content); }
        }
        if raw_material.is_empty() {
            tracing::error!("auto_create collect returned empty for: {}", name_clone);
            fail!("收魂返回空内容 — 请检查魂名是否存在");
        }
        tracing::info!("auto_create collected {} bytes of raw material for: {}", raw_material.len(), name_clone);

        send("refining", Some("正在炼化魂 profile…"), None);

        // Step 2: 炼化
        let refine_prompt = prompt_builder.build_refine_prompt(&raw_material);
        let refine_req = LLMRequest {
            provider,
            prompt: refine_prompt,
            config: CallConfig { temperature: 0.3, max_tokens: 4096, stream: false, model: None, reasoning_effort: None, structured_output: None, thinking_enabled: None, tools: None, tool_choice: None },
        };
        let mut refine_rx = match gateway.call(&refine_req) {
            Ok(rx) => rx,
            Err(e) => fail!(&format!("炼化 LLM 调用失败: {}", e)),
        };
        let mut resp = String::new();
        while let Some(r) = refine_rx.recv().await {
            if let Ok(c) = r { resp.push_str(&c.content); }
        }

        let json_str = resp.find('{')
            .and_then(|s| resp.rfind('}').map(|e| &resp[s..=e]))
            .unwrap_or(&resp);
        let parsed: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(p) => p,
            Err(e) => fail!(&format!("炼化 JSON 解析失败: {}", e)),
        };

        let ismism_code = parsed["ismism_code"].as_str().unwrap_or("0-0-0-0").to_string();
        let profile = SoulProfile {
            name: name_clone.clone(), ismism_code,
            field: parsed["field"].as_str().unwrap_or("").into(),
            ontology: parsed["ontology"].as_str().unwrap_or("").into(),
            epistemology: parsed["epistemology"].as_str().unwrap_or("").into(),
            teleology: parsed["teleology"].as_str().unwrap_or("").into(),
            domains: parsed["domains"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
            exclude_scenarios: vec![],
            summon_count: 0, effectiveness: foundation::EffectivenessStats::default(),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            tags: parsed["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
            summon_prompt: parsed["summon_prompt"].as_str().unwrap_or("").into(),
            practice_observations: vec![],
            title: String::new(), description: String::new(), voice: String::new(), mind: String::new(), self_declare: String::new(), skills_expertise: vec![], model: String::new(), tools: String::new(), trigger_keywords: vec![],
            compat: vec![], incompat: vec![],
        };

        let _ = registry.create_soul(profile.clone()).await;
        tracing::info!("auto_create DONE: task={} name={} ismism={}", task_id_clone, name_clone, profile.ismism_code);
        send("done", Some("收魂炼化完成"), Some(profile));
        // Drop guard auto-removes from tasks map
    });

    Ok(Json(AutoCreateAccepted { task_id, soul_name: name }))
}
