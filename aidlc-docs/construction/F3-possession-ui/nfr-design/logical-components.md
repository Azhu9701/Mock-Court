# Logical Components — F3: Possession UI

## File Structure

```
app/possess/
├── page.tsx                    # Wizard (5 steps)
├── layout.tsx                  # 无侧栏布局
└── [sessionId]/
    └── page.tsx                # SessionRunner

hooks/
└── use-websocket.ts            # WS connect + reconnect + message dispatch

components/
├── possession-wizard.tsx       # 5-step Wizard容器
├── mode-card.tsx               # 模式卡片
├── soul-checkbox-list.tsx      # 魂选择器 (复用 fetchSouls)
├── session-runner.tsx          # mode→view dispatch
├── soul-chat-bubble.tsx        # 聊天气泡 (Q1: B)
├── single-view.tsx             # 单魂聊天布局
├── conference-view.tsx         # 主副面板 (Q3: B)
├── soul-overview-panel.tsx     # 概览侧栏
├── soul-focus-panel.tsx        # 聚焦面板
├── synthesis-panel.tsx         # 辩证综合
├── debate-view.tsx             # 上下分屏 (Q4: B)
├── relay-view.tsx              # 接力链
├── learn-view.tsx              # 学习
├── practice-opening-view.tsx   # 实践开口
└── session-status-bar.tsx      # 连接状态

config/
└── possession-modes.ts         # 模式常量配置
```

## Component Dependencies

```
PossessPage
└── PossessionWizard
    ├── Step1: ModeCard[] → nextStep
    ├── Step2: SoulCheckboxList → fetch('/api/v1/souls')
    ├── Step3: TaskTextarea
    └── Step4: ConfigSummary → POST /api/v1/possess
        └── router.push → /possess/[sessionId]

SessionPage
└── SessionRunner
    └── useWebSocket(sessionId)
        ├── onMessage → messageAccumulator (useTransition)
        └── View dispatch: mode → {Single|Conference|Debate|Relay|Learn|PracticeOpening}View
```

## External Dependencies

无新增 — 全部使用 F1/F2 已有栈。
