"use client";

import { SoulChatBubble } from "@/components/soul-chat-bubble";
import type { SoulMessage } from "@/hooks/use-websocket";

interface SoulFocusPanelProps {
  soulName: string;
  message: SoulMessage;
}

export function SoulFocusPanel({ soulName, message }: SoulFocusPanelProps) {
  return (
    <div className="flex-1 overflow-y-auto p-4" data-testid="soul-focus-panel">
      <SoulChatBubble message={message} autoScroll={message.isStreaming} />
    </div>
  );
}
