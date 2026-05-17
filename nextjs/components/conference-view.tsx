"use client";

import { useState, useMemo, useCallback } from "react";
import type { SoulMessage, CollisionEvent, ToolCallEvent } from "@/hooks/use-websocket";
import { SoulPanel } from "@/components/soul-panel";
import { SynthesisSection } from "@/components/synthesis-section";
import { CollisionNotification } from "@/components/collision-notification";
import { ToolCallList } from "@/components/tool-call-indicator";
import { ArticleModal } from "@/components/article-modal";

interface ConferenceViewProps {
  messages: Record<string, SoulMessage>;
  synthesis: string;
  collisions: CollisionEvent[];
  toolCalls: ToolCallEvent[];
}

export function ConferenceView({ messages, synthesis, collisions, toolCalls }: ConferenceViewProps) {
  const names = useMemo(() => Object.keys(messages), [messages]);
  const [focusedSoul, setFocusedSoul] = useState<string | null>(null);

  const hasActiveCollisions = useMemo(
    () => collisions.some(c => c.to === names.find(n => messages[n].isStreaming)),
    [collisions, names, messages]
  );
  const streamingCount = useMemo(
    () => names.filter(n => messages[n].isStreaming).length,
    [names, messages]
  );

  const openModal = useCallback((name: string) => {
    setFocusedSoul(name);
  }, []);
  const closeModal = useCallback(() => {
    setFocusedSoul(null);
  }, []);

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
        {names.map((name) => (
          <button
            type="button"
            key={name}
            className="w-full h-full text-left transition-all duration-300"
            onClick={() => openModal(name)}
            onKeyDown={(e) => {
              if (e.nativeEvent.isComposing || e.keyCode === 229) return;
              if (e.key === "Enter" || e.key === " ") { e.preventDefault(); openModal(name); }
            }}
            aria-label={`${name} 面板 — 点击展开沉浸阅读`}
          >
            <SoulPanel
              name={name}
              content={messages[name].content}
              isStreaming={messages[name].isStreaming}
              error={messages[name].error}
              hasCollision={collisions.some(c => c.to === name || c.from === name)}
              ismismCode={messages[name].ismismCode || ""}
              isExpanded={false}
            />
          </button>
        ))}
      </div>

      {/* 碰撞通知栏 - 实时弹出 */}
      {collisions.length > 0 && (
        <CollisionNotification collisions={collisions} />
      )}

      {/* 辩证综合 — 与历史详情页统一卡片样式 */}
      {synthesis && (
        <SynthesisSection messages={[{ id: "synthesis", content: synthesis, created_at: new Date().toISOString() }]} />
      )}

      {/* 沉浸阅读 modal — 点击卡片后弹出 */}
      {focusedSoul && messages[focusedSoul] && (
        <ArticleModal
          isOpen={!!focusedSoul}
          onClose={closeModal}
          title={focusedSoul}
          ismismCode={messages[focusedSoul].ismismCode || ""}
          content={messages[focusedSoul].content}
        />
      )}
    </div>
  );
}
