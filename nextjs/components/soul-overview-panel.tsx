"use client";

import type { SoulMessage } from "@/hooks/use-websocket";

interface SoulOverviewPanelProps {
  souls: Record<string, SoulMessage>;
  focusSoul: string | null;
  onFocus: (name: string) => void;
}

export function SoulOverviewPanel({
  souls,
  focusSoul,
  onFocus,
}: SoulOverviewPanelProps) {
  const entries = Object.entries(souls);

  return (
    <div
      className="w-60 shrink-0 border-r overflow-y-auto p-2 space-y-1"
      data-testid="soul-overview-panel"
    >
      <h3 className="text-xs font-semibold text-muted-foreground px-2 mb-2">
        魂列表
      </h3>
      {entries.map(([name, msg]) => (
        <button
          key={name}
          onClick={() => onFocus(name)}
          className={`w-full flex items-center gap-2 rounded-md px-3 py-2 text-sm text-left transition-colors
            ${
              focusSoul === name
                ? "bg-primary/10 text-primary font-medium"
                : "hover:bg-muted"
            }`}
          data-testid={`overview-soul-${name}`}
        >
          <span className="w-4 text-center text-xs">
            {msg.error ? "✗" : msg.isStreaming ? "●" : msg.content ? "✓" : "○"}
          </span>
          <span className="truncate">{name}</span>
        </button>
      ))}
    </div>
  );
}
