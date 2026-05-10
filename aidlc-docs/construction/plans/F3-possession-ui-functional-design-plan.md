# Functional Design Plan — F3: Possession UI

## Plan Steps

- [ ] Step 1: 创建 `domain-entities.md` — 附体/模式/流式类型
- [ ] Step 2: 创建 `business-logic-model.md` — 组件树 + WS 连接流程
- [ ] Step 3: 创建 `business-rules.md` — 模式路由/流式渲染/错误处理
- [ ] Step 4: 创建 `frontend-components.md` — 组件 Props/State

## Design Questions

### Q1: 终端/聊天布局风格
附体界面采用什么视觉风格？
- [Answer]: B — 聊天风格（气泡对话）

### Q2: 模式选择入口
- [Answer]: B — Wizard 向导

### Q3: 合议多魂展示
- [Answer]: B — 主副面板（1大N小）

### Q4: 辩论展示
- [Answer]: B — 上下分屏
