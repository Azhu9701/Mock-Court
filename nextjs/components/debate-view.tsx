"use client";

import type { SoulMessage } from "@/hooks/use-websocket";
import { SoulChatBubble } from "@/components/soul-chat-bubble";

interface DebateViewProps {
  messages: Record<string, SoulMessage>;
}

export function DebateView({ messages }: DebateViewProps) {
  const entries = Object.values(messages);
  const hasStreaming = entries.some((m) => m.isStreaming);

  return (
    <div
      data-testid="debate-view"
      className="flex flex-col flex-1 overflow-hidden"
    >
      {entries.map((msg, i) => (
        <div key={msg.soulName} className="flex-1 overflow-y-auto border-b last:border-b-0 p-4 min-h-0">
          <div className="text-xs font-semibold text-muted-foreground mb-2">
            {i === 0 ? "正方" : "反方"} · {msg.soulName}
          </div>
          <SoulChatBubble message={msg} autoScroll={hasStreaming} />
        </div>
      ))}
    </div>
  );
}
