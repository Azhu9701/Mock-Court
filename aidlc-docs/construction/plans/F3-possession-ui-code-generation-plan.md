# Code Generation Plan — F3: Possession UI

## Plan Steps

### Config & Hooks
- [ ] Step 1: 创建 `config/possession-modes.ts` — 模式常量配置
- [ ] Step 2: 创建 `hooks/use-websocket.ts` — WS 连接 + 重连 + 消息分发

### Wizard Components
- [ ] Step 3: 创建 `components/mode-card.tsx` — 6 张模式卡片
- [ ] Step 4: 创建 `components/soul-checkbox-list.tsx` — 魂多选列表
- [ ] Step 5: 创建 `components/possession-wizard.tsx` — 5 步向导

### Streaming Components
- [ ] Step 6: 创建 `components/soul-chat-bubble.tsx` — 聊天气泡
- [ ] Step 7: 创建 `components/single-view.tsx` — 单魂布局
- [ ] Step 8: 创建 `components/soul-overview-panel.tsx` + `soul-focus-panel.tsx` + `synthesis-panel.tsx`
- [ ] Step 9: 创建 `components/conference-view.tsx` — 主副面板
- [ ] Step 10: 创建 `components/debate-view.tsx` — 上下分屏
- [ ] Step 11: 创建 `components/relay-view.tsx` + `learn-view.tsx` + `practice-opening-view.tsx`
- [ ] Step 12: 创建 `components/session-runner.tsx` — mode→view 分发
- [ ] Step 13: 创建 `components/session-status-bar.tsx`

### Pages
- [ ] Step 14: 创建 `app/possess/page.tsx` — Wizard 入口
- [ ] Step 15: 创建 `app/possess/[sessionId]/page.tsx` — SessionRunner

### Verification
- [ ] Step 16: `pnpm build` — 0 errors
