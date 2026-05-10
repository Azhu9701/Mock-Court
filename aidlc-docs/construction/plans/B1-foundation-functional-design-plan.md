# Functional Design Plan — B1: Foundation

## Plan Checklist

- [ ] Generate `business-logic-model.md` — Storage trait, data access patterns
- [ ] Generate `business-rules.md` — validation, constraints, integrity rules
- [ ] Generate `domain-entities.md` — Soul, Session, CallRecord, etc.
- [ ] B1 does NOT include frontend → skip frontend-components.md

## Unit Context

B1 Foundation is the data layer for the entire application. It defines:
- **Storage trait** — SQLite + FS 统一抽象
- **Domain models** — all shared data types (Soul, Session, CallRecord, Registry, etc.)
- **Config** — application configuration
- **Migrations** — database schema

## Design Questions

### Question 1: Soul 档案文件格式
魂档案（souls/{name}.xxx）的持久化格式？

A) YAML — 与现有万民幡格式完全兼容，方便导入导出
B) JSON — 解析更快，Rust serde 原生支持好
C) Markdown（带 YAML frontmatter）— 与 Obsidian vault 格式兼容
D) YAML + Markdown 混合（元数据用 YAML，summon_prompt 用 MD）
E) Other (please describe after [Answer]: tag below)

[Answer]: D

### Question 2: CallRecord 有效性评分建模
call-records 的有效性评分如何建模？

A) 枚举（有效/部分有效/无效）— 与现有万民幡一致
B) 枚举 + 数值评分（1-5 + 标签）— 更细粒度
C) 枚举 + 文本判据 — 保留现有格式
D) Other (please describe after [Answer]: tag below)

[Answer]: C

### Question 3: Session 对话会话建模
会话（Session）的数据结构设计？

A) 扁平模式 — 一个 session 含多个 message（含 role: User/Soul/System）
B) 嵌套模式 — session → possession_runs（每 run 含 1-N 魂输出）→ messages
C) 分层模式 — session → rounds → soul_outputs → messages
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 4: 数据库迁移策略
SQLite 数据库迁移方案？

A) 嵌入式 SQL 文件（migrations/001_init.sql）— 简单直接
B) Rust 迁移库（refinery / sqlx migrate）— 版本化管理
C) 应用启动时自动创建表（CREATE TABLE IF NOT EXISTS）
D) Other (please describe after [Answer]: tag below)

[Answer]: C

### Question 5: 文件系统存储路径约定
存档文件的路径命名约定？

A) 遵循现有万民幡 Obsidian 约定（单魂/{魂名}/YYYY-MM-DD-{任务}.md 等）
B) 简化为扁平结构（sessions/{id}/soul_{name}.md）
C) 日期分组（YYYY/MM/DD/{session_id}/{soul}.md）
D) Other (please describe after [Answer]: tag below)

[Answer]: C 
