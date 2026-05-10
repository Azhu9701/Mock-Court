# NFR Requirements Plan — B6: API Layer

## Plan Steps

- [x] Step 1: 分析 B6 functional design 确定 NFR 需求
- [x] Step 2: 创建 `nfr-requirements.md` — 性能/并发/可靠性指标
- [x] Step 3: 创建 `tech-stack-decisions.md` — axum 及其依赖选择

## Design Questions

### Q1: HTTP 请求超时
B6 提供的 HTTP 端点中，possess、archive/export 等操作会立即返回（异步执行），但 import、analytics 聚合查询可能需要等待。HTTP 请求超时如何设置？
- [Answer]: A — 30s 短超时

### Q2: Request Body 大小限制
- [Answer]: D — 不限制

### Q3: 速率限制
- [Answer]: A — 不需要

### Q4: TLS/HTTPS
- [Answer]: A — 不需要

### Q5: Tracing/Log 级别
- [Answer]: A — `info` 级别
