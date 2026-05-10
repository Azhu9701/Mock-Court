# Tech Stack Decisions — B1: Foundation

## Core Dependencies

| Category | Crate | Version | Rationale |
|----------|-------|---------|-----------|
| SQLite | **rusqlite** (bundled) | 0.31+ | 最成熟，bundled feature 免系统依赖 |
| YAML | **serde_yaml** | 0.9+ | Rust YAML 标准库，与魂档案格式一致 |
| Async Runtime | **tokio** | 1.x | Cargo workspace 统一 runtime |
| Serde | **serde** + **serde_json** | 1.x | 序列化标准 |
| Config | **config** crate | 0.14+ | YAML + 环境变量覆盖 |
| DateTime | **chrono** | 0.4+ | 时间处理标准库 |
| UUID | **uuid** (v4) | 1.x | Session/Message/CallRecord ID 生成 |
| Logging | **tracing** | 0.1+ | 结构化日志 |
| File Lock | **fs2** | 0.4+ | 文件锁（flock） |

## Config Strategy

```
config/default.yaml         # 默认值（仓库内）
config/local.yaml           # 本地覆盖（gitignored）
环境变量                     # 运行时覆盖（WANMINFAN_* 前缀）
```

### 加载优先级
```
环境变量 > config/local.yaml > config/default.yaml
```

### 环境变量映射
| 环境变量 | Config 键 | 默认值 |
|----------|----------|--------|
| WANMINFAN_DATA_DIR | data_dir | ./data |
| WANMINFAN_SERVER_HOST | server_host | 127.0.0.1 |
| WANMINFAN_SERVER_PORT | server_port | 3001 |
| WANMINFAN_SOURCE | import_source | — |

## Cargo.toml (foundation crate)

```toml
[package]
name = "foundation"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
serde_yaml = "0.9"
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
config = "0.14"
tracing = "0.1"
fs2 = "0.4"
thiserror = "1"
async-trait = "0.1"
```

## Port Convention

```
Next.js dev server  :3000    (前端开发)
axum API server      :3001    (后端 API + WebSocket)
```

Next.js 开发时通过 `next.config.js` 的 `rewrites` 将 `/api/*` 代理到 `:3001`。

## Workspace Cargo.toml

```toml
[workspace]
members = [
    "rust/foundation",
    "rust/registry",
    "rust/ai-gateway",
    "rust/possession",
    "rust/archive",
    "rust/api",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
thiserror = "1"
```
