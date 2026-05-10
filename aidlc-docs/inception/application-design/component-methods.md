# Component Methods — 万民幡 Web Application

> Detailed business logic will be defined in Functional Design (CONSTRUCTION phase, per-unit).

## Soul Registry Service

```
// 魂查询
list_souls(filter: IsmismFilter?, query: str?) -> Vec<SoulSummary>
get_soul(name: str) -> SoulProfile
search_souls(query: str, field: SearchField) -> Vec<SoulMatch>

// Registry 管理
load_registry() -> Registry
reload_registry() -> Registry
get_ismism_distribution() -> IsmismStats
```

## Possession Engine

```
// 模式入口
start_possession(mode: PossessionMode, input: PossessionInput) -> Session

// 单魂
possess_single(soul: str, task: str) -> StreamOutput

// 合议
start_conference(task: str, souls: Vec<str>) -> ConferenceSession
run_parallel_souls(session: ConferenceSession) -> Vec<SoulOutput>
run_dialectical_synthesis(outputs: Vec<SoulOutput>) -> SynthesisReport

// 辩论
start_debate(topic: str, soul_a: str, soul_b: str) -> DebateSession
run_debate(session: DebateSession) -> (SoulOutput, SoulOutput, Verdict)

// 接力
start_relay(task: str, soul_chain: Vec<str>) -> RelaySession
run_relay_stage(prev_output: Option<str>, soul: str) -> SoulOutput

// 实践开口
detect_practitioner(input: str) -> bool
run_practice_opening_P1(data: PractitionerInput) -> FieldData
run_practice_opening_P2(field_data: FieldData, souls: Vec<str>) -> Vec<DigestionReport>
run_practice_opening_P3(reports: Vec<DigestionReport>) -> Vec<RevisionRecord>
run_practice_opening_P4(revisions: Vec<RevisionRecord>) -> ActionMemo

// 入口分流
classify_entry(user_input: str) -> EntryType
```

## AI Gateway

```
// Provider 管理
list_providers() -> Vec<ProviderInfo>
get_provider(name: str) -> Provider

// 调用
call_llm(provider: str, prompt: Prompt, config: CallConfig) -> StreamOutput
call_llm_parallel(requests: Vec<LLMRequest>) -> Vec<StreamOutput>

// Prompt 管理
build_summon_prompt(soul: SoulProfile, task: str) -> Prompt
build_synthesis_prompt(outputs: Vec<SoulOutput>) -> Prompt
build_review_prompt(soul: SoulProfile) -> Prompt

// 并发控制
acquire_token(provider: str) -> Result<Token>
release_token(token: Token)
```

## Archive System

```
// 存档
archive_soul_output(session_id: str, soul: str, output: str) -> ArchivePath
archive_synthesis(session_id: str, report: SynthesisReport) -> ArchivePath
archive_debate(session_id: str, outputs: DebateOutputs) -> Vec<ArchivePath>

// call-records
record_call(entry: CallRecord) -> ()
query_call_records(filter: CallFilter) -> Vec<CallRecord>
get_soul_stats(soul: str) -> SoulStats

// 完整性
verify_archive(session_id: str) -> ArchiveVerification
list_sessions(filter: SessionFilter) -> Vec<SessionSummary>
get_session(session_id: str) -> SessionDetail

// 导出
export_archive(format: ExportFormat) -> ExportBundle
import_archive(bundle: ExportBundle) -> ()
```

## Analytics Engine

```
// 统计
get_summon_stats(period: Period) -> SummonStats
get_soul_effectiveness(soul: str) -> EffectivenessTrend
get_mode_distribution() -> ModeDistribution

// 检测
detect_unsummoned_souls(threshold: Duration) -> Vec<SoulAlert>
detect_low_effectiveness(threshold: f32) -> Vec<BoundaryReview>

// 配额
get_usage_quota() -> UsageQuota
check_rate_limit(action: str) -> bool
```

## Storage Layer

```
// SQLite
db_connect() -> Connection
db_migrate() -> ()
db_query<T>(sql: str, params: Vec<Value>) -> Vec<T>
db_execute(sql: str, params: Vec<Value>) -> ()

// File System
fs_read_yaml<T>(path: str) -> T
fs_write_yaml<T>(path: str, data: T) -> ()
fs_read_markdown(path: str) -> str
fs_write_markdown(path: str, content: str) -> ()
fs_ensure_dir(path: str) -> ()
```

## WebSocket Manager

```
// Session 管理
ws_connect(session_id: str) -> WebSocket
ws_broadcast(session_id: str, event: WsEvent) -> ()
ws_send_soul_output(session_id: str, soul: str, chunk: str) -> ()
ws_send_synthesis(session_id: str, report: SynthesisReport) -> ()
ws_close(session_id: str) -> ()

// Channel 管理
ws_subscribe(session_id: str, channel: str) -> ()
ws_unsubscribe(session_id: str, channel: str) -> ()
```
