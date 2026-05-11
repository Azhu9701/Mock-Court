"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Sparkles, ShieldCheck, ChevronUp, ChevronDown, ArrowRightCircle } from "lucide-react";
import { MODE_LABELS_LONG } from "@/config/possession-modes";

export interface MatchedSoulInfo {
  name: string;
  field: string;
  ismism_code: string;
  rationale: string;
}

export interface ReviewResult {
  verdict: string;
  checks: string[];
  notes: string;
  reviewer: string;
}

interface SessionContextHeaderProps {
  task: string;
  mode: string;
  matchedSouls: MatchedSoulInfo[];
  review: ReviewResult | null;
}

function getVerdictLabel(v: string) {
  const labels: Record<string, string> = {
    "pass": "✅ 通过",
    "conditional": "⚠️ 条件通过",
    "reject": "❌ 拒绝"
  };
  return labels[v] || v;
}

export function SessionContextHeader({ task, mode, matchedSouls, review }: SessionContextHeaderProps) {
  const [showDetail, setShowDetail] = useState(true);

  return (
    <div className="rounded-xl border bg-gradient-to-br from-muted/40 to-background p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">{task}</h2>
          <p className="text-sm text-muted-foreground mt-1 flex items-center gap-2">
            <span>模式：{(MODE_LABELS_LONG as Record<string, string>)[mode] || mode}</span>
            <span>·</span>
            <span>{matchedSouls.length} 魂</span>
            {review && (
              <>
                <span>·</span>
                <span>审查：{review.reviewer}</span>
                <span className={review.verdict === "pass" ? "text-green-600" : review.verdict === "conditional" ? "text-yellow-600" : "text-red-600"}>
                  [{review.verdict}]
                </span>
              </>
            )}
          </p>
        </div>
        <Button variant="ghost" size="sm" onClick={() => setShowDetail(!showDetail)} className="transition-all hover:bg-muted">
          {showDetail ? <ChevronUp className="h-4 w-4 mr-1" /> : <ChevronDown className="h-4 w-4 mr-1" />}
          {showDetail ? "收起" : "详情"}
        </Button>
      </div>

      {showDetail && (
        <div className="mt-4 space-y-4 text-sm border-t pt-4">
          <div>
            <h4 className="font-medium text-muted-foreground mb-3 flex items-center gap-2">
              <Sparkles className="h-4 w-4" />
              匹配魂
            </h4>
            <div className="grid gap-3">
              {matchedSouls.map((s) => (
                <div key={s.name} className="rounded-lg border p-3 bg-background transition-all hover:shadow-sm">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-semibold text-base">{s.name}</span>
                    <span className="text-xs bg-muted px-2 py-0.5 rounded">{s.field}</span>
                    <span className="text-xs text-muted-foreground font-mono">{s.ismism_code}</span>
                  </div>
                  <p className="text-muted-foreground mt-2 text-sm leading-relaxed">{s.rationale}</p>
                </div>
              ))}
            </div>
          </div>

          {review && (
            <div>
              <h4 className="font-medium text-muted-foreground mb-3 flex items-center gap-2">
                <ShieldCheck className="h-4 w-4" />
                审查 · {review.reviewer}
              </h4>
              <div className={`rounded-lg border p-3 ${
                review.verdict === "pass" ? "border-green-200 bg-green-50 dark:bg-green-950/20" :
                review.verdict === "conditional" ? "border-yellow-200 bg-yellow-50 dark:bg-yellow-950/20" :
                "border-red-200 bg-red-50 dark:bg-red-950/20"
              }`}>
                <div className="font-medium mb-2">裁决: {getVerdictLabel(review.verdict)}</div>
                <ul className="space-y-1">
                  {review.checks.map((c, i) => (
                    <li key={i} className="text-sm flex items-start gap-2">
                      <ArrowRightCircle className="h-4 w-4 mt-0.5 shrink-0 text-muted-foreground" />
                      <span>{c}</span>
                    </li>
                  ))}
                </ul>
                {review.notes && (
                  <p className="text-sm mt-2 italic text-muted-foreground border-t pt-2">
                    📝 {review.notes}
                  </p>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
