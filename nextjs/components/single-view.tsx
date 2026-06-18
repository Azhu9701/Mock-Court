"use client";

import { SoulChatBubble } from "@/components/soul-chat-bubble";
import type { SoulMessage } from "@/hooks/use-websocket";

interface SingleViewProps {
  messages: Record<string, SoulMessage>;
}

export function SingleView({ messages }: SingleViewProps) {
  const entries = Object.values(messages);
  const hasStreaming = entries.some((m) => m.isStreaming);

  return (
    <div data-testid="single-view" className="max-w-3xl mx-auto space-y-2">
      {entries.map((msg) => (
        <SoulChatBubble
          key={msg.soulName}
          message={msg}
          autoScroll={hasStreaming}
        />
      ))}
      {entries.length === 0 && (
        <p className="text-center text-muted-foreground py-10">
          等待角色回应...
        </p>
      )}
    </div>
  );
}
