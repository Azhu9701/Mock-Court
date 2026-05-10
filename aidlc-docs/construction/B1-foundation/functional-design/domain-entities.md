# Domain Entities — B1: Foundation

## Entity Overview

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│    Soul      │     │   Session    │     │  CallRecord  │
│  (FS: YAML+MD)│     │  (SQLite)   │     │  (SQLite)    │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │ 1:N                │ 1:N                │ 1:1
       ▼                    ▼                    ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Message    │     │ PossessionRun│     │Effectiveness │
│  (SQLite)    │     │  (SQLite)    │     │   (SQLite)   │
└──────────────┘     └──────────────┘     └──────────────┘
```

## Soul (文件系统: YAML + Markdown)

魂档案格式：YAML frontmatter（元数据）+ Markdown body（summon_prompt）

```yaml
# souls/{name}.md
---
name: 列宁
ismism_code: 4-1-4-3
field: 政治
ontology: 唯物主义
epistemology: 实践论
teleology: 革命
grade: S
domains:
  - 政治
  - 革命
  - 哲学
exclude_scenarios:
  - 无原则妥协和调和主义
  - 脱离实际的空谈和本本主义
summon_count: 20
effectiveness:
  effective: 20
  partial: 0
  invalid: 0
created_at: "2026-04-20T00:00:00Z"
updated_at: "2026-05-04T08:44:20Z"
tags: [政治, 革命, 马克思主义, 4-1-4-3]
---

# 列宁 — Summon Prompt

[魂的完整 summon_prompt 内容...]
```

### Rust Model

```rust
struct SoulProfile {
    name: String,
    ismism_code: String,
    field: String,
    ontology: String,
    epistemology: String,
    teleology: String,
    grade: SoulGrade,
    domains: Vec<String>,
    exclude_scenarios: Vec<String>,
    summon_count: u32,
    effectiveness: EffectivenessStats,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    tags: Vec<String>,
    summon_prompt: String,      // Markdown body
    practice_observations: Vec<PracticeObservation>,
}

enum SoulGrade { S, A, B, C, D }

struct EffectivenessStats {
    effective: u32,
    partial: u32,
    invalid: u32,
}

struct PracticeObservation {
    date: NaiveDate,
    observation: String,
    revision_type: RevisionType, // Confirmed / Modified / Overturned
}
```

## Session (SQLite)

扁平模式：一个 session 含多个 messages。

```sql
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    mode        TEXT NOT NULL,   -- single / conference / debate / relay / learn / practice_opening
    status      TEXT NOT NULL DEFAULT 'active',  -- active / completed / archived
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE messages (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    role        TEXT NOT NULL,   -- user / soul / synthesis / system
    soul_name   TEXT,            -- NULL for user/system messages
    content     TEXT NOT NULL,
    seq         INTEGER NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Rust Model

```rust
struct Session {
    id: String,
    title: String,
    mode: PossessionMode,
    status: SessionStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    messages: Vec<Message>,
}

enum PossessionMode {
    Single,
    Conference,
    Debate,
    Relay,
    Learn,
    PracticeOpening,
}

enum SessionStatus {
    Active,
    Completed,
    Archived,
}

struct Message {
    id: String,
    session_id: String,
    role: MessageRole,
    soul_name: Option<String>,
    content: String,
    seq: u32,
    created_at: DateTime<Utc>,
}

enum MessageRole {
    User,
    Soul,
    Synthesis,
    System,
}
```

## CallRecord (SQLite)

有效性评分：枚举 + 文本判据，与现有万民幡一致。

```sql
CREATE TABLE call_records (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES sessions(id),
    soul_name       TEXT NOT NULL,
    mode            TEXT NOT NULL,
    task_summary    TEXT NOT NULL,
    effectiveness   TEXT NOT NULL,  -- effective / partial / invalid
    notes           TEXT NOT NULL,  -- 判据文本
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Rust Model

```rust
struct CallRecord {
    id: String,
    session_id: String,
    soul_name: String,
    mode: PossessionMode,
    task_summary: String,
    effectiveness: Effectiveness,
    notes: String,
    created_at: DateTime<Utc>,
}

enum Effectiveness {
    Effective,
    Partial,
    Invalid,
}
```

## Registry (文件系统 + SQLite 缓存)

```yaml
# data/registry.yaml
souls:
  列宁:
    ismism_code: "4-1-4-3"
    grade: S
    domains: [政治, 革命, 哲学]
    summon_count: 20
    effectiveness:
      effective: 20
      partial: 0
      invalid: 0
    created_at: "2026-04-20"
    updated_at: "2026-05-04"
```

### Rust Model

```rust
struct Registry {
    souls: HashMap<String, RegistryEntry>,
}

struct RegistryEntry {
    ismism_code: String,
    grade: SoulGrade,
    domains: Vec<String>,
    summon_count: u32,
    effectiveness: EffectivenessStats,
    created_at: NaiveDate,
    updated_at: NaiveDate,
}
```

## IsmismFilter (查询用)

```rust
struct IsmismFilter {
    field: Option<String>,          // 场域
    ontology: Option<String>,       // 本体论
    epistemology: Option<String>,   // 认识论
    teleology: Option<String>,      // 目的论
    grade: Option<SoulGrade>,       // 品级
}

struct IsmismCode {
    field: u8,          // 1=唯心 2=唯物 3=批判 4=革命
    ontology: u8,       // 1=有神 2=理性 3=生命 4=解构
    epistemology: u8,   // 1=经验 2=逻辑 3=直觉 4=实践
    teleology: u8,      // 1=秩序 2=效率 3=自由 4=解放
}
```

## File System Layout

```
data/
├── registry.yaml
├── souls/
│   ├── 列宁.md            # YAML frontmatter + Markdown summon_prompt
│   ├── 毛泽东.md
│   └── ...
├── archive/
│   └── YYYY/
│       └── MM/
│           └── DD/
│               └── {session_id}/
│                   ├── {soul_name}.md
│                   ├── synthesis.md
│                   └── verdict.md (辩论模式)
├── call-records.yaml       # 兼容现有格式，同时写入 SQLite
└── wanminfan.db            # SQLite (sessions, messages, call_records, analytics)
```
