# 前端流式组件复用

## 核心 Hook：useWebSocket

```typescript
// 用法
const { 
  agentOutputs,    // Record<string, string> — 每个 Agent 的累计输出
  agentStatus,     // Record<string, "streaming" | "done" | "error">
  synthesis,       // string — 综合内容
  collisions,      // Collision[] — 碰撞列表
  cost,            // { totalTokens, estimatedCost }
  status,          // "connecting" | "streaming" | "done" | "error"
  intervene,       // (message: string) => void — 发送干预消息
} = useWebSocket(sessionId);
```

## 预置视图组件

| 组件 | 用途 | 导入 |
|------|------|------|
| `AgentChatBubble` | 单个 Agent 流式气泡，含思考过程展开 | `@/components/agent/agent-chat-bubble` |
| `AgentMultiColumn` | 多 Agent 并排布局（Conference 模式） | `@/components/agent/agent-multi-column` |
| `AgentDebateLayout` | 两栏对立布局（Debate 模式） | `@/components/agent/agent-debate-layout` |
| `SynthesisPanel` | 综合输出面板 | `@/components/agent/synthesis-panel` |
| `CollisionNotification` | 碰撞提示浮窗 | `@/components/agent/collision-notification` |
| `ToolCallIndicator` | 工具调用状态指示器 | `@/components/agent/tool-call-indicator` |
| `CostDisplay` | Token 消耗实时显示 | `@/components/agent/cost-display` |
| `PossessionEntry` | 完整入口组件（输入→分析→确认→执行） | `@/components/agent/possession-entry` |

## 最小集成示例

```tsx
// app/consult/page.tsx
"use client";

import { useWebSocket } from "@/hooks/use-websocket";
import { AgentMultiColumn } from "@/components/agent/agent-multi-column";
import { SynthesisPanel } from "@/components/agent/synthesis-panel";
import { CollisionNotification } from "@/components/agent/collision-notification";

export default function ConsultPage({ params }: { params: { id: string } }) {
  const {
    agentOutputs,
    agentStatus,
    synthesis,
    collisions,
    status,
    intervene,
  } = useWebSocket(params.id);

  return (
    <div className="flex flex-col gap-4 p-4">
      {/* Agent 多列视图 */}
      <AgentMultiColumn 
        outputs={agentOutputs} 
        status={agentStatus} 
      />

      {/* 碰撞通知 */}
      {collisions.map((c, i) => (
        <CollisionNotification key={i} collision={c} />
      ))}

      {/* 综合面板 */}
      {synthesis && <SynthesisPanel content={synthesis} />}

      {/* 干预输入 */}
      {status === "streaming" && (
        <input
          placeholder="追问或纠正..."
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              intervene(e.currentTarget.value);
              e.currentTarget.value = "";
            }
          }}
        />
      )}
    </div>
  );
}
```

## 自定义样式

所有组件接受 `className` prop，支持 Tailwind CSS 覆盖：

```tsx
<AgentChatBubble 
  agentName="分析师"
  content={output}
  className="bg-blue-50 border-blue-200"
/>
```

## 事件总线

跨组件通信通过自定义事件：

```typescript
// 监听会话更新事件
window.addEventListener("SESSIONS_UPDATED", () => {
  // 刷新会话列表
});

// 发送通知（由 PossessionEntry 等组件发出）
window.dispatchEvent(new CustomEvent("SESSIONS_UPDATED"));
```

## 完整页面示例

参考原项目中的页面路由：

| 路由 | 页面 | 参考价值 |
|------|------|---------|
| `/possess` | 模式选择 → 输入 → 分析 → 确认 | 完整入口流程 |
| `/possess/[sessionId]` | 会话进行中（多列视图） | 多 Agent 实时渲染 |
| `/sessions/[id]` | 会话回顾 | 历史回放 |
| `/souls/[name]` | Agent 详情 | Agent 卡片展示 |
