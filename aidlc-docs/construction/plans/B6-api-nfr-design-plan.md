# NFR Design Plan — B6: API Layer

## Plan Steps

- [x] Step 1: 创建 `nfr-design-patterns.md` — 中间件链/错误映射/优雅关闭等模式
- [x] Step 2: 创建 `logical-components.md` — AppState/Router/Middleware 链逻辑组件

## Design Questions

### Q1: FoundationError → HTTP Status 映射粒度
`FoundationError` 包含多种变体（SoulNotFound、Validation、Storage、InvalidState 等）。映射到 HTTP 状态码的策略？
- [Answer]: A — 粗粒度映射

### Q2: 依赖注入方式
- [Answer]: A — `State<Arc<AppState>>`
