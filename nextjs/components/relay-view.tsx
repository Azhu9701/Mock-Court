"use client";

import type { SoulMessage } from "@/hooks/use-websocket";
import { SoulChatBubble } from "@/components/soul-chat-bubble";

interface RelayViewProps {
  messages: Record<string, SoulMessage>;
}

export function RelayView({ messages }: RelayViewProps) {
  const entries = Object.values(messages);
  const hasStreaming = entries.some((m) => m.isStreaming);

  return (
    <div data-testid="relay-view" className="space-y-4 max-w-3xl mx-auto">
      {entries.map((msg, i) => (
        <div key={msg.soulName}>
          <div className="text-xs text-muted-foreground mb-1 px-1">
            第 {i + 1} 棒 · {msg.soulName}
          </div>
          <SoulChatBubble message={msg} autoScroll={hasStreaming} />
        </div>
      ))}
    </div>
  );
}
