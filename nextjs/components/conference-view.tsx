"use client";

import { useState } from "react";
import type { SoulMessage, CollisionEvent, CostInfo, ToolCallEvent } from "@/hooks/use-websocket";
import { SoulPanel } from "@/components/soul-panel";
import { SynthesisPanel } from "@/components/synthesis-panel";
import { CollisionNotification } from "@/components/collision-notification";
import { ToolCallList } from "@/components/tool-call-indicator";

interface ConferenceViewProps {
  messages: Record<string, SoulMessage>;
  synthesis: string;
  collisions: CollisionEvent[];
  cost: CostInfo | null;
  toolCalls: ToolCallEvent[];
}

export function ConferenceView({ messages, synthesis, collisions, cost, toolCalls }: ConferenceViewProps) {
  const names = Object.keys(messages);
  const [expandedSoul, setExpandedSoul] = useState<string | null>(null);

  const hasActiveCollisions = collisions.some(c => c.to === names.find(n => messages[n].isStreaming));
  const streamingCount = names.filter(n => messages[n].isStreaming).length;

  return (
    <div data-testid="conference-view" className="flex-1 flex flex-col h-full overflow-hidden">
      {/* 顶部信息栏 */}
      <div className="flex items-center justify-between px-4 py-2 border-b bg-muted/20">
        <div className="flex items-center gap-4">
          <span className="text-sm font-medium">合议模式</span>
          <span className="text-xs text-muted-foreground">{names.length} 魂参与</span>
          {hasActiveCollisions && (
            <span className="text-xs bg-amber-100 text-amber-700 px-2 py-0.5 rounded-full animate-pulse">
              交叉追问进行中
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {streamingCount > 0 && (
            <span className="text-xs text-muted-foreground">
              {streamingCount} 魂正在回应
            </span>
          )}
        </div>
      </div>

      {/* 工具调用通知 */}
      {toolCalls.length > 0 && (
        <div className="px-4 pt-3">
          <ToolCallList toolCalls={toolCalls} />
        </div>
      )}

      {/* 魂面板区 - 多列并行 */}
      <div className="flex-1 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3 p-3 overflow-hidden">
        {names.map((name) => {
          const isCurrentlyExpanded = expandedSoul === name;
          return (
            <div
              key={name}
              className={`transition-all duration-300 ${
                isCurrentlyExpanded ? "lg:col-span-2 xl:col-span-2" : ""
              }`}
              onClick={() => setExpandedSoul(isCurrentlyExpanded ? null : name)}
            >
              <SoulPanel
                name={name}
                content={messages[name].content}
                isStreaming={messages[name].isStreaming}
                error={messages[name].error}
                hasCollision={collisions.some(c => c.to === name || c.from === name)}
                ismismCode={messages[name].ismismCode || ""}
                isExpanded={isCurrentlyExpanded}
                onToggleExpand={() => setExpandedSoul(isCurrentlyExpanded ? null : name)}
              />
            </div>
          );
        })}
      </div>

      {/* 碰撞通知栏 - 实时弹出 */}
      {collisions.length > 0 && (
        <CollisionNotification collisions={collisions} />
      )}

      {/* 辩证综合面板 - 持续更新 */}
      <SynthesisPanel content={synthesis} cost={cost} />
    </div>
  );
}
