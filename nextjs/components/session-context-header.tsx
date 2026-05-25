"use client";

import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Button } from "@/components/ui/button";
import { Sparkles, ShieldCheck, ChevronUp, ChevronDown, ArrowRightCircle } from "lucide-react";
import { MODE_LABELS_LONG } from "@/config/possession-modes";

/** 过滤 AI thinking / reasoning 内容 */
function stripThinking(text: string): string {
  if (!text) return "";
  // 匹配 "Here's a thinking process:..." 到下一个明显分界或结尾
  return text
    .replace(/Here's a thinking process:[\s\S]*?(?=\n\n[A-Z]|\n#[^#]|$)/gi, "")
    .replace(/<thinking>[\s\S]*?<\/thinking>/gi, "")
    .replace(/Thinking:[\s\S]*?(?=\n\n[A-Z]|$)/gi, "")
    .trim();
}

/** Markdown 渲染容器，复用 prose 样式 */
function Md({ children }: { children: string }) {
  const cleaned = stripThinking(children);
  if (!cleaned) return null;
  return (
    <span className="prose prose-slate prose-sm max-w-none [&_p]:my-1 [&_strong]:font-semibold [&_ul]:my-1 [&_ol]:my-1 [&_li]:my-0.5]">
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{cleaned}</ReactMarkdown>
    </span>
  );
}

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
                  <div className="text-muted-foreground mt-2 text-sm leading-relaxed">
                    <Md>{s.rationale}</Md>
                  </div>
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
