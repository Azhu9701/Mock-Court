use std::sync::Arc;

use axum::extract::{Multipart, Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use foundation::{LLMRequest, CallConfig, Prompt, PromptMessage, Provider, SoulListEntry, SoulProfile};
use possession::PossessionInput;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

use crate::error::{map_api_error, ApiError};
use crate::ocr;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(start_possession))
        .route("/analyze", post(analyze_task))
        .route("/ocr", post(ocr_upload))
        .route("/:session_id/status", get(possession_status))
        .route("/:session_id/follow-up", post(follow_up))
}

#[derive(Debug, Deserialize)]
struct StartPossessionRequest {
    #[serde(default)] mode: Option<String>,
    task: String,
    #[serde(default)] souls: Vec<String>,
    #[serde(default)] topic: Option<String>,
    #[serde(default)] judgment: Option<String>,
    #[serde(default)] worry: Option<String>,
    #[serde(default)] unknown: Option<String>,
    #[serde(default)] task_cards: std::collections::HashMap<String, String>,
    #[serde(default)] search_topic: bool,
}

#[derive(Debug, Serialize)]
struct StartPossessionResponse { session_id: String, mode: String, ws_url: String }

async fn start_possession(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StartPossessionRequest>,
) -> Result<Json<StartPossessionResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    if body.task.trim().is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, Json(ApiError { error: "task is required".into() })));
    }
    tracing::info!("Starting possession with {} souls, search_topic={}", body.souls.len(), body.search_topic);
    
    // 议题搜索：在召唤魂之前，通过 SearXNG 实时搜索 Web 获取背景信息
    let search_results = if body.search_topic {
        tracing::info!("Running SearXNG topic search for: {}", &body.task[..body.task.len().min(80)]);
        match state.collector.search_topic_searxng(&body.task, 3).await {
            Ok(md) => {
                tracing::info!("SearXNG topic search returned {} bytes", md.len());
                Some(md)
            }
            Err(e) => {
                tracing::warn!("SearXNG topic search failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let input = PossessionInput {
        mode: body.mode.and_then(|m| foundation::PossessionMode::from_str(&m)),
        task: body.task, souls: body.souls, topic: body.topic,
        judgment: body.judgment, worry: body.worry, unknown: body.unknown,
        task_cards: body.task_cards,
        search_topic: body.search_topic,
        search_results,
    };
    let (tx, rx) = tokio::sync::mpsc::channel::<possession::WsEvent>(256);
    let session_id = state.engine.start_possession(input, tx).await.map_err(map_api_error)?;
    tracing::info!("Possession session created: {}", session_id);

    // Relay: forward events from dispatch_mode to WebSocket subscribers
    let ws = state.engine.ws_manager().clone();
    let sid = session_id.clone();
    tokio::spawn(async move {
        let mut rx = rx;
        while let Some(event) = rx.recv().await {
            ws.broadcast_system(&sid, &event);
        }
        // Don't remove session - keep it for follow-up questions
    });

    Ok(Json(StartPossessionResponse {
        ws_url: format!("/ws/possess/{}/main", session_id), session_id, mode: "unknown".into(),
    }))
}

// ── AI Analyze: Matching + Review as separate sub-agent calls ──

#[derive(Debug, Deserialize)]
struct AnalyzeRequest { task: String, #[serde(default)] judgment: Option<String>, #[serde(default)] worry: Option<String>, #[serde(default)] unknown: Option<String>, #[serde(default)] reviewer: Option<String> }

#[derive(Debug, Serialize)]
struct AnalyzeResponse { entry_type: String, matched_souls: Vec<SoulMatch>, recommended_mode: String, review: ReviewResult, #[serde(default)] task_cards: std::collections::HashMap<String, String> }

#[derive(Debug, Clone, Serialize)]
struct SoulMatch { name: String, field: String, ismism_code: String, rationale: String }

#[derive(Debug, Serialize)]
struct ReviewResult { verdict: String, checks: Vec<String>, notes: String, reviewer: String }

fn pick_provider(state: &AppState) -> Result<Provider, (axum::http::StatusCode, Json<ApiError>)> {
    state.engine.gateway().list_providers().into_iter().find(|i| i.available).map(|i| i.provider)
        .ok_or_else(|| {
            tracing::error!("No LLM provider available");
            (axum::http::StatusCode::SERVICE_UNAVAILABLE, Json(ApiError { error: "No LLM provider".into() }))
        })
}

/// Extract a JSON object from text that may contain reasoning prose before/after the JSON.
fn extract_json(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end > start {
        Some(&text[start..=end])
    } else {
        None
    }
}

fn build_soul_match(js: &serde_json::Value, all_souls: &[SoulListEntry]) -> Option<SoulMatch> {
    let name = js["name"].as_str()?;
    all_souls.iter().find(|e| e.name == name).map(|entry| SoulMatch {
        name: name.into(),

        field: entry.field.clone(),
        ismism_code: entry.ismism_code.clone(),
        rationale: js["rationale"].as_str().unwrap_or("").into(),
    })
}
async fn llm_call_json(state: &AppState, provider: Provider, system_prompt: Option<&str>, user_prompt: &str, temp: f64, max_tokens: u32, model: Option<&str>) -> Result<serde_json::Value, (axum::http::StatusCode, Json<ApiError>)> {
    let mut messages = Vec::new();
    if let Some(sys) = system_prompt {
        messages.push(PromptMessage { role: "system".into(), content: sys.into(), reasoning_content: None, tool_call_id: None, tool_calls: None });
    }
    messages.push(PromptMessage { role: "user".into(), content: user_prompt.into(), reasoning_content: None, tool_call_id: None, tool_calls: None });

    let req = LLMRequest { provider, prompt: Prompt { messages }, config: CallConfig { temperature: temp, max_tokens, stream: false, model: model.map(String::from), reasoning_effort: None, structured_output: None, thinking_enabled: None, tools: None, tool_choice: None } };
    let mut rx = state.engine.gateway().call(&req).map_err(|e| {
        tracing::error!("LLM call failed: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
    })?;

    let mut resp = String::new();
    while let Some(result) = rx.recv().await {
        if let Ok(chunk) = result { resp.push_str(&chunk.content); }
    }
    // Extract JSON from response — DeepSeek reasoning models may prepend thinking text
    let json_str = extract_json(&resp).unwrap_or(&resp);
    tracing::debug!(len = resp.len(), json_len = json_str.len(), "llm_call_json response");
    Ok(serde_json::from_str(json_str).unwrap_or_else(|e| {
        // Safely truncate to avoid char boundary errors
        let trunc_len = resp.len().min(300);
        let safe_idx = (0..=trunc_len).rev().find(|&i| resp.is_char_boundary(i)).unwrap_or(0);
        tracing::warn!("llm_call_json parse failed: {} | raw[..{}]={}", e, safe_idx, &resp[..safe_idx]);
        serde_json::Value::default()
    }))
}

fn spawn_banner_lord_review(
    gateway: Arc<ai_gateway::GatewayRegistry>,
    banner_lord: SoulProfile,
    task: String,
    candidate_profiles: Vec<SoulProfile>,
    judgment: String,
    worry: String,
    unknown: String,
) -> tokio::task::JoinHandle<Result<serde_json::Value, String>> {
    tokio::spawn(async move {
        let provider = gateway
            .list_providers()
            .into_iter()
            .find(|i| i.available)
            .map(|i| i.provider)
            .ok_or_else(|| "No LLM provider available".to_string())?;

        let review_system = format!(
            "{}你是{}，ismism坐标{}。你作为幡主审查官，需要完成两项任务：\n\
             1. 审查候选魂是否适合这个任务——不适合的要去掉或替换\n\
             2. 为每个确定使用的魂分派一个**差异化的子问题**——不是所有人分析同一个问题，\
             而是把你的总任务拆解成每个魂最擅长回答的那一个侧面\n\n\
             不读取文件——所有上下文已在 prompt 中。",
            banner_lord.summon_prompt, banner_lord.name, banner_lord.ismism_code
        );

        let mut candidates_info = String::new();
        for p in &candidate_profiles {
            let exclude_str = p.exclude_scenarios.join("、");
            candidates_info.push_str(&format!(
                "- **{}** [{}] ismism={} self_declare=\"{}\" exclude_scenarios=\"{}\"\n",
                p.name, p.field, p.ismism_code,
                if p.self_declare.is_empty() { "无" } else { &p.self_declare },
                if p.exclude_scenarios.is_empty() { "无" } else { &exclude_str }
            ));
        }

        let review_user = format!(
            "## 总任务\n{}\n\n## 使用者预设\n判断：{}\n担忧：{}\n未知：{}\n\n## 候选魂\n{}\n\n\
             ## 你的两阶段任务\n\n\
             ### 第一阶段：审查魂组合\n\
             逐魂检查：领域覆盖、场域定位、魂间互补、视角缺失。裁决：pass / conditional / reject\n\n\
             ### 第二阶段：差异化任务分派\n\
             为每个确认使用的魂分配一个**只有他能回答好的子问题**。原则：\n\
             - 利用场域差异——场域1做地基（\"这是什么\"），场域2做边界（\"这看不到什么\"），\
             场域3做自反（\"这个问法本身有什么问题\"），场域4做实践（\"怎么落地\"）\n\
             - 每个子问题要具体（\"请回答：X在Y条件下的Z\"），不要\"请分析\"这种空指令\n\n\
             返回JSON：\
             {{\"verdict\":\"pass|conditional|reject\",\
             \"verified_souls\":[\"魂名\"],\
             \"task_cards\":{{\"魂名\":\"专属子问题\"}},\
             \"checks\":[\"审查结果\"],\
             \"notes\":\"审查备注\",\
             \"missing_perspectives\":[\"缺失视角\"],\
             \"boundary_risks\":[\"边界风险\"]}}",
            task,
            &judgment,
            &worry,
            &unknown,
            candidates_info
        );

        let messages = vec![
            PromptMessage {
                role: "system".into(),
                content: review_system,
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            },
            PromptMessage {
                role: "user".into(),
                content: review_user,
                reasoning_content: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        let req = LLMRequest {
            provider,
            prompt: Prompt { messages },
            config: CallConfig {
                temperature: 0.3,
                max_tokens: 4096,
                stream: false,
                model: None,
                reasoning_effort: None,
                structured_output: None,
                thinking_enabled: None,
                tools: None,
                tool_choice: None,
            },
        };

        let mut rx = gateway
            .call(&req)
            .map_err(|e| format!("LLM call failed: {}", e))?;
        let mut resp = String::new();
        while let Some(result) = rx.recv().await {
            if let Ok(chunk) = result {
                resp.push_str(&chunk.content);
            }
        }

        let json_str = extract_json(&resp).unwrap_or(&resp);
        Ok(serde_json::from_str(json_str).unwrap_or_default())
    })
}

// ── Ismism Inference Engine ──
// 主义主义四位坐标体系：基于未明子的哲学分类学，从任务文本中推断
// 最可能的 ismism 坐标，用于提升匹配精度。

/// 任务中检测到的主义主义信号
#[derive(Debug, Default)]
struct TaskIsmismProfile {
    /// 每个 field (1-4) 的置信度，归一化到 [0, 1]
    field_weights: [f64; 4],
    /// 推断的 F-O-E-T 坐标（None 表示该维度不确定）
    inferred: Option<[u8; 4]>,
}

/// 主义主义术语 → 坐标映射。每个条目：(关键词, [F, O, E, T] 或 [F, 0, 0, 0])
/// 坐标来源于「主义主义完整目录_未明子原版」
const ISMISM_TERMS: &[(&str, [u8; 4])] = &[
    // ── Field 1: 形而下学（气学）──
    ("物理主义", [1, 1, 1, 0]),
    ("科学实在论", [1, 1, 0, 0]),
    ("建构论", [1, 1, 2, 0]),
    ("认知主义", [1, 1, 3, 0]),
    ("行为主义", [1, 1, 4, 0]),
    ("宗教实在论", [1, 2, 0, 0]),
    ("神创论", [1, 2, 1, 0]),
    ("偶像崇拜", [1, 2, 2, 0]),
    ("唯灵论", [1, 2, 3, 0]),
    ("反偶像崇拜", [1, 2, 4, 0]),
    ("唯我论", [1, 3, 0, 0]),
    ("伪唯心主义", [1, 3, 1, 0]),
    ("客观唯心", [1, 3, 1, 1]),
    ("主观唯心", [1, 3, 1, 2]),
    ("本真主义", [1, 3, 2, 0]),
    ("唯意志主义", [1, 3, 3, 0]),
    ("直觉主义", [1, 3, 4, 0]),
    ("平庸主义", [1, 4, 0, 0]),
    ("自然主义", [1, 4, 1, 0]),
    ("世俗人道", [1, 4, 2, 0]),
    ("心理主义", [1, 4, 3, 0]),
    ("庸俗主义", [1, 4, 4, 0]),

    // ── Field 2: 形而上学（道学）──
    ("在场形而上学", [2, 1, 0, 0]),
    ("普遍主义", [2, 1, 1, 0]),
    ("本质主义", [2, 1, 2, 0]),
    ("合理主义", [2, 1, 3, 0]),
    ("绝对主义", [2, 1, 4, 0]),
    ("辩证形而上学", [2, 2, 0, 0]),
    ("无限主义", [2, 2, 1, 0]),
    ("否定主义", [2, 2, 2, 0]),
    ("超验主义", [2, 2, 3, 0]),
    ("我思形而上学", [2, 3, 0, 0]),
    ("实体一元论", [2, 3, 1, 0]),
    ("理性主义", [2, 3, 2, 0]),
    ("反形而上学", [2, 4, 0, 0]),
    ("经验主义", [2, 4, 1, 0]),
    ("实证主义", [2, 4, 2, 0]),
    ("逻辑还原", [2, 4, 3, 0]),
    ("实用主义", [2, 4, 4, 0]),
    ("辩证唯物主义", [2, 4, 4, 2]),

    // ── Field 3: 观念论（心学）──
    ("现象学", [3, 1, 0, 0]),
    ("先验现象学", [3, 1, 1, 0]),
    ("象征主义", [3, 1, 2, 0]),
    ("生活世界", [3, 1, 3, 0]),
    ("德国观念论", [3, 2, 0, 0]),
    ("批判哲学", [3, 2, 1, 0]),
    ("知识学", [3, 2, 2, 0]),
    ("生存论", [3, 2, 3, 0]),
    ("辩证法", [3, 2, 4, 0]),
    ("存在主义", [3, 3, 1, 0]),
    ("尼采", [3, 3, 3, 0]),
    ("符号学", [3, 4, 0, 0]),
    ("结构主义", [3, 4, 1, 0]),
    ("后结构主义", [3, 4, 2, 0]),
    ("差异的辩证法", [3, 4, 3, 0]),
    ("解释学", [3, 4, 4, 0]),
    ("精神分析", [3, 4, 4, 4]),

    // ── Field 4: 实践 · 辩证唯物主义 ──
    ("政治经济学批判", [4, 1, 0, 0]),
    ("资本主义", [4, 1, 1, 0]),
    ("文化霸权", [4, 1, 3, 4]),
    ("意识形态批判", [4, 1, 0, 0]),
    ("列宁", [4, 1, 4, 3]),
    ("组织建设", [4, 2, 0, 0]),
    ("国际主义", [4, 2, 3, 0]),
    ("理想社会", [4, 3, 0, 0]),
    ("生产活动", [4, 3, 2, 0]),
    ("乌托邦", [4, 4, 0, 0]),
    ("去人类中心", [4, 4, 2, 0]),

    // ── 跨域术语 ──
    ("康德", [3, 2, 1, 0]),
    ("黑格尔", [3, 2, 4, 0]),
    ("海德格尔", [3, 2, 3, 0]),
    ("胡塞尔", [3, 1, 1, 0]),
    ("马克思", [2, 4, 4, 2]),
    ("毛泽东", [4, 1, 3, 0]),
    ("萨特", [3, 3, 1, 2]),
    ("维特根斯坦", [3, 4, 4, 3]),
    ("拉康", [3, 4, 4, 4]),
    ("德勒兹", [3, 4, 3, 0]),
    ("福柯", [3, 4, 2, 0]),
    ("葛兰西", [4, 1, 3, 4]),
    ("柏拉图", [2, 1, 2, 0]),
    ("亚里士多德", [2, 1, 2, 3]),
    ("苏格拉底", [2, 1, 2, 1]),
    ("笛卡尔", [2, 3, 2, 0]),
    ("斯宾诺莎", [2, 3, 1, 0]),
    ("莱布尼茨", [2, 3, 1, 0]),
    ("叔本华", [1, 3, 3, 4]),
    ("柏格森", [1, 3, 4, 0]),
    ("巴门尼德", [2, 1, 2, 0]),
    ("赫拉克利特", [2, 1, 1, 4]),
    ("毕达哥拉斯", [2, 1, 3, 0]),
    ("阿多诺", [3, 2, 4, 2]),
    ("齐泽克", [3, 4, 4, 4]),
];

/// 从任务文本推断 ismism 坐标
fn infer_task_ismism(task: &str) -> TaskIsmismProfile {
    let task_lower = task.to_lowercase();
    let mut field_hits = [0u32; 4];
    let mut weighted_coords = [0f64; 4];
    let mut total_weight = 0f64;

    for &(term, coords) in ISMISM_TERMS {
        if task_lower.contains(&term.to_lowercase()) {
            let idx = coords[0] as usize - 1; // field 1→index 0, etc.
            field_hits[idx] += 1;
            // Weight: each matched dimension contributes, dim-1 (field) = ×2 weight
            let term_weight = if coords[3] != 0 { 2.0 } else if coords[2] != 0 { 1.5 } else { 1.0 };
            for d in 0..4 {
                if coords[d] != 0 {
                    weighted_coords[d] += coords[d] as f64 * term_weight;
                }
            }
            total_weight += term_weight;
        }
    }

    let total_hits: u32 = field_hits.iter().sum();
    let field_weights: [f64; 4] = if total_hits > 0 {
        [
            field_hits[0] as f64 / total_hits as f64,
            field_hits[1] as f64 / total_hits as f64,
            field_hits[2] as f64 / total_hits as f64,
            field_hits[3] as f64 / total_hits as f64,
        ]
    } else {
        [0.25, 0.25, 0.25, 0.25]
    };

    let inferred = if total_weight > 0.0 {
        Some([
            (weighted_coords[0] / total_weight).round().clamp(1.0, 4.0) as u8,
            (weighted_coords[1] / total_weight).round().clamp(0.0, 4.0) as u8,
            (weighted_coords[2] / total_weight).round().clamp(0.0, 4.0) as u8,
            (weighted_coords[3] / total_weight).round().clamp(0.0, 4.0) as u8,
        ])
    } else {
        None
    };

    tracing::debug!(
        "Ismism inference: task=\"{:.80}\" fields={:?} inferred={:?}",
        task, field_weights, inferred
    );

    TaskIsmismProfile { field_weights, inferred }
}

/// 解析 ismism code（如 "1-2-3-4"）为 [u8; 4]
fn parse_ismism(code: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = code.split('-').collect();
    if parts.len() < 4 { return None; }
    let nums: Vec<u8> = parts.iter().filter_map(|p| p.parse().ok()).collect();
    (nums.len() >= 4).then(|| [nums[0], nums[1], nums[2], nums[3]])
}

/// 两个字符串的 trigram Jaccard 重叠度，用于缺失视角→候选魂的轻量匹配
fn jaccard_trigram_overlap(a: &str, b: &str) -> f64 {
    fn trigrams(s: &str) -> std::collections::HashSet<[char; 3]> {
        let chars: Vec<char> = s.chars().collect();
        chars.windows(3).filter_map(|w| <[char; 3]>::try_from(w).ok()).collect()
    }
    let ta = trigrams(a);
    let tb = trigrams(b);
    if ta.is_empty() || tb.is_empty() { return 0.0; }
    let intersection = ta.intersection(&tb).count();
    let union = ta.union(&tb).count();
    intersection as f64 / union as f64
}

/// 计算魂的 ismism 坐标与任务推断坐标的邻近度 (0.0 ~ 1.0)
fn ismism_proximity_score(profile: &TaskIsmismProfile, soul: &SoulListEntry) -> f64 {
    let soul_code = match parse_ismism(&soul.ismism_code) {
        Some(c) => c,
        None => return 0.0,
    };

    // 在场域权重：任务所在的 field 越高，同 field 的魂得分越高
    let soul_field_idx = soul_code[0].saturating_sub(1) as usize;
    if soul_field_idx >= 4 { return 0.0; }
    let field_weight = profile.field_weights[soul_field_idx];

    // 如果有精确的坐标推断，计算各维度邻近度
    let coord_prox = if let Some(inferred) = profile.inferred {
        let mut sim = 0.0;
        // Field dim (×2 weight — most important)
        let f_diff = (soul_code[0] as f64 - inferred[0] as f64).abs();
        sim += (1.0 - f_diff / 4.0) * 0.40;
        // 其余三维
        for d in 1..4 {
            if inferred[d] != 0 && soul_code[d] != 0 {
                let diff = (soul_code[d] as f64 - inferred[d] as f64).abs();
                sim += (1.0 - diff / 4.0) * 0.20;
            }
        }
        sim
    } else {
        // 无精确推断，纯 field 权重
        field_weight
    };

    // Mix: 60% field weight + 40% coordinate proximity
    field_weight * 0.6 + coord_prox * 0.4
}

/// 统计任务文本命中魂的领域术语的次数
fn count_domain_term_hits(task_lower: &str, soul: &SoulListEntry) -> usize {
    let mut hits = 0;
    for domain in &soul.domains {
        for word in domain.split(&['/', '、', ',', '，'][..]) {
            let w = word.trim();
            if w.len() >= 2 && task_lower.contains(&w.to_lowercase()) {
                hits += 1;
                break;
            }
        }
    }
    // Also check field name
    for word in soul.field.split(&['/', '、', '.', '·'][..]) {
        let w = word.trim();
        if w.len() >= 2 && task_lower.contains(&w.to_lowercase()) {
            hits += 1;
            break;
        }
    }
    hits
}

async fn analyze_task(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AnalyzeRequest>,
) -> Sse<UnboundedReceiverStream<Result<Event, std::convert::Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, std::convert::Infallible>>();

    tokio::spawn(async move {
        let send = |data: String| {
            let _ = tx.send(Ok(Event::default().data(data)));
        };

        let trunc_len = body.task.len().min(50);
        let safe_idx = (0..=trunc_len).rev().find(|&i| body.task.is_char_boundary(i)).unwrap_or(0);
        tracing::info!("Starting task analysis for: {}", &body.task[..safe_idx]);

        // Entry classification
        let has_specific = body.task.contains("我") && (body.task.contains("做了") || body.task.contains("正在") || body.task.contains("经历过") || body.task.contains("我公司") || body.task.contains("我的项目") || body.task.contains("我的工厂"));
        let has_first_person = body.task.starts_with("我");
        let has_concrete = !body.task.contains("通常") && !body.task.contains("一般") && (body.task.contains("上次") || body.task.contains("最近") || body.task.contains("今天") || body.task.contains("昨天") || body.task.contains("这周"));
        let score = [has_specific, has_first_person, has_concrete].iter().filter(|&&x| x).count();

        if score >= 2 {
            send(serde_json::json!({
                "phase": "practice_opening"
            }).to_string());
            send(serde_json::json!({
                "phase": "done",
                "response": {
                    "entry_type": "practice_opening",
                    "matched_souls": serde_json::json!([]),
                    "recommended_mode": "practice_opening",
                    "review": { "verdict": "practice_opening", "checks": [], "notes": "", "reviewer": "" },
                    "task_cards": {}
                }
            }).to_string());
            return;
        }

        let entry_type = if score == 1 { "hybrid" } else { "conventional" };

        let provider = match pick_provider(&state) {
            Ok(p) => p,
            Err(_) => {
                send(serde_json::json!({ "phase": "error", "message": "无可用的 AI provider" }).to_string());
                send(serde_json::json!({
                    "phase": "done",
                    "response": {
                        "entry_type": "conventional",
                        "matched_souls": [],
                        "recommended_mode": "single",
                        "review": { "verdict": "pass", "checks": [], "notes": "No LLM provider available", "reviewer": "" },
                        "task_cards": {}
                    }
                }).to_string());
                return;
            }
        };

        let all_souls = match state.registry.list_souls(&foundation::IsmismFilter::default()) {
            Ok(s) => s,
            Err(_) => {
                send(serde_json::json!({ "phase": "error", "message": "魂列表加载失败" }).to_string());
                send(serde_json::json!({
                    "phase": "done",
                    "response": {
                        "entry_type": "conventional",
                        "matched_souls": [],
                        "recommended_mode": "single",
                        "review": { "verdict": "pass", "checks": [], "notes": "Failed to list souls", "reviewer": "" },
                        "task_cards": {}
                    }
                }).to_string());
                return;
            }
        };
        let task_lower = body.task.to_lowercase();

        // ── Phase: classifying ──
        send(serde_json::json!({ "phase": "classifying", "entry_type": entry_type }).to_string());

        // ── Step 1: Algorithmic matching ──
        // 1a. FT semantic search
        let ft_results = state.registry.search_souls(&body.task).unwrap_or_default();
        let ft_scores: std::collections::HashMap<String, f64> = ft_results.iter()
            .map(|m| (m.entry.name.clone(), m.relevance))
            .collect();

        // 1b. Task → Ismism inference: scan for known philosophical terms
        let task_ismism = infer_task_ismism(&body.task);

        // 1c. Multi-factor composite scoring
        let mut scored: Vec<(&SoulListEntry, f64, &str)> = all_souls.iter().map(|s| {
            let kw_hits = s.trigger_keywords.iter()
                .filter(|kw| task_lower.contains(&kw.to_lowercase()))
                .count();
            let ft_score = ft_scores.get(&s.name).copied().unwrap_or(0.0);
            let ismism_prox = ismism_proximity_score(&task_ismism, s);
            let domain_hits = count_domain_term_hits(&task_lower, s);

            // Composite: FT(35%) + Ismism(30%) + Keywords(20%) + Domain(15%)
            let composite = ft_score * 0.35
                + ismism_prox * 0.30
                + (kw_hits as f64).min(3.0) / 3.0 * 0.20
                + (domain_hits as f64).min(3.0) / 3.0 * 0.15;

            (s, composite, if kw_hits > 0 { "keyword" } else if ismism_prox > 0.5 { "ismism" } else if ft_score > 0.3 { "semantic" } else { "fallback" })
        }).collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.0.summon_count.cmp(&a.0.summon_count)));

        // Dynamic top-N selection: stop at relevance floor + elbow + hard cap
        let relevance_floor = 0.05; // ignore souls with near-zero composite
        let hard_cap = 5usize;      // never match more than 5 souls, target ~4
        let mut take_n = 0usize;
        let mut prev_score = f64::MAX;
        for (i, (_, score, _)) in scored.iter().enumerate() {
            if *score < relevance_floor { break; }
            // Elbow detection: if score drops >50% from previous, stop here
            if i > 0 && i >= 2 && prev_score > 0.0 && (*score / prev_score) < 0.5 {
                break;
            }
            if i >= hard_cap { break; }
            take_n = i + 1;
            prev_score = *score;
        }
        // Floor: at least 1 soul if any scored above relevance floor
        take_n = take_n.max(if scored.first().map(|s| s.1).unwrap_or(0.0) >= relevance_floor { 1 } else { 1 })
                       .min(scored.len());

        let souls: Vec<SoulMatch> = scored.iter().take(take_n).map(|(s, composite, match_type)| {
            let kw_hits = s.trigger_keywords.iter()
                .filter(|kw| task_lower.contains(&kw.to_lowercase()))
                .count();
            let ismism_prox = ismism_proximity_score(&task_ismism, s);
            let rationale = match *match_type {
                "keyword" => format!("命中 {} 个关键词 | 坐标邻近度 {:.0}% | 综合分 {:.3}", kw_hits, ismism_prox*100.0, composite),
                "ismism" => format!("坐标邻近度 {:.0}% | 综合分 {:.3}", ismism_prox*100.0, composite),
                "semantic" => format!("全文相关性 | 坐标邻近度 {:.0}% | 综合分 {:.3}", ismism_prox*100.0, composite),
                _ => format!("综合相关性 | 坐标邻近度 {:.0}% | 综合分 {:.3}", ismism_prox*100.0, composite),
            };
            SoulMatch {
                name: s.name.clone(),
                field: s.field.clone(),
                ismism_code: s.ismism_code.clone(),
                rationale,
            }
        }).collect();

        // ── Ismism-based mode determination ──

        // Calculate Ismism diversity score (0.0 = identical, 1.0 = maximum diversity)
        fn calc_ismism_diversity(codes: &[[u8; 4]]) -> (f64, usize, usize) {
            if codes.len() < 2 {
                return (0.0, 0, 0);
            }

            let mut total_field_diff = 0.0;
            let mut total_ontology_diff = 0.0;
            let mut total_epistemology_diff = 0.0;
            let mut total_teleology_diff = 0.0;
            let comparisons = codes.len() * (codes.len() - 1) / 2;
            let mut high_field_count = 0;
            let mut high_ontology_count = 0;
            let mut high_epistemology_count = 0;
            let mut high_teleology_count = 0;

            for i in 0..codes.len() {
                for j in (i+1)..codes.len() {
                    let diff0 = codes[i][0].abs_diff(codes[j][0]) as f64 / 3.0;
                    let diff1 = codes[i][1].abs_diff(codes[j][1]) as f64 / 3.0;
                    let diff2 = codes[i][2].abs_diff(codes[j][2]) as f64 / 3.0;
                    let diff3 = codes[i][3].abs_diff(codes[j][3]) as f64 / 3.0;
                    total_field_diff += diff0;
                    total_ontology_diff += diff1;
                    total_epistemology_diff += diff2;
                    total_teleology_diff += diff3;
                    if diff0 >= 0.4 { high_field_count += 1; }
                    if diff1 >= 0.4 { high_ontology_count += 1; }
                    if diff2 >= 0.4 { high_epistemology_count += 1; }
                    if diff3 >= 0.4 { high_teleology_count += 1; }
                }
            }

            let avg_field = total_field_diff / comparisons as f64;
            let _avg_ontology = total_ontology_diff / comparisons as f64;
            let _avg_epistemology = total_epistemology_diff / comparisons as f64;
            let _avg_teleology = total_teleology_diff / comparisons as f64;

            // Weighted diversity: teleology > epistemology > ontology > field
            let diversity = avg_field * 0.15 + _avg_ontology * 0.20 + _avg_epistemology * 0.25 + _avg_teleology * 0.40;

            // Count how many dimensions have high diversity
            let high_dimensions = [high_field_count, high_ontology_count, high_epistemology_count, high_teleology_count]
                .iter().filter(|&&c| c >= comparisons / 2).count();

            (diversity, high_dimensions, comparisons)
        }

        let mode = if souls.len() <= 1 {
            "single".to_string()
        } else {
            // Parse ismism codes from matched souls
            let ismism_codes: Vec<[u8; 4]> = souls.iter()
                .filter_map(|s| parse_ismism(&s.ismism_code))
                .collect();

            let (diversity, high_dims, _comparisons) = calc_ismism_diversity(&ismism_codes);

            tracing::debug!(
                "Ismism mode analysis: souls={} diversity={:.3} high_dims={}/4",
                souls.len(), diversity, high_dims
            );

            // Mode determination based on Ismism diversity:
            // - Single: 1 soul or diversity < 0.2
            // - Conference: diversity 0.2-0.5, 1-2 high-dim
            // - Debate: diversity >= 0.5 OR 3+ dimensions with high divergence
            if diversity < 0.2 || ismism_codes.len() < 2 {
                "single".to_string()
            } else if diversity >= 0.5 || high_dims >= 3 {
                "debate".to_string()
            } else {
                "conference".to_string()
            }
        };

        // ── Phase: matched ──
        send(serde_json::json!({
            "phase": "matched",
            "souls": souls.iter().map(|s| serde_json::json!({
                "name": s.name, "field": s.field, "ismism_code": s.ismism_code, "rationale": s.rationale,
            })).collect::<Vec<_>>(),
            "mode": mode,
        }).to_string());

        // ── Step 2: Banner lord review ──
        let reviewer_name = body.reviewer.clone()
            .unwrap_or_else(|| std::env::var("AIONUI_REVIEWER_SOUL").unwrap_or_else(|_| "未明子".to_string()));
        let mut task_cards: std::collections::HashMap<String, String> = Default::default();

        // ── Phase: reviewing (stream before waiting for LLM) ──
        send(serde_json::json!({
            "phase": "reviewing",
            "reviewer": reviewer_name,
        }).to_string());

        let (verdict, checks, notes, verified_souls, missing_perspectives) = if let Ok(banner_lord) = state.registry.get_soul(&reviewer_name) {
            let candidate_profiles: Vec<SoulProfile> = souls.iter()
                .filter_map(|s| state.registry.get_soul(&s.name).ok())
                .collect();

            let review_handle = spawn_banner_lord_review(
                state.engine.gateway().clone(),
                banner_lord,
                body.task.clone(),
                candidate_profiles,
                body.judgment.clone().unwrap_or_default(),
                body.worry.clone().unwrap_or_default(),
                body.unknown.clone().unwrap_or_default(),
            );

            let result = match review_handle.await {
                Ok(Ok(json)) => json,
                Ok(Err(e)) => {
                    tracing::error!("Banner lord review sub-agent failed: {}", e);
                    serde_json::Value::default()
                }
                Err(e) => {
                    tracing::error!("Banner lord review sub-agent panicked: {}", e);
                    serde_json::Value::default()
                }
            };

            if let Some(cards) = result["task_cards"].as_object() {
                for (k, v) in cards {
                    if let Some(val) = v.as_str() {
                        task_cards.insert(k.clone(), val.to_string());
                    }
                }
            }
            let missing_perspectives: Vec<String> = result["missing_perspectives"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let boundary_risks: Vec<String> = result["boundary_risks"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let verified_souls: Vec<String> = result["verified_souls"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let v = result["verdict"].as_str().unwrap_or("pass").to_string();
            let c: Vec<String> = result["checks"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let n = if !missing_perspectives.is_empty() {
                format!("{} | 缺失视角：{} | 边界风险：{}",
                    result["notes"].as_str().unwrap_or(""),
                    missing_perspectives.join("、"),
                    boundary_risks.join("、"))
            } else {
                result["notes"].as_str().unwrap_or("").to_string()
            };
            (v, c, n, verified_souls, missing_perspectives)
        } else {
            ("pass".to_string(), vec!["审查官不可用，默认通过".into()], String::new(), vec![], vec![])
        };

        // ── Phase: review_done ──
        send(serde_json::json!({
            "phase": "review_done",
            "review": {
                "verdict": verdict,
                "checks": checks,
                "notes": notes,
                "reviewer": reviewer_name,
            }
        }).to_string());

        // ── Step 3: 根据审查反馈重新匹配，再最终裁决 ──
        // 审查官先对算法结果做初裁（谁 pass/reject，缺什么视角），
        // 系统根据缺失视角补充候选魂，审查官再做终裁。
        let (final_verdict, final_checks, final_notes, final_verified_souls) =
            if !missing_perspectives.is_empty() {
                // 审查官发现了缺失视角 → 搜索补充魂
                let rejected_names: Vec<&str> = souls.iter()
                    .map(|s| s.name.as_str())
                    .filter(|n| !verified_souls.iter().any(|v| v == *n))
                    .collect();

                let mut complementary: Vec<SoulMatch> = Vec::new();
                for perspective in &missing_perspectives {
                    let perspective_lower = perspective.to_lowercase();
                    for entry in &all_souls {
                        // 跳过已在 verified_souls 或 rejected 中的魂
                        if verified_souls.iter().any(|v| v == &entry.name) { continue; }
                        if rejected_names.contains(&entry.name.as_str()) { continue; }
                        if complementary.iter().any(|s| s.name == entry.name) { continue; }

                        // 用关键词匹配判断魂是否覆盖该视角
                        let search_text = format!("{} {} {} {}",
                            entry.field, entry.ismism_code,
                            entry.self_declare,
                            entry.tags.join(" ")
                        ).to_lowercase();

                        // 简单 trigram 重叠匹配
                        let overlap = jaccard_trigram_overlap(&perspective_lower, &search_text);
                        if overlap > 0.08 {
                            let ismism_prox = ismism_proximity_score(&task_ismism, entry);
                            complementary.push(SoulMatch {
                                name: entry.name.clone(),
                                field: entry.field.clone(),
                                ismism_code: entry.ismism_code.clone(),
                                rationale: format!("补充视角「{}」| trigram重叠 {:.0}% | 坐标邻近度 {:.0}%",
                                    perspective, overlap * 100.0, ismism_prox * 100.0),
                            });
                        }
                    }
                }
                complementary.sort_by(|a, b| {
                    b.rationale.len().partial_cmp(&a.rationale.len()).unwrap_or(std::cmp::Ordering::Equal)
                });
                complementary.truncate(5);

                if !complementary.is_empty() {
                    // ── 二轮审查：将补充魂加入候选，审查官终裁 ──
                    send(serde_json::json!({
                        "phase": "rematching",
                        "missing": missing_perspectives,
                        "complementary": complementary.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
                    }).to_string());

                    let mut final_candidates: Vec<SoulProfile> = verified_souls.iter()
                        .filter_map(|name| state.registry.get_soul(name).ok())
                        .collect();
                    for s in &complementary {
                        if let Ok(profile) = state.registry.get_soul(&s.name) {
                            final_candidates.push(profile);
                        }
                    }

                    if let Ok(banner_lord2) = state.registry.get_soul(&reviewer_name) {
                        let r2_handle = spawn_banner_lord_review(
                            state.engine.gateway().clone(),
                            banner_lord2,
                            body.task.clone(),
                            final_candidates,
                            body.judgment.clone().unwrap_or_default(),
                            body.worry.clone().unwrap_or_default(),
                            body.unknown.clone().unwrap_or_default(),
                        );
                        match r2_handle.await {
                            Ok(Ok(ref result2)) => {
                                let v2 = result2["verdict"].as_str().unwrap_or("pass").to_string();
                                let vs2: Vec<String> = result2["verified_souls"].as_array()
                                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                    .unwrap_or_default();
                                let c2: Vec<String> = result2["checks"].as_array()
                                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                    .unwrap_or_default();
                                let n2 = result2["notes"].as_str().unwrap_or("").to_string();
                                if let Some(cards) = result2["task_cards"].as_object() {
                                    for (k, v) in cards {
                                        if let Some(val) = v.as_str() {
                                            task_cards.insert(k.clone(), val.to_string());
                                        }
                                    }
                                }
                                tracing::info!("Round 2 review complete: verdict={}, verified_souls={:?}", v2, vs2);
                                (v2, c2, n2, vs2)
                            }
                            _ => {
                                tracing::warn!("Round 2 review failed, using round 1 results");
                                (verdict.clone(), checks.clone(), notes.clone(), verified_souls.clone())
                            }
                        }
                    } else {
                        (verdict.clone(), checks.clone(), notes.clone(), verified_souls.clone())
                    }
                } else {
                    (verdict.clone(), checks.clone(), notes.clone(), verified_souls.clone())
                }
            } else {
                (verdict.clone(), checks.clone(), notes.clone(), verified_souls.clone())
            };

        // ── Step 4: 以审查官终裁的 verified_souls 为准 ──
        // 审查官是唯一权威。算法匹配只是参考，终裁的 verified_souls 决定最终阵容。
        let final_souls = if !final_verified_souls.is_empty() {
            let chosen: Vec<SoulMatch> = final_verified_souls.iter()
                .filter_map(|name| {
                    // 先在算法匹配结果中找
                    if let Some(s) = souls.iter().find(|s| s.name == *name).cloned() {
                        return Some(s);
                    }
                    // 算法没匹配到但审查官指定了——从 registry 补齐
                    all_souls.iter().find(|e| e.name == *name).map(|entry| {
                        let ismism_prox = ismism_proximity_score(&task_ismism, entry);
                        SoulMatch {
                            name: entry.name.clone(),
                            field: entry.field.clone(),
                            ismism_code: entry.ismism_code.clone(),
                            rationale: format!("审查官指定 | 领域 {} | 坐标邻近度 {:.0}%", entry.field, ismism_prox * 100.0),
                        }
                    })
                })
                .collect();

            if chosen.is_empty() {
                tracing::warn!("final_verified_souls={:?} not found in registry or matched, keeping all", final_verified_souls);
                souls
            } else {
                tracing::info!(
                    "Banner lord final authority: final_verified_souls={:?}, final {} souls (algorithm matched {})",
                    final_verified_souls, chosen.len(), souls.len()
                );
                chosen
            }
        } else if final_verdict == "pass" {
            tracing::info!("No verified_souls from review, pass → using all {} algorithm-matched souls", souls.len());
            souls
        } else {
            // ── Phase: adjusting ──
            send(serde_json::json!({ "phase": "adjusting" }).to_string());

            let adjustment_prompt = format!(
                "## 任务\n{}\n\n## 当前魂组合\n{}\n\n## 幡主审查结果\n裁决：{}\n{}\n备注：{}\n\n## 指令\n根据审查结果调整魂组合。如果是条件通过——增删魂以满足约束。如果是拒绝——完全重新匹配。\n返回JSON：{{\"souls\":[{{\"name\":\"魂名\",\"rationale\":\"调整理由\"}}]}}",
                body.task,
                souls.iter().map(|s| format!("- {} [{}] {}", s.name, s.field, s.rationale)).collect::<Vec<_>>().join("\n"),
                final_verdict, final_checks.join("\n"), final_notes
            );
            let provider3 = provider.clone();
            match llm_call_json(&state, provider3, None, &adjustment_prompt, 0.3, 4096, None).await {
                Ok(adjusted) => {
                    let adjusted_souls = adjusted["souls"].as_array()
                        .map(|arr| arr.iter().filter_map(|s| build_soul_match(s, &all_souls)).collect())
                        .unwrap_or(souls.clone());
                    if adjusted_souls.iter().any(|s| !souls.iter().any(|orig| orig.name == s.name))
                        || souls.iter().any(|orig| !adjusted_souls.iter().any(|s| s.name == orig.name))
                    {
                        if !adjusted_souls.is_empty() {
                            let adjusted_profiles: Vec<SoulProfile> = adjusted_souls.iter()
                                .filter_map(|s| state.registry.get_soul(&s.name).ok())
                                .collect();
                            if let Ok(banner_lord) = state.registry.get_soul(&reviewer_name) {
                                let rs = format!(
                                    "{}你是{}，ismism坐标{}。你作为幡主审查官，为调整后的魂组合分配差异化子问题。",
                                    banner_lord.summon_prompt, banner_lord.name, banner_lord.ismism_code
                                );
                                let p_info: Vec<String> = adjusted_profiles.iter().map(|p| {
                                    format!("- {} [{}] ismism={}", p.name, p.field, p.ismism_code)
                                }).collect();
                                let ru = format!(
                                    "## 总任务\n{}\n\n## 调整后的魂组合\n{}\n\n为每个魂分配一个只有他能回答好的子问题。\n返回JSON：{{\"task_cards\":{{\"魂名\":\"专属子问题\"}}}}",
                                    body.task, p_info.join("\n")
                                );
                                if let Ok(tc) = llm_call_json(&state, provider.clone(), Some(&rs), &ru, 0.3, 4096, None).await {
                                    if let Some(cards) = tc["task_cards"].as_object() {
                                        task_cards.clear();
                                        for (k, v) in cards {
                                            if let Some(val) = v.as_str() {
                                                task_cards.insert(k.clone(), val.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        adjusted_souls
                    } else {
                        souls
                    }
                }
                Err(_) => souls,
            }
        };

        // ── Phase: done ──
        send(serde_json::json!({
            "phase": "done",
            "response": {
                "entry_type": entry_type,
                "matched_souls": final_souls.iter().map(|s| serde_json::json!({
                    "name": s.name, "field": s.field, "ismism_code": s.ismism_code, "rationale": s.rationale,
                })).collect::<Vec<_>>(),
                "recommended_mode": mode,
                "review": { "verdict": final_verdict, "checks": final_checks, "notes": final_notes, "reviewer": reviewer_name },
                "task_cards": task_cards,
            }
        }).to_string());
    });

    Sse::new(UnboundedReceiverStream::new(rx)).keep_alive(KeepAlive::default())
}

fn spawn_follow_up_agent(
    gateway: Arc<ai_gateway::GatewayRegistry>,
    banner_lord: SoulProfile,
    question: String,
    history: Vec<String>,
    ws: possession::WsSessionManager,
    session_id: String,
    archive: Arc<archive::ArchiveSystem>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let provider = gateway
            .list_providers()
            .into_iter()
            .find(|i| i.available)
            .map(|i| i.provider)
            .unwrap_or(foundation::Provider::Claude);

        let system_prompt = banner_lord.summon_prompt;
        let user_prompt = if history.is_empty() {
            format!(
                "## 新问题\n{}\n\n根据这个新问题，以你的立场和视角回应。你是{}，ismism坐标{}。",
                question, banner_lord.name, banner_lord.ismism_code
            )
        } else {
            format!(
                "## 历史对话\n{}\n\n## 新问题\n{}\n\n以上是之前的多魂附体会话。作为{}（ismism={}），请以你的立场和视角回应这个追问。",
                history.join("\n\n"), question, banner_lord.name, banner_lord.ismism_code
            )
        };

        let q_msg = foundation::Message {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.clone(),
            role: foundation::MessageRole::User,
            soul_name: None,
            content: question.clone(),
            seq: 990,
            created_at: chrono::Utc::now(),
        };
        let _ = archive.append_message(&q_msg).await;

        let config = CallConfig {
            temperature: 0.7,
            max_tokens: 8192,
            stream: true,
            model: None,
            reasoning_effort: Some(foundation::ReasoningEffort::Think),
            structured_output: None,
            thinking_enabled: None,
            tools: None,
            tool_choice: None,
        };

        let req = LLMRequest {
            provider,
            prompt: Prompt {
                messages: vec![
                    PromptMessage {
                        role: "system".into(),
                        content: system_prompt,
                        reasoning_content: None,
                        tool_call_id: None,
                        tool_calls: None,
                    },
                    PromptMessage {
                        role: "user".into(),
                        content: user_prompt,
                        reasoning_content: None,
                        tool_call_id: None,
                        tool_calls: None,
                    },
                ],
            },
            config,
        };

        match gateway.call(&req) {
            Ok(rx) => {
                match possession::stream::stream_synthesis(rx, &session_id, &ws).await {
                    Ok((content, _)) => {
                        let msg = foundation::Message {
                            id: uuid::Uuid::new_v4().to_string(),
                            session_id,
                            role: foundation::MessageRole::Synthesis,
                            soul_name: Some(banner_lord.name),
                            content,
                            seq: 991,
                            created_at: chrono::Utc::now(),
                        };
                        let _ = archive.append_message(&msg).await;
                    }
                    Err(e) => {
                        tracing::error!("Follow-up agent stream error: {}", e);
                        let _ = ws.broadcast_system(
                            &session_id,
                            &possession::WsEvent {
                                event_type: possession::WsEventType::Error,
                                payload: format!("流式响应错误: {}", e),
                                reasoning_content: None,
                                soul_name: None,
                                seq: 0,
                            },
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!("Follow-up agent LLM call failed: {}", e);
                let _ = ws.broadcast_system(
                    &session_id,
                    &possession::WsEvent {
                        event_type: possession::WsEventType::Error,
                        payload: format!("启动 LLM 失败: {}", e),
                        reasoning_content: None,
                        soul_name: None,
                        seq: 0,
                    },
                );
            }
        }
    })
}

// ── Follow-up ──

#[derive(Debug, Deserialize)]
struct FollowUpRequest { question: String }

async fn follow_up(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(body): Json<FollowUpRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ApiError>)> {
    tracing::info!("Received follow-up request for session: {}", session_id);

    let ws = state.engine.ws_manager().clone();
    let sid = session_id.clone();
    ws.create_session(&sid);

    let history: Vec<String> = match state.archive.get_session_detail(&session_id).await {
        Ok(session) => {
            session.messages.iter().map(|m| format!("[{:?}] {}: {}", m.role, m.soul_name.as_deref().unwrap_or("系统"), m.content)).collect()
        }
        Err(e) => {
            tracing::warn!("Session not found: {}, proceeding with basic prompt", e);
            vec![]
        }
    };

    let reviewer_name = std::env::var("AIONUI_REVIEWER_SOUL").unwrap_or_else(|_| "未明子".to_string());
    let banner_lord = state.registry.get_soul(&reviewer_name).map_err(|e| {
        tracing::error!("Reviewer soul not found: {}", e);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: format!("审查官 {} 不可用", reviewer_name) }))
    })?;

    let question = body.question.clone();
    let gateway = state.engine.gateway().clone();
    let archive = state.archive.clone();

    let _ = spawn_follow_up_agent(
        gateway,
        banner_lord,
        question,
        history,
        ws,
        sid,
        archive,
    );

    tracing::info!("Follow-up sub-agent spawned, returning immediately");
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── Status ──

#[derive(Debug, Serialize)]
struct PossessionStatusResponse { session_id: String, connected: bool }

async fn possession_status(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Json<PossessionStatusResponse> {
    Json(PossessionStatusResponse { session_id: session_id.clone(), connected: state.engine.ws_manager().has_session(&session_id) })
}

// ── OCR ──

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
const MAX_FILES: usize = 5;
const ALLOWED_MIMES: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif"];

#[derive(Debug, Serialize)]
struct OcrResultItem {
    filename: String,
    text: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct OcrUploadResponse {
    results: Vec<OcrResultItem>,
}

async fn ocr_upload(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<OcrUploadResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let mut file_data: Vec<(String, Vec<u8>)> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let Some(filename) = field.file_name().map(String::from) else { continue; };
        let Some(content_type) = field.content_type().map(String::from) else { continue; };

        if !ALLOWED_MIMES.contains(&content_type.as_str()) {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                Json(ApiError { error: format!("不支持的文件类型: {}", content_type) }),
            ));
        }

        let Ok(data) = field.bytes().await else { continue; };
        if data.len() > MAX_FILE_SIZE {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                Json(ApiError { error: format!("文件 {} 超过 10MB 限制", filename) }),
            ));
        }

        file_data.push((filename, data.to_vec()));
        if file_data.len() >= MAX_FILES { break; }
    }

    if file_data.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ApiError { error: "未收到有效文件".into() }),
        ));
    }

    let mut handles = Vec::new();
    for (filename, data) in file_data {
        let handle = tokio::task::spawn_blocking(move || {
            let text = ocr::ocr_image(&data, "chi_sim+eng");
            (filename, text)
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok((filename, Ok(text))) => {
                let filtered = text.trim();
                results.push(OcrResultItem {
                    filename,
                    text: if filtered.is_empty() { None } else { Some(filtered.to_string()) },
                    error: None,
                });
            }
            Ok((filename, Err(e))) => {
                results.push(OcrResultItem { filename, text: None, error: Some(e) });
            }
            Err(e) => {
                results.push(OcrResultItem {
                    filename: "unknown".into(),
                    text: None,
                    error: Some(format!("spawn_blocking 错误: {}", e)),
                });
            }
        }
    }

    Ok(Json(OcrUploadResponse { results }))
}
