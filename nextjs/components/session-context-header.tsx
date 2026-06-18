"use client";

import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Button } from "@/components/ui/button";
import { Sparkles, ChevronUp, ChevronDown } from "lucide-react";
import { MODE_LABELS_LONG } from "@/config/possession-modes";
import { useDomain } from "@/contexts/domain-context";

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


interface SessionContextHeaderProps {
  task: string;
  mode: string;
  matchedSouls: MatchedSoulInfo[];
}


export function SessionContextHeader({ task, mode, matchedSouls }: SessionContextHeaderProps) {
  const [showDetail, setShowDetail] = useState(true);
  const { agentNoun } = useDomain();

  return (
    <div className="rounded-xl border bg-gradient-to-br from-muted/40 to-background p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">{task}</h2>
          <p className="text-sm text-muted-foreground mt-1 flex items-center gap-2">
            <span>模式：{(MODE_LABELS_LONG as Record<string, string>)[mode] || mode}</span>
            <span>·</span>
            <span>{matchedSouls.length} {agentNoun}</span>
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
              匹配{agentNoun}
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

        </div>
      )}
    </div>
  );
}
