# NFR Requirements Plan — F3: Possession UI

## Plan Steps

- [ ] Step 1: 创建 `nfr-requirements.md`
- [ ] Step 2: 创建 `tech-stack-decisions.md`

## Design Questions

### Q1: WebSocket 客户端
- [Answer]: A — 原生 WebSocket API

### Q2: 大量 chunk 渲染性能
- [Answer]: B — useTransition 低优先级更新

### Q3: 自动滚动策略
- [Answer]: C — 仅 streaming 时自动滚动
