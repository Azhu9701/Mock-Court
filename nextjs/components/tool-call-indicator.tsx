"use client";

import type { ToolCallEvent } from "@/hooks/use-websocket";
import { Wrench, Loader2, CheckCircle2, ChevronDown, ChevronUp } from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/utils";

interface ToolCallIndicatorProps {
  toolCall: ToolCallEvent;
}

function truncate(str: string, maxLen: number): string {
  if (str.length <= maxLen) return str;
  return str.slice(0, maxLen) + "...";
}

export function ToolCallIndicator({ toolCall }: ToolCallIndicatorProps) {
  const [expanded, setExpanded] = useState(false);
  const isDone = toolCall.status === "done";
  const argsPreview = truncate(toolCall.arguments, 80);

  return (
    <div
      className={cn(
        "my-1.5 rounded-lg border px-3 py-2 text-sm transition-colors",
        isDone
          ? "border-emerald-500/30 bg-emerald-500/5"
          : "border-amber-500/30 bg-amber-500/5"
      )}
    >
      <div className="flex items-center gap-2">
        {isDone ? (
          <CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-500" />
        ) : (
          <Loader2 className="h-4 w-4 shrink-0 animate-spin text-amber-500" />
        )}
        <Wrench className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        <span className="font-medium">{toolCall.toolName}</span>
        <span className="text-muted-foreground">
          {isDone ? "完成" : "执行中..."}
        </span>
        <button
          onClick={() => setExpanded(!expanded)}
          className="ml-auto shrink-0 text-muted-foreground hover:text-foreground"
        >
          {expanded ? (
            <ChevronUp className="h-3.5 w-3.5" />
          ) : (
            <ChevronDown className="h-3.5 w-3.5" />
          )}
        </button>
      </div>

      {!expanded && (
        <div className="mt-1 text-xs text-muted-foreground truncate">
          {argsPreview}
        </div>
      )}

      {expanded && (
        <div className="mt-2 space-y-2 text-xs">
          <div>
            <div className="font-medium text-muted-foreground mb-0.5">参数</div>
            <pre className="whitespace-pre-wrap break-all rounded bg-muted/50 p-2 text-xs font-mono">
              {toolCall.arguments}
            </pre>
          </div>
          {toolCall.result !== undefined && (
            <div>
              <div className="font-medium text-muted-foreground mb-0.5">结果</div>
              <pre className="whitespace-pre-wrap break-all rounded bg-muted/50 p-2 text-xs font-mono max-h-32 overflow-y-auto">
                {truncate(toolCall.result, 500)}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface ToolCallListProps {
  toolCalls: ToolCallEvent[];
  soulName?: string;
}

export function ToolCallList({ toolCalls, soulName }: ToolCallListProps) {
  const filtered = soulName
    ? toolCalls.filter((tc) => tc.soulName === soulName)
    : toolCalls;

  if (filtered.length === 0) return null;

  return (
    <div className="space-y-1">
      {filtered.map((tc) => (
        <ToolCallIndicator key={tc.toolCallId} toolCall={tc} />
      ))}
    </div>
  );
}
