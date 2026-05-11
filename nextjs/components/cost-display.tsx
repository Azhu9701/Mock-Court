"use client";

import { DollarSign, Zap } from "lucide-react";
import type { CostInfo } from "@/hooks/use-websocket";

interface CostDisplayProps {
  cost: CostInfo | null;
}

export function CostDisplay({ cost }: CostDisplayProps) {
  if (!cost) return null;

  return (
    <div className="flex flex-col gap-2 text-xs text-muted-foreground bg-muted/50 rounded-lg px-3 py-2">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-1">
          <Zap className="h-3 w-3" />
          <span>调用: {cost.llm_calls}</span>
        </div>
        <div className="flex items-center gap-1">
          <span>Tokens: {cost.tokens_used.toLocaleString()}</span>
        </div>
        {cost.estimated_cost && (
          <div className="flex items-center gap-1">
            <DollarSign className="h-3 w-3" />
            <span>预估: {cost.estimated_cost}</span>
          </div>
        )}
      </div>
      {cost.per_soul && cost.per_soul.length > 0 && (
        <div className="border-t border-muted pt-1.5 mt-0.5">
          <div className="grid grid-cols-1 gap-1">
            {cost.per_soul.map((s) => (
              <div key={s.soul_name} className="flex items-center gap-2 text-[11px]">
                <span className="font-medium min-w-[60px] truncate">{s.soul_name}</span>
                <span className="text-muted-foreground/70">
                  P:{s.prompt_tokens.toLocaleString()} C:{s.completion_tokens.toLocaleString()} T:{s.total_tokens.toLocaleString()}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
