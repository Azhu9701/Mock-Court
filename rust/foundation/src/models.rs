use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulProfile {
    pub name: String,
    pub ismism_code: String,
    #[serde(default)]
    pub field: String,
    #[serde(default)]
    pub ontology: String,
    #[serde(default)]
    pub epistemology: String,
    #[serde(default)]
    pub teleology: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub exclude_scenarios: Vec<String>,
    #[serde(default)]
    pub summon_count: u32,
    #[serde(default)]
    pub effectiveness: EffectivenessStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub summon_prompt: String,
    #[serde(default)]
    pub practice_observations: Vec<PracticeObservation>,
    // Agent-specific fields from soul-banner format
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub voice: String,
    #[serde(default)]
    pub mind: String,
    #[serde(default)]
    pub self_declare: String,
    #[serde(default)]
    pub skills_expertise: Vec<String>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub tools: String,
    #[serde(default)]
    pub trigger_keywords: Vec<String>,
    #[serde(default)]
    pub compat: Vec<String>,
    #[serde(default)]
    pub incompat: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulFrontmatter {
    pub name: String,
    pub ismism_code: String,
    #[serde(default)]
    pub field: String,
    #[serde(default)]
    pub ontology: String,
    #[serde(default)]
    pub epistemology: String,
    #[serde(default)]
    pub teleology: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub exclude_scenarios: Vec<String>,
    #[serde(default)]
    pub summon_count: u32,
    #[serde(default)]
    pub effectiveness: EffectivenessStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub practice_observations: Vec<PracticeObservation>,
    // Agent fields
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub voice: String,
    #[serde(default)]
    pub mind: String,
    #[serde(default)]
    pub self_declare: String,
    #[serde(default)]
    pub skills_expertise: Vec<String>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub tools: String,
    #[serde(default)]
    pub trigger_keywords: Vec<String>,
    #[serde(default)]
    pub compat: Vec<String>,
    #[serde(default)]
    pub incompat: Vec<String>,
}

impl From<SoulProfile> for SoulFrontmatter {
    fn from(p: SoulProfile) -> Self {
        SoulFrontmatter {
            name: p.name,
            ismism_code: p.ismism_code,
            field: p.field,
            ontology: p.ontology,
            epistemology: p.epistemology,
            teleology: p.teleology,
            domains: p.domains,
            exclude_scenarios: p.exclude_scenarios,
            summon_count: p.summon_count,
            effectiveness: p.effectiveness,
            created_at: p.created_at,
            updated_at: p.updated_at,
            tags: p.tags,
            practice_observations: p.practice_observations,
            title: p.title,
            description: p.description,
            voice: p.voice,
            mind: p.mind,
            self_declare: p.self_declare,
            skills_expertise: p.skills_expertise,
            model: p.model,
            tools: p.tools,
            trigger_keywords: p.trigger_keywords,
            compat: p.compat,
            incompat: p.incompat,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulListEntry {
    pub name: String,
    pub ismism_code: String,
    pub field: String,
    pub tags: Vec<String>,
    pub domains: Vec<String>,
    pub summon_count: u32,
    #[serde(default)]
    pub trigger_keywords: Vec<String>,
    #[serde(default)]
    pub self_declare: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub compat: Vec<String>,
    #[serde(default)]
    pub incompat: Vec<String>,
}

impl From<&SoulProfile> for SoulListEntry {
    fn from(p: &SoulProfile) -> Self {
        SoulListEntry {
            name: p.name.clone(),
            ismism_code: p.ismism_code.clone(),
            field: p.field.clone(),
            tags: p.tags.clone(),
            domains: p.domains.clone(),
            summon_count: p.summon_count,
            trigger_keywords: p.trigger_keywords.clone(),
            self_declare: p.self_declare.clone(),
            model: p.model.clone(),
            compat: p.compat.clone(),
            incompat: p.incompat.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulMatch {
    pub entry: SoulListEntry,
    pub relevance: f64,
    pub matched_fields: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectivenessStats {
    pub effective: u32,
    #[serde(default)]
    pub partial: u32,
    #[serde(default)]
    pub invalid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeObservation {
    pub date: NaiveDate,
    pub observation: String,
    pub revision_type: RevisionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevisionType {
    Confirmed,
    Modified,
    Overturned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub mode: PossessionMode,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub mode: PossessionMode,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub message_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PossessionMode {
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "conference")]
    Conference,
    #[serde(rename = "debate")]
    Debate,
    #[serde(rename = "relay")]
    Relay,
    #[serde(rename = "learn")]
    Learn,
    #[serde(rename = "practice_opening")]
    PracticeOpening,
}

impl PossessionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PossessionMode::Single => "single",
            PossessionMode::Conference => "conference",
            PossessionMode::Debate => "debate",
            PossessionMode::Relay => "relay",
            PossessionMode::Learn => "learn",
            PossessionMode::PracticeOpening => "practice_opening",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "single" => Some(PossessionMode::Single),
            "conference" => Some(PossessionMode::Conference),
            "debate" => Some(PossessionMode::Debate),
            "relay" => Some(PossessionMode::Relay),
            "learn" => Some(PossessionMode::Learn),
            "practice_opening" => Some(PossessionMode::PracticeOpening),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "archived")]
    Archived,
    #[serde(rename = "inconsistent")]
    Inconsistent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub soul_name: Option<String>,
    pub content: String,
    pub seq: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "soul")]
    Soul,
    #[serde(rename = "synthesis")]
    Synthesis,
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecord {
    pub id: String,
    pub session_id: String,
    pub soul_name: String,
    pub mode: PossessionMode,
    pub task_summary: String,
    pub effectiveness: Effectiveness,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub self_negation: Option<String>,
    #[serde(default)]
    pub empty_chair: Option<String>,
    #[serde(default)]
    pub user_feedback: Option<String>,
    #[serde(default)]
    pub usage: UsageStats,
}

/// 模型能力等级 — 决定 prompt 注入量
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelTier {
    /// Haiku / DeepSeek小模型 → 完整 methodology + 坐标解释
    Economy,
    /// Sonnet / GPT-4o → 坐标锚定 + 关键约束
    Pro,
    /// Opus → 仅坐标锚定
    Max,
}

impl ModelTier {
    pub fn for_provider(provider: &Provider, model: &str) -> Self {
        match provider {
            Provider::Claude => match model {
                m if m.contains("opus") => ModelTier::Max,
                m if m.contains("haiku") => ModelTier::Economy,
                _ => ModelTier::Pro,
            },
            Provider::OpenAI => match model {
                m if m.contains("gpt-4.5") || m.contains("o1") || m.contains("o3") => ModelTier::Max,
                m if m.contains("mini") || m.contains("3.5") => ModelTier::Economy,
                _ => ModelTier::Pro,
            },
            Provider::DeepSeek => match model {
                m if m.contains("v4") || m.contains("r1") => ModelTier::Pro,
                _ => ModelTier::Economy,
            },
        }
    }
}

/// 魂匹配结果
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub souls: Vec<SoulMatch>,
    pub confidence: f64,
    pub blind_spots: Vec<String>,
    pub constraints: Vec<String>,
    pub field_diversity: f64,
}

/// 失败条件告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAlert {
    pub soul_name: String,
    pub alert_type: FailureAlertType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureAlertType {
    #[serde(rename = "boundary_review")]
    BoundaryReview,
    #[serde(rename = "suspension")]
    Suspension,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effectiveness {
    #[serde(rename = "effective")]
    Effective,
    #[serde(rename = "partial")]
    Partial,
    #[serde(rename = "invalid")]
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub souls: HashMap<String, RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub ismism_code: String,
    pub domains: Vec<String>,
    pub summon_count: u32,
    pub effectiveness: EffectivenessStats,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IsmismStats {
    pub field_distribution: HashMap<u8, usize>,
    pub ontology_distribution: HashMap<u8, usize>,
    pub epistemology_distribution: HashMap<u8, usize>,
    pub teleology_distribution: HashMap<u8, usize>,
    pub total_souls: usize,
}

#[derive(Debug, Clone)]
pub struct IsmismSearch {
    pub target: IsmismCode,
    pub weights: Option<(f64, f64, f64, f64)>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct IsmismFilter {
    pub field: Option<String>,
    pub ontology: Option<String>,
    pub epistemology: Option<String>,
    pub teleology: Option<String>,
    pub nearest: Option<IsmismSearch>,
}

#[derive(Debug, Clone)]
pub struct IsmismCode {
    pub field: u8,
    pub ontology: u8,
    pub epistemology: u8,
    pub teleology: u8,
}

impl IsmismCode {
    pub fn distance(&self, other: &IsmismCode, weights: Option<(f64, f64, f64, f64)>) -> f64 {
        let (wf, wo, we, wt) = weights.unwrap_or((1.0, 1.0, 1.0, 1.0));
        let df = (self.field as f64 - other.field as f64) * wf;
        let od = (self.ontology as f64 - other.ontology as f64) * wo;
        let ed = (self.epistemology as f64 - other.epistemology as f64) * we;
        let td = (self.teleology as f64 - other.teleology as f64) * wt;
        (df * df + od * od + ed * ed + td * td).sqrt()
    }
}

impl TryFrom<&str> for IsmismCode {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid ismism code: {}", s));
        }
        Ok(IsmismCode {
            field: parts[0].trim().parse().map_err(|_| format!("Invalid field: {}", parts[0]))?,
            ontology: parts[1].trim().parse().map_err(|_| format!("Invalid ontology: {}", parts[1]))?,
            epistemology: parts[2].trim().parse().map_err(|_| format!("Invalid epistemology: {}", parts[2]))?,
            teleology: parts[3].trim().parse().map_err(|_| format!("Invalid teleology: {}", parts[3]))?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    pub mode: Option<PossessionMode>,
    pub status: Option<SessionStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct CallFilter {
    pub soul_name: Option<String>,
    pub mode: Option<PossessionMode>,
    pub effectiveness: Option<Effectiveness>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

// ── AI Gateway Types ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Provider {
    Claude,
    OpenAI,
    DeepSeek,
}

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub provider: Provider,
    pub model: String,
    pub available: bool,
    pub tier: ModelTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: FunctionDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// DeepSeek 推理强度
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReasoningEffort {
    /// 不使用深度推理（适合简单任务、Flash模型）
    NonThink,
    /// 标准推理（Pro模型默认）
    Think,
    /// 深度推理（复杂分析）
    ThinkHigh,
    /// 最大深度推理（综合官、自我审计）
    ThinkMax,
}

impl ReasoningEffort {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReasoningEffort::NonThink => "low",
            ReasoningEffort::Think => "medium",
            ReasoningEffort::ThinkHigh => "high",
            ReasoningEffort::ThinkMax => "max",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "non-think" | "nonthink" | "low" | "none" => ReasoningEffort::NonThink,
            "think" | "medium" | "standard" => ReasoningEffort::Think,
            "think-high" | "thinkhigh" | "high" => ReasoningEffort::ThinkHigh,
            "think-max" | "thinkmax" | "max" => ReasoningEffort::ThinkMax,
            _ => ReasoningEffort::Think,
        }
    }
}

/// 结构化输出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredOutputConfig {
    /// 是否启用结构化输出
    pub enabled: bool,
    /// JSON Schema（可选）
    pub json_schema: Option<serde_json::Value>,
}

impl Default for StructuredOutputConfig {
    fn default() -> Self {
        StructuredOutputConfig {
            enabled: false,
            json_schema: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallConfig {
    pub temperature: f64,
    pub max_tokens: u32,
    pub stream: bool,
    /// Optional model override (e.g. "deepseek-chat" for fast matching/review)
    pub model: Option<String>,
    /// 推理强度（DeepSeek v4专用）
    pub reasoning_effort: Option<ReasoningEffort>,
    /// 结构化输出配置
    pub structured_output: Option<StructuredOutputConfig>,
    /// 思考模式开关（DeepSeek v4专用）
    pub thinking_enabled: Option<bool>,
    /// 工具定义
    pub tools: Option<Vec<ToolDefinition>>,
    /// 工具选择策略：auto/none/required 或指定函数名
    pub tool_choice: Option<String>,
}

impl Default for CallConfig {
    fn default() -> Self {
        CallConfig { 
            temperature: 0.7, 
            max_tokens: 8192, 
            stream: true, 
            model: None,
            reasoning_effort: None,
            structured_output: None,
            thinking_enabled: None,
            tools: None,
            tool_choice: None,
        }
    }
}

impl CallConfig {
    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }
    
    pub fn with_structured_output(mut self, schema: serde_json::Value) -> Self {
        self.structured_output = Some(StructuredOutputConfig {
            enabled: true,
            json_schema: Some(schema),
        });
        self
    }

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_tool_choice(mut self, choice: &str) -> Self {
        self.tool_choice = Some(choice.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub content: String,
    pub reasoning_content: Option<String>,
    pub finish_reason: Option<String>,
    pub index: u32,
    pub usage: Option<UsageStats>,
    pub tool_calls: Vec<ToolCall>,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            content: String::new(),
            reasoning_content: None,
            finish_reason: None,
            index: 0,
            usage: None,
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LLMRequest {
    pub provider: Provider,
    pub prompt: Prompt,
    pub config: CallConfig,
}

#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub provider: Provider,
    pub content: String,
    pub usage: UsageStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ── Structured Synthesis Output (§9.5) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisOutput {
    pub consensus: Vec<ConsensusItem>,
    pub divergence: Vec<DivergenceItem>,
    pub blind_spots: Vec<BlindSpotItem>,
    pub principal_contradiction: Contradiction,
    pub action_program: Vec<ActionItem>,
    #[serde(default)]
    pub synthesis_self_audit: Option<SynthesisSelfAudit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SynthesisSelfAudit {
    #[serde(default)]
    pub missing_perspectives: Vec<String>,
    #[serde(default)]
    pub synthesizer_bias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusItem {
    pub point: String,
    pub shared_by: Vec<String>,
    #[serde(default)]
    pub depth: String, // "独立抵达" | "表面共识"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceItem {
    pub axis: String,
    #[serde(default)]
    pub divergence_type: String, // "事实" | "价值" | "前提"
    pub positions: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub soul_name: String,
    pub stance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindSpotItem {
    pub dimension: String,
    pub missing_perspective: String,
    pub coverable_by_existing: bool,
    pub suggested_soul: Option<String>,
    #[serde(default)]
    pub is_structural: bool, // true = 所有参与魂结构性地看不到此维度
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub description: String,
    pub parties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub direction: String,
    pub rationale: String,
    pub priority: u8, // 1-3
    #[serde(default)]
    pub timeline: String, // "立即" | "一周" | "一月" | "长期"
}

// ── 数据库扩展相关模型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulRevision {
    pub id: String,
    pub soul_name: String,
    pub revision_type: RevisionType,
    pub description: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub reviewer: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindSpot {
    pub id: String,
    pub soul_name: String,
    pub dimension: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCard {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source_soul: Option<String>,
    pub source_session: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionProposal {
    pub id: String,
    pub soul_name: String,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub proposed_changes: String,
    pub status: ProposalStatus,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewer: Option<String>,
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalType {
    BoundaryAdjustment,
    OntologyUpdate,
    DomainExpansion,
    SelfDeclareUpdate,
    BlindSpotMitigation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Implemented,
}

#[derive(Debug, Clone, Default)]
pub struct SoulRevisionFilter {
    pub soul_name: Option<String>,
    pub revision_type: Option<RevisionType>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct BlindSpotFilter {
    pub soul_name: Option<String>,
    pub resolved: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeCardFilter {
    pub soul_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeTopic {
    pub session_id: String,
    pub title: String,
    pub mode: String,
    pub created_at: DateTime<Utc>,
    pub soul_names: Vec<String>,
    pub card_summary: Option<String>,
    pub synthesis_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BannerLordDecision {
    pub verified_souls: Vec<String>,
    pub task_cards: HashMap<String, String>,
    pub verdict: String,
    pub missing_perspectives: Vec<String>,
    pub boundary_risks: Vec<String>,
}
