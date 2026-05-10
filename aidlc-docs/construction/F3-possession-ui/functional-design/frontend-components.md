# Frontend Components — F3: Possession UI

## Component Hierarchy

```
PossessPage
├── PossessionWizard (Client, 5 steps)
│   ├── StepModeSelector
│   │   └── ModeCard[] (6 modes)
│   ├── StepSoulPicker
│   │   ├── SoulSearchInput
│   │   └── SoulCheckboxList
│   │       └── SoulCheckboxItem[]
│   ├── StepTaskInput
│   │   └── TaskTextarea + TopicInput (debate only)
│   └── StepReview
│       ├── ConfigSummary
│       └── StartButton → POST /possess

PossessionSessionPage
├── WebSocketProvider (Context)
│   └── SessionRunner
│       ├── SingleView — 单魂聊天
│       │   ├── SoulChatBubble[]
│       │   └── FollowUpInput
│       ├── ConferenceView — 主副面板
│       │   ├── SoulOverviewPanel
│       │   │   └── SoulStatusItem[]
│       │   ├── SoulFocusPanel
│       │   │   └── SoulChatBubble[]
│       │   └── SynthesisPanel
│       ├── DebateView — 上下分屏
│       │   ├── DebatePane (Top — Soul A)
│       │   ├── ResizeHandle
│       │   ├── DebatePane (Bottom — Soul B)
│       │   └── VerdictPanel
│       ├── RelayView — 接力链
│       ├── LearnView — 学习模式
│       └── PracticeOpeningView — 实践开口
└── SessionStatusBar (连接状态、耗时)
```

## Key Components

### PossessionWizard

```typescript
interface PossessionWizardProps {
  preset?: { mode?: string; souls?: string[] };
}

// State
const [step, setStep] = useState<WizardStep>('mode');
const [mode, setMode] = useState<PossessionMode | null>(null);
const [souls, setSouls] = useState<string[]>([]);
const [task, setTask] = useState('');
const [topic, setTopic] = useState('');

// Submit
async function onStart() {
  const res = await fetch('/api/v1/possess', {
    method: 'POST',
    body: JSON.stringify({ mode, task, souls, topic }),
  });
  const { session_id } = await res.json();
  router.push(`/possess/${session_id}`);
}
```

### ModeCard

```typescript
interface ModeCardProps {
  mode: PossessionMode;
  label: string;
  description: string;
  icon: string;
  selected: boolean;
  onSelect: () => void;
}
```

- 6 张卡片：Single / Conference / Debate / Relay / Learn / PracticeOpening
- 选中态：border-primary + bg-primary/5
- 卡片描述模式特点和魂数量要求

### SoulCheckboxList

```typescript
interface SoulCheckboxListProps {
  souls: SoulListEntry[];
  selected: string[];
  maxSelect: number;
  onToggle: (name: string) => void;
}
```

- 从 `/api/v1/souls` 获取列表
- 搜索过滤
- 超出 maxSelect 时禁用未选中项
- 调用 F2 的 fetchSouls() API helper

### WebSocketProvider

```typescript
interface WebSocketProviderProps {
  sessionId: string;
  wsUrl: string;
  children: React.ReactNode;
}

// Context value
interface WsContextValue {
  messages: Record<string, SoulMessage>;
  synthesis: string;
  status: 'connecting' | 'streaming' | 'done' | 'error';
  error: string | null;
  reconnect: () => void;
}
```

- 管理 WS 连接生命周期
- 解析 WsEvent 更新 messages
- 断线自动重连 (exponential backoff, max 3)

### SoulChatBubble

```typescript
interface SoulChatBubbleProps {
  soulName: string;
  content: string;
  isStreaming: boolean;
  error: string | null;
}
```

- 左侧气泡布局
- 头像：魂名首字（圆形背景）
- streaming 时显示闪烁光标
- error 时显示红色边框

### SoulOverviewPanel (Conference)

```typescript
interface SoulOverviewPanelProps {
  souls: Record<string, SoulMessage>;
  focusSoul: string | null;
  onFocus: (name: string) => void;
}
```

- 240px 宽侧面板
- 魂列表：名称 + 状态图标 (○/●/✓/✗)
- 单击切换 focusSoul

### SoulFocusPanel (Conference)

```typescript
interface SoulFocusPanelProps {
  soulName: string;
  message: SoulMessage;
}
```

- 主展示区
- 完整聊天气泡
- 自动滚动到底部

### DebatePane

```typescript
interface DebatePaneProps {
  soulName: string;
  role: 'affirmative' | 'negative';
  message: SoulMessage;
}
```

- 上下分屏各占一个 Pane
- 标题栏显示魂名 + 立场

## Files Created

```
app/possess/
├── page.tsx                    # PossessionWizard
├── layout.tsx                  # 独立布局 (无侧栏)
└── [sessionId]/
    └── page.tsx                # SessionRunner

components/
├── possession-wizard.tsx       # 5-step Wizard
├── mode-card.tsx               # 模式卡片
├── soul-checkbox-list.tsx      # 魂选择器
├── session-runner.tsx          # 按模式分发视图
├── websocket-provider.tsx      # WS Context
├── soul-chat-bubble.tsx        # 聊天气泡
├── single-view.tsx             # 单魂视图
├── conference-view.tsx         # 主副面板
├── soul-overview-panel.tsx     # 概览侧面板
├── soul-focus-panel.tsx        # 聚焦主面板
├── synthesis-panel.tsx         # 辩证综合面板
├── debate-view.tsx             # 上下分屏
├── verdict-panel.tsx           # 裁决面板
├── relay-view.tsx              # 接力视图
├── learn-view.tsx              # 学习视图
├── practice-opening-view.tsx   # 实践开口
└── session-status-bar.tsx      # 状态栏

config/
└── possession-modes.ts         # 模式配置常量
```
