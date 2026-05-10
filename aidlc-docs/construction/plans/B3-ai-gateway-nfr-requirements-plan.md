# NFR Requirements Plan — B3: AI Gateway

## Plan Checklist

- [x] Generate `nfr-requirements.md` — 网络性能、超时、连接池、流式性能
- [x] Generate `tech-stack-decisions.md` — HTTP client, SSE 解析, 模板引擎选型

## NFR Questions

### Question 1: HTTP Client 库选型
LLM API 调用的 HTTP client 使用什么库？

A) reqwest — Rust 生态标准，支持 HTTP/2、连接池、streaming，最成熟
B) hyper — 底层 HTTP 库，更轻量但需自行处理更多细节
C) isahc — 简单 HTTP client，curl 绑定
D) Other (please describe after [Answer]: tag below)

[Answer]: 

### Question 2: SSE 解析方案
流式响应的 SSE (Server-Sent Events) 如何解析？

A) 手动解析 — 逐行读取 reqwest response stream，按 SSE 协议解析 `data:` 行
B) eventsource-stream crate — 专用 SSE 解析库，处理 `data:`/`event:`/`id:` 字段
C) reqwest-eventsource — reqwest 生态的 SSE 扩展
D) Other (please describe after [Answer]: tag below)

[Answer]: 

### Question 3: 连接管理策略
HTTP 连接池和连接复用策略？

A) reqwest 默认 Client 连接池 — 使用 `reqwest::Client::new()` 内置连接池，自动复用
B) 单连接 — 每次请求新建连接
C) 自定义连接池配置 — 指定 pool_max_idle_per_host、pool_idle_timeout 等
D) Other (please describe after [Answer]: tag below)

[Answer]: 

### Question 4: 异步运行时策略
Provider 的 HTTP 调用是否需要 spawn 到独立 task？

A) 直接在当前 async context 中 await — 简单直接
B) `tokio::spawn` 到独立 task — 每个 LLM 调用独立 task，便于并行和取消
C) 专用 task set — 每个 provider 一个专属 task + mpsc channel 请求队列
D) Other (please describe after [Answer]: tag below)

[Answer]: 
