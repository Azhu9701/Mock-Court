# NFR Design Patterns — F3: Possession UI

## Pattern 1: WS Reconnect with Exponential Backoff

**问题**: WS 连接可能因网络波动断开，需要自动恢复。

**方案**: onclose 触发定时重连，指数退避。

```typescript
function useWebSocket(sessionId: string) {
  const retryRef = useRef(0);
  const MAX_RETRIES = 3;

  const connect = useCallback(() => {
    const url = `ws://127.0.0.1:3096/ws/possess/${sessionId}/main`;
    const ws = new WebSocket(url);
    ws.onopen = () => { retryRef.current = 0; };
    ws.onmessage = (e) => handleEvent(JSON.parse(e.data));
    ws.onclose = () => {
      if (retryRef.current < MAX_RETRIES) {
        const delay = Math.pow(2, retryRef.current) * 1000;
        setTimeout(connect, delay);
        retryRef.current++;
      }
    };
  }, [sessionId]);
}
```

## Pattern 2: useTransition Batch Rendering (Q2: B)

**问题**: 10 魂并行流式输出每秒数十个 chunk，直接 setState 造成卡顿。

**方案**: 用 ref 缓冲数据，useTransition 批量提交。

```typescript
const bufferRef = useRef<Record<string, string>>({});
const [messages, setMessages] = useState<Record<string, string>>({});
const [isPending, startTransition] = useTransition();

function onChunk(soulName: string, content: string) {
  bufferRef.current[soulName] = (bufferRef.current[soulName] || '') + content;
  startTransition(() => {
    setMessages({ ...bufferRef.current });
  });
}
```

## Pattern 3: Auto-scroll Only During Streaming (Q3: C)

**问题**: 需要在合适的时机自动滚动，不干扰用户阅读。

**方案**: streaming 状态控制 auto-scroll 行为。

```typescript
const bottomRef = useRef<HTMLDivElement>(null);
const [isStreaming, setIsStreaming] = useState(true);

useEffect(() => {
  if (isStreaming && bottomRef.current) {
    bottomRef.current.scrollIntoView({ behavior: 'smooth' });
  }
}, [messages, isStreaming]);
```

## Pattern 4: Mode-based View Dispatch

**问题**: 6 种模式需要不同的 UI 布局。

**方案**: mode → component mapping（策略模式）。

```typescript
const viewMap: Record<string, React.ComponentType<SessionViewProps>> = {
  single: SingleView,
  conference: ConferenceView,
  debate: DebateView,
  relay: RelayView,
  learn: LearnView,
  practice_opening: PracticeOpeningView,
};

function SessionRunner({ mode, sessionId }: Props) {
  const View = viewMap[mode] || SingleView;
  return <View sessionId={sessionId} />;
}
```

## Pattern 5: Message Accumulator

**问题**: 流式 chunk 需要累积为完整消息。

**方案**: useRef accumulator + useTransition 渲染。

```typescript
function useMessageAccumulator() {
  const accumRef = useRef<Record<string, SoulMessage>>({});
  const [messages, setMessages] = useState<typeof accumRef.current>({});
  const [, startTransition] = useTransition();

  function onChunk(soulName: string, chunk: string) {
    if (!accumRef.current[soulName]) {
      accumRef.current[soulName] = { soulName, content: '', isStreaming: true };
    }
    accumRef.current[soulName].content += chunk;
    startTransition(() => setMessages({ ...accumRef.current }));
  }

  function onDone(soulName: string) {
    accumRef.current[soulName].isStreaming = false;
    startTransition(() => setMessages({ ...accumRef.current }));
  }

  return { messages, onChunk, onDone };
}
```
