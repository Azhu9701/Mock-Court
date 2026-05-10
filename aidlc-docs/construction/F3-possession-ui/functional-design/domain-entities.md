# Domain Entities — F3: Possession UI

## Wizard State (Q2: B — 分步向导)

```typescript
// 步骤流: Mode → Souls → Task → Review → Start
type WizardStep = 'mode' | 'souls' | 'task' | 'review' | 'running';

interface PossessionWizardState {
  step: WizardStep;
  mode: PossessionMode | null;      // 选中的模式
  souls: string[];                   // 选中的魂名列表
  task: string;                      // 任务描述
  topic: string;                     // 辩论主题
}

// 6 种模式
type PossessionMode = 'single' | 'conference' | 'debate' | 'relay' | 'learn' | 'practice_opening';
```

## Session State

```typescript
interface PossessionSession {
  sessionId: string;
  mode: PossessionMode;
  task: string;
  wsUrl: string;
  status: 'connecting' | 'streaming' | 'done' | 'error';
  startedAt: Date;
}
```

## Stream Event Types (from WS)

```typescript
// WsEvent from possession crate
interface WsEvent {
  event_type: 'SoulChunk' | 'SoulDone' | 'SoulError' | 'SynthesisChunk' | 'SynthesisDone' | 'SessionComplete';
  payload: string;
  soul_name: string | null;
  seq: number;
}
```

## UI Display Types

### SoulMessage — 聊天气泡 (Q1: B)

```typescript
interface SoulMessage {
  soulName: string;
  content: string;        // 累积内容
  chunks: string[];       // 流式 chunk 历史
  isStreaming: boolean;   // 是否仍在接收
  error: string | null;
  timestamp: Date;
}
```

### ConferenceView — 主副面板 (Q3: B)

```typescript
interface ConferenceViewState {
  focusSoul: string | null;          // 当前聚焦的魂
  souls: Record<string, SoulMessage>; // 各魂消息
  synthesis: string | null;           // 辩证综合结果
  layout: 'focus' | 'overview';       // 主面板 / 概览
}
```

### DebateView — 上下分屏 (Q4: B)

```typescript
interface DebateViewState {
  soulA: SoulMessage;
  soulB: SoulMessage;
  verdict: string | null;
}
```

## API Types

### StartPossessionRequest

```typescript
interface StartPossessionRequest {
  mode?: string;
  task: string;
  souls: string[];
  topic?: string;
}
```

### StartPossessionResponse

```typescript
interface StartPossessionResponse {
  session_id: string;
  ws_url: string;
  mode: string;
}
```

## Relations

```
/possess
├── PossessionWizard (Step 1-4)
│   ├── StepMode — 选择模式 (6种)
│   ├── StepSouls — 选择魂 (最多10个)
│   ├── StepTask — 输入任务描述
│   └── StepReview — 确认配置
└── PossessionRunner (Step 5)
    ├── WS Connection → B6 /ws/possess/:sessionId/main
    ├── SingleView — 单魂聊天 (Q1: B)
    ├── ConferenceView — 主副面板 (Q3: B)
    ├── DebateView — 上下分屏 (Q4: B)
    ├── RelayView — 接力链
    ├── LearnView — 学习模式
    └── PracticeOpeningView — 实践开口 Wizard
```
