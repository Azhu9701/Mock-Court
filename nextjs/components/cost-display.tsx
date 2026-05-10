"use client";

import { DollarSign, Zap } from "lucide-react";
import type { CostInfo } from "@/hooks/use-websocket";

interface CostDisplayProps {
  cost: CostInfo | null;
}

export function CostDisplay({ cost }: CostDisplayProps) {
  if (!cost) return null;

  return (
    <div className="flex items-center gap-4 text-xs text-muted-foreground bg-muted/50 rounded-lg px-3 py-2">
      <div className="flex items-center gap-1">
        <Zap className="h-3 w-3" />
        <span>调用: {cost.llm_calls}</span>
      </div>
      <div className="flex items-center gap-1">
        <span>Tokens: {cost.tokens_used}</span>
      </div>
      <div className="flex items-center gap-1">
        <DollarSign className="h-3 w-3" />
        <span>预估: {cost.estimated_cost}</span>
      </div>
    </div>
  );
}
