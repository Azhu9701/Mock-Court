# NFR Requirements Plan — B2: Soul Registry

## Plan Checklist

- [x] Generate `nfr-requirements.md` — 搜索性能、内存使用、启动时间要求
- [x] Generate `tech-stack-decisions.md` — 必要依赖选型（如中文分词库）

## NFR Questions

### Question 1: 倒排索引方案
全文搜索的倒排索引如何实现？

A) 简单 HashMap 分词索引 — 内存内实现，无需额外依赖，24魂足够用
B) Tantivy 全文搜索引擎 — 专业 FTS 库，支持 BM25 排序，但引入较重依赖
C) SQLite FTS5 — 利用已有 SQLite，FTS5 扩展，但需额外建表
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 2: 中文分词策略
中文全文搜索的分词如何处理？

A) 简单单字分词 + 双字组合 — 无外部依赖，24魂场景精度足够
B) jieba-rs 分词 — Rust 中文分词库，分词精度高
C) lindera 分词 — 日本 IPADic 维护的分词库，支持中文
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 3: SoulRegistry 内存策略
启动时加载策略？

A) 全量预加载 — 启动时一次性加载所有魂到内存（24魂 ~240KB，毫秒级）
B) 懒加载 — 首次访问时加载，减少启动时间
C) 热加载 — 启动时加载元数据（列表），详情按需加载
D) Other (please describe after [Answer]: tag below)

[Answer]: A
