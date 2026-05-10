# NFR Requirements Plan — B1: Foundation

## Plan Checklist

- [ ] Generate `nfr-requirements.md` — 性能、可靠性、数据完整性要求
- [ ] Generate `tech-stack-decisions.md` — Rust crate 选型

## NFR Questions

### Question 1: SQLite 库选型
推荐使用哪个 Rust SQLite 库？

A) rusqlite — 最成熟，同步 API，需配合 tokio::task::spawn_blocking
B) sqlx — 异步原生，支持编译时查询检查
C) sea-orm — ORM 层，自动迁移，但 Foundation 层过重
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 2: 配置文件格式
应用配置（Config struct）使用什么格式？

A) TOML — Rust 生态标准（Cargo.toml 风格）
B) YAML — 与魂档案格式一致
C) 环境变量 — 12-factor app 风格
D) YAML + 环境变量覆盖 — 配置文件为基础，环境变量覆盖敏感项
E) Other (please describe after [Answer]: tag below)

[Answer]: D

### Question 3: 前端与后端通信端口
Next.js（前端）和 axum（后端）的端口约定？

A) Next.js :3000, axum :3001（标准约定）
B) Next.js :3000, axum :8080
C) 统一由 axum 服务前端静态文件（单端口）
D) Other (please describe after [Answer]: tag below)

[Answer]: A 
