# Framework Extraction Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete CLI `snake init` scaffold command + Rust code de-domainification + frontend component alias exports.

**Architecture:** Three workstreams: (1) extend `rust/cli` with `Init` subcommand that renders templates into new project directories, (2) refactor hardcoded Chinese domain terms in `prompt.rs` and `triage.rs` to read from `DomainProfile`, (3) add `agent-*` named re-exports in Next.js components without breaking existing `soul-*` imports.

**Tech Stack:** Rust (clap, tera/serde for templating), Next.js TypeScript (barrel exports)

---

## File Structure Map

```
Files to CREATE:
  rust/cli/src/init.rs              # Init subcommand logic (template rendering, dir creation)
  rust/cli/templates/               # Embedded template files for snake init

Files to MODIFY:
  rust/cli/Cargo.toml               # Add tera dependency for template rendering
  rust/cli/src/main.rs              # Add Init variant to Commands enum, wire cmd_init
  rust/ai-gateway/src/prompt.rs     # ~12 locations: replace hardcoded "魂"/"幡主"/"辩证综合" with domain terms
  rust/possession/src/triage.rs     # Move CHINESE_MARKERS consts → load from DomainProfile.trigger_markers
  rust/foundation/src/domain.rs     # Add trigger_markers field + from_yaml deserialization
  rust/foundation/src/models.rs     # Add AgentProfile type alias for SoulProfile
  rust/api/src/routes/config.rs     # Add GET/POST /config/domain endpoints if missing
  nextjs/components/providers.tsx   # Already has DomainProvider — verify
  nextjs/components/index.ts        # NEW: barrel file with agent-* re-exports
  nextjs/contexts/domain-context.tsx # Already exists — verify it reads from /api/v1/config/domain

Files NOT modified (reference only):
  rust/possession/src/modes/*.rs    # Prompt terms come from PromptBuilder, which already uses DomainProfile
  rust/possession/src/ws.rs         # WsEventType names kept as-is (SoulToken etc.), alias later
  nextjs/components/soul-*.tsx      # Existing files unchanged; new barrel adds agent-* re-exports
```

---

### Task 1: Add trigger_markers to DomainProfile

**Files:**
- Modify: `rust/foundation/src/domain.rs`

**Why:** `triage.rs` currently has hardcoded Chinese keyword arrays. To make the framework domain-switchable, triage must read keywords from `DomainProfile`, which loads from `domain.yaml`.

- [ ] **Step 1: Add trigger_markers struct and field to DomainProfile**

Read `rust/foundation/src/domain.rs` to understand current structure, then add:

```rust
// In rust/foundation/src/domain.rs, add after existing fields:

/// Per-mode trigger keywords loaded from domain.yaml.
/// These drive the triage classifier so users can define
/// mode-detection keywords in their own language/domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMarkers {
    #[serde(default)]
    pub single: Vec<String>,
    #[serde(default)]
    pub conference: Vec<String>,
    #[serde(default)]
    pub debate: Vec<String>,
    #[serde(default)]
    pub relay: Vec<String>,
    #[serde(default)]
    pub learn: Vec<String>,
    #[serde(default)]
    pub practice: Vec<String>,
}

impl Default for TriggerMarkers {
    fn default() -> Self {
        TriggerMarkers {
            single: vec!["简单".into(), "快速".into(), "一句话".into(), "查询".into()],
            conference: vec!["分析".into(), "综合".into(), "多角度".into(), "全面".into(), "评估".into()],
            debate: vec!["还是".into(), "要么".into(), "或者".into(), "利弊".into(), "优劣".into(), "权衡".into()],
            relay: vec!["步骤".into(), "流程".into(), "阶段".into(), "路线".into(), "路径".into()],
            learn: vec!["学习".into(), "了解".into(), "是什么".into(), "教我".into(), "解释".into()],
            practice: vec!["我的".into(), "我公司".into(), "我们".into(), "最近".into(), "正在".into()],
        }
    }
}
```

Add field to `DomainProfile`:
```rust
// Add inside DomainProfile struct:
#[serde(default)]
pub trigger_markers: TriggerMarkers,
```

- [ ] **Step 2: Update DomainProfile::default()**

Ensure `trigger_markers: TriggerMarkers::default()` is included in the Default impl.

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p foundation`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add rust/foundation/src/domain.rs
git commit -m "feat(foundation): add TriggerMarkers to DomainProfile for domain-switchable triage"
```

---

### Task 2: Refactor triage.rs to use DomainProfile trigger_markers

**Files:**
- Modify: `rust/possession/src/triage.rs`
- Modify: `rust/possession/Cargo.toml` (if DomainProfile not already accessible)

- [ ] **Step 1: Change triage function signature to accept DomainProfile**

Read current `rust/possession/src/triage.rs` (already read above). Replace the hardcoded const arrays and function with a version that accepts `&DomainProfile`:

```rust
// rust/possession/src/triage.rs — REPLACE entire file content

use foundation::{DomainProfile, PossessionMode};
use crate::{EntryType, PossessionInput};

const TRIGGER_THRESHOLD: u32 = 2;

fn any_contains(text: &str, markers: &[String]) -> bool {
    markers.iter().any(|m| text.contains(m.as_str()))
}

pub fn triage(input: &PossessionInput, domain: &DomainProfile) -> EntryType {
    // Explicit mode override
    if let Some(ref mode) = input.mode {
        return match mode {
            PossessionMode::Single => EntryType::Single,
            PossessionMode::Conference => EntryType::Conference,
            PossessionMode::Debate => EntryType::Debate,
            PossessionMode::Relay => EntryType::Relay,
            PossessionMode::Learn => EntryType::Learn,
            PossessionMode::PracticeOpening => EntryType::PracticeOpening,
        };
    }

    let task = &input.task;
    let soul_count = input.souls.len();

    let tm = &domain.trigger_markers;

    let practice_score = score_markers(task, &tm.practice).min(3);
    let learn_score = score_markers(task, &tm.learn).min(3);
    let debate_score = {
        let mut s = score_markers(task, &tm.debate);
        if soul_count == 2 { s += 1; }
        s.min(3)
    };
    let relay_score = score_markers(task, &tm.relay).min(3);

    if practice_score >= TRIGGER_THRESHOLD {
        EntryType::PracticeOpening
    } else if debate_score >= TRIGGER_THRESHOLD {
        EntryType::Debate
    } else if learn_score >= TRIGGER_THRESHOLD && soul_count <= 1 {
        EntryType::Learn
    } else if relay_score >= TRIGGER_THRESHOLD {
        EntryType::Relay
    } else if soul_count >= 2 {
        EntryType::Conference
    } else {
        EntryType::Single
    }
}

fn score_markers(task: &str, markers: &[String]) -> u32 {
    let mut score = 0u32;
    for m in markers {
        if task.contains(m.as_str()) {
            score += 1;
            if score >= 3 { break; }
        }
    }
    score
}
```

- [ ] **Step 2: Update all call sites of triage()**

Find all `triage::triage(` calls and add the `&domain` argument. The main call site is in `rust/possession/src/lib.rs` or `rust/possession/src/modes/mod.rs`. The `PossessionEngine` already holds a `DomainProfile` reference.

Search for `triage(` in the possession crate and update each call:
```rust
// Before:
let entry_type = triage::triage(&input);
// After:
let entry_type = triage::triage(&input, &self.domain);
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p possession`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add rust/possession/src/triage.rs
git commit -m "refactor(possession): triage reads trigger keywords from DomainProfile"
```

---

### Task 3: Add AgentProfile type alias

**Files:**
- Modify: `rust/foundation/src/models.rs`

- [ ] **Step 1: Add type alias**

After the `SoulProfile` struct definition, add:

```rust
/// Framework-standard name for SoulProfile.
/// SoulProfile is retained for backward compatibility with existing code.
pub type AgentProfile = SoulProfile;
```

- [ ] **Step 2: Add re-export in lib.rs**

In `rust/foundation/src/lib.rs`, ensure `AgentProfile` is re-exported alongside `SoulProfile`.

- [ ] **Step 3: Verify**

Run: `cargo check -p foundation`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add rust/foundation/src/models.rs rust/foundation/src/lib.rs
git commit -m "feat(foundation): add AgentProfile as standard type alias for SoulProfile"
```

---

### Task 4: Add CLI init subcommand

**Files:**
- Create: `rust/cli/src/init.rs`
- Modify: `rust/cli/src/main.rs`
- Modify: `rust/cli/Cargo.toml`

- [ ] **Step 1: Add tera dependency**

Edit `rust/cli/Cargo.toml`, add to `[dependencies]`:
```toml
tera = "1"
```

- [ ] **Step 2: Create init.rs with template rendering logic**

Create `rust/cli/src/init.rs`:

```rust
use std::fs;
use std::path::Path;

pub struct InitArgs {
    pub project_name: String,
    pub domain: String,
    pub port: u16,
    pub frontend_port: u16,
    pub skip_frontend: bool,
}

pub fn run(args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = Path::new(&args.project_name);

    if project_dir.exists() {
        eprintln!("Error: directory '{}' already exists", args.project_name);
        std::process::exit(1);
    }

    // Create directory structure
    create_dirs(project_dir, args.skip_frontend)?;

    // Write config files
    write_config(project_dir, &args)?;

    // Write Cargo.toml
    write_cargo_toml(project_dir, &args)?;

    // Write main.rs skeleton
    write_main_rs(project_dir, &args)?;

    // Write .env.example
    write_env_example(project_dir)?;

    // Write Next.js skeleton (if not skipped)
    if !args.skip_frontend {
        write_nextjs_skeleton(project_dir, &args)?;
    }

    // Write README
    write_readme(project_dir, &args)?;

    println!("✓ Project '{}' created successfully!", args.project_name);
    println!();
    println!("Next steps:");
    println!("  cd {}", args.project_name);
    println!("  cp .env.example .env   # edit .env with your API keys");
    println!("  cd rust/agent-app && cargo run  # start API (port {})", args.port);
    if !args.skip_frontend {
        println!("  cd nextjs && pnpm dev   # start frontend (port {})", args.frontend_port);
    }

    Ok(())
}

fn create_dirs(project_dir: &Path, skip_frontend: bool) -> Result<(), Box<dyn std::error::Error>> {
    let dirs = vec![
        "config",
        "data/agents",
        "data/knowledge",
        "rust/agent-app/src/agents",
        "rust/agent-app/src/modes",
        "rust/agent-app/src/tools",
    ];

    let frontend_dirs = if !skip_frontend {
        vec!["nextjs/app", "nextjs/components", "nextjs/hooks", "nextjs/lib", "nextjs/public"]
    } else {
        vec![]
    };

    for d in dirs.iter().chain(frontend_dirs.iter()) {
        fs::create_dir_all(project_dir.join(d))?;
    }
    Ok(())
}

fn write_config(project_dir: &Path, args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    // default.yaml
    let default_yaml = format!(r#"# Server configuration
server_host: "127.0.0.1"
server_port: {port}
nextjs_port: {frontend_port}

# Data paths
data_dir: "./data"
agents_dir: "./data/agents"
archive_dir: "./data/archive"
db_path: "./data/app.db"

# Rate limiting
rate_limit:
  enabled: true
  requests_per_second: 30
  burst_size: 60

# CORS
cors_origins:
  - "http://localhost:{frontend_port}"
"#, port = args.port, frontend_port = args.frontend_port);

    fs::write(project_dir.join("config/default.yaml"), default_yaml)?;

    // domain.yaml — copy from the selected template
    let domain_template = match args.domain.as_str() {
        "philosophy" => include_str!("../../meta/templates/domains/philosophy/domain.yaml"),
        "legal" => include_str!("../../meta/templates/domains/legal/domain.yaml"),
        "labor" => include_str!("../../meta/templates/domains/labor/domain.yaml"),
        _ => include_str!("../../meta/templates/base/domain.yaml.tmpl"),
    };

    fs::write(project_dir.join("config/domain.yaml"), domain_template)?;

    // Write a sample agent
    let sample_agent = format!(r#"---
name: "assistant"
title: "通用助手"
description: "默认助手Agent"
model: "deepseek-chat"
tools: []
trigger_keywords: ["帮助", "问题"]
system_prompt: |
  你是一个通用助手，请简洁清晰地回答用户问题。
---
"#);
    fs::write(project_dir.join("data/agents/assistant.md"), sample_agent)?;

    Ok(())
}

fn write_cargo_toml(project_dir: &Path, args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Generated project uses path dependencies to reference framework crates.
    // No workspace — it's a standalone crate during Phase 1.
    // Paths go from <project>/rust/agent-app/ up 3 levels to repo root.
    // Phase 3 switches to crates.io dependencies.
    let app_toml = r#"[package]
name = "agent-app"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }

# Framework crates (local path during Phase 1)
foundation = { path = "../../../rust/foundation" }
ai-gateway = { path = "../../../rust/ai-gateway" }
registry = { path = "../../../rust/registry" }
possession = { path = "../../../rust/possession" }
archive = { path = "../../../rust/archive" }
api = { path = "../../../rust/api" }
"#;
    fs::create_dir_all(project_dir.join("rust/agent-app/src"))?;
    fs::write(project_dir.join("rust/agent-app/Cargo.toml"), app_toml)?;

    Ok(())
}

fn write_main_rs(project_dir: &Path, args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let main_rs = format!(r#"use std::sync::Arc;
use foundation::Config;
use api::state::AppState;
use api::routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let config = Config::load("config/default.yaml")?;
    let state = AppState::new(config).await?;

    let app = routes::create_router(Arc::new(state));

    let addr = format!("127.0.0.1:{}", {port});
    tracing::info!("API server listening on {{}}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}}
"#, port = args.port);

    fs::write(project_dir.join("rust/agent-app/src/main.rs"), main_rs)?;
    Ok(())
}

fn write_env_example(project_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let env = r#"# AI Provider API Keys (fill at least one)
OPENAI_API_KEY=sk-xxxxx
CLAUDE_API_KEY=sk-ant-xxxxx
DEEPSEEK_API_KEY=sk-xxxxx

# Optional: Local model
LMSTUDIO_URL=http://localhost:1234/v1
LMSTUDIO_MODEL=qwen/qwen3.6-27b

# Optional: API auth token
API_TOKEN=your-secret-token
"#;
    fs::write(project_dir.join(".env.example"), env)?;
    Ok(())
}

fn write_readme(project_dir: &Path, args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let readme = format!(r#"# {name}

Generated with `snake init --domain {domain}`.

## Quick Start

```bash
# 1. Configure API keys
cp .env.example .env
# Edit .env with your AI provider keys

# 2. Start API server (port {port})
cd rust/agent-app && cargo run

# 3. Start frontend (port {frontend_port})
cd nextjs && pnpm dev
```

## Project Structure

```
config/         # YAML configuration (domain.yaml, default.yaml)
data/agents/    # Agent definition files (Markdown + YAML frontmatter)
data/knowledge/ # Domain knowledge base
rust/           # Backend (Rust workspace)
nextjs/         # Frontend (Next.js 16)
```

See `meta/docs/` for framework documentation.
"#, name = args.project_name, domain = args.domain, port = args.port, frontend_port = args.frontend_port);

    fs::write(project_dir.join("README.md"), readme)?;
    Ok(())
}

fn write_nextjs_skeleton(project_dir: &Path, args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Minimal package.json with framework dependencies
    let pkg_json = format!(r#"{{
  "name": "{name}-frontend",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "dev": "next dev -p {port}",
    "build": "next build",
    "start": "next start"
  }},
  "dependencies": {{
    "next": "^16",
    "react": "^19",
    "react-dom": "^19",
    "lucide-react": "^1",
    "tailwindcss": "^4",
    "@tailwindcss/postcss": "^4"
  }}
}}
"#, name = args.project_name, port = args.frontend_port);

    fs::create_dir_all(project_dir.join("nextjs"))?;
    fs::write(project_dir.join("nextjs/package.json"), pkg_json)?;

    // Minimal layout.tsx
    let layout_tsx = r#"export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh">
      <body>{children}</body>
    </html>
  );
}
"#;
    fs::create_dir_all(project_dir.join("nextjs/app"))?;
    fs::write(project_dir.join("nextjs/app/layout.tsx"), layout_tsx)?;

    // Minimal page.tsx
    let page_tsx = r#"export default function Home() {
  return (
    <main className="p-8">
      <h1 className="text-2xl font-bold">Agent System Ready</h1>
      <p className="mt-2 text-gray-600">Your multi-agent reasoning system is running.</p>
    </main>
  );
}
"#;
    fs::write(project_dir.join("nextjs/app/page.tsx"), page_tsx)?;

    Ok(())
}
```

- [ ] **Step 3: Add Init variant to CLI main.rs**

Edit `rust/cli/src/main.rs`:

Add module declaration at top:
```rust
mod init;
```

Add variant to `Commands` enum (after `Sessions`):
```rust
/// 生成新项目骨架
Init {
    /// 项目名称
    name: String,
    /// 领域模板: custom, philosophy, legal, labor
    #[arg(long, default_value = "custom")]
    domain: String,
    /// API 端口
    #[arg(long, default_value = "3001")]
    port: u16,
    /// 前端端口
    #[arg(long, default_value = "3000")]
    frontend_port: u16,
    /// 跳过前端生成
    #[arg(long)]
    skip_frontend: bool,
},
```

Add match arm in `main()`:
```rust
Commands::Init { name, domain, port, frontend_port, skip_frontend } => {
    init::run(init::InitArgs {
        project_name: name,
        domain,
        port,
        frontend_port,
        skip_frontend,
    })?;
}
```

- [ ] **Step 4: Handle include_str! path resolution**

The `include_str!("../../meta/templates/...")` paths work when building from the `rust/cli` directory. Add a build-time check. Since `meta/` is at repo root and `rust/cli/` is two levels down, `../../meta/` is correct.

- [ ] **Step 5: Verify compilation**

Run: `cargo check -p snake-cli`
Expected: Compiles without errors (may need path adjustments).

- [ ] **Step 6: Test init command**

Run: `cargo run -p snake-cli -- init test-project --domain custom`
Expected: Creates `test-project/` directory with all files.
Verify: `ls -la test-project/` shows expected structure.
Cleanup: `rm -rf test-project`

- [ ] **Step 7: Commit**

```bash
git add rust/cli/
git commit -m "feat(cli): add snake init command for project scaffolding"
```

---

### Task 5: Frontend agent-* barrel exports

**Files:**
- Create: `nextjs/components/agent.ts` (barrel file)

- [ ] **Step 1: Create barrel file with re-exports**

Create `nextjs/components/agent.ts`:

```typescript
// Agent component aliases — framework-standard names.
// Original soul-* imports continue to work unchanged.

export { SoulCardGrid as AgentCardGrid } from "./soul-card-grid";
export { SoulCard as AgentCard } from "./soul-card";
export { SoulPanel as AgentPanel } from "./soul-panel";
export { SoulFilterBar as AgentFilterBar } from "./soul-filter-bar";
export { SoulChatBubble as AgentChatBubble } from "./soul-chat-bubble";
export { SoulResponseCard as AgentResponseCard } from "./soul-response-card";
export { SoulResponsesGrid as AgentResponsesGrid } from "./soul-responses-grid";
export { SoulCarousel as AgentCarousel } from "./soul-carousel";
export { SoulOverviewPanel as AgentOverviewPanel } from "./soul-overview-panel";
export { SoulEffectivenessTable as AgentEffectivenessTable } from "./soul-effectiveness-table";
export { SoulModelConfig as AgentModelConfig } from "./soul-model-config";
export { EditSoulDialog as EditAgentDialog } from "./edit-soul-dialog";
export { DeleteSoulButton as DeleteAgentButton } from "./delete-soul-button";
export { SummonButton as InvokeAgentButton } from "./summon-button";
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd nextjs && pnpm tsc --noEmit`
Expected: No new errors (re-exports don't create new dependencies).

- [ ] **Step 3: Commit**

```bash
git add nextjs/components/agent.ts
git commit -m "feat(frontend): add agent-* barrel exports for framework-standard names"
```

---

### Task 6: Domain config API endpoints

**Files:**
- Modify: `rust/api/src/routes/config.rs`

- [ ] **Step 1: Add GET /config/domain endpoint (if missing)**

Read current `rust/api/src/routes/config.rs`. If `GET /config/domain` does not exist, add:

```rust
pub async fn get_domain(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let domain = &state.config.domain;
    Json(serde_json::json!({
        "name": domain.name,
        "icon": domain.icon,
        "system_name": domain.system_name,
        "agent_noun": domain.agent_noun,
        "user_title": domain.user_title,
        "synthesis_verb": domain.synthesis_verb,
        "dimensions": domain.dimensions,
    }))
}
```

Register the route in the config router:
```rust
.route("/domain", get(get_domain))
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p api`
Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add rust/api/src/routes/config.rs
git commit -m "feat(api): add GET /config/domain endpoint for frontend DomainContext"
```

---

### Task 7: Integration test — end-to-end verification

- [ ] **Step 1: Run full workspace check**

```bash
cargo check --workspace
```
Expected: All crates compile without errors.

- [ ] **Step 2: Run init and verify generated project builds**

```bash
cargo run -p snake-cli -- init test-e2e --domain legal
cd test-e2e
# Verify Cargo.toml paths are correct (relative to workspace root)
cargo check -p agent-app
```

Expected: The generated project's Cargo.toml works with the local crate paths.

Note: The generated `rust/agent-app/Cargo.toml` uses `path = "../../../rust/foundation"` (3 levels up from `test-e2e/rust/agent-app/` to repo root). This only works inside the monorepo. Phase 1 limitation. Phase 3 switches to crates.io dependencies.

- [ ] **Step 3: Cleanup**

```bash
rm -rf test-e2e
```

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "chore: finalize Phase 1 framework extraction deliverables"
```
