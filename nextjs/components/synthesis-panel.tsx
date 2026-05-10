"use client";

import { useState, useMemo } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { CostDisplay } from "@/components/cost-display";
import { Brain, ChevronDown, ChevronUp, CircleCheck, AlertTriangle, HelpCircle, Zap, Sparkles } from "lucide-react";
import type { CostInfo } from "@/hooks/use-websocket";

interface SynthesisPanelProps {
  content: string;
  cost?: CostInfo | null;
}

function cleanContent(raw: string): string {
  return raw.replace(/<[^>]+>/g, "").trim();
}

interface Section {
  title: string;
  items: string[];
}

function extractSections(raw: string): Section[] {
  const cleaned = cleanContent(raw);
  const sections: Section[] = [];
  const h3Parts = cleaned.split(/^###\s*/m).slice(1);

  for (const part of h3Parts) {
    const lines = part.split("\n");
    const title = lines[0].replace(/\*\*/g, "").trim();
    const items: string[] = [];
    const liRe = /^[-*]\s+(.+)/;
    const boldLiRe = /^[-*]\s+\*\*(.+?)\*\*[：:]\s*(.+)/;

    for (const line of lines.slice(1)) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("---") || trimmed.startsWith("###")) continue;

      const boldMatch = boldLiRe.exec(trimmed);
      if (boldMatch) {
        items.push(`**${boldMatch[1]}**：${boldMatch[2]}`);
        continue;
      }
      const liMatch = liRe.exec(trimmed);
      if (liMatch) {
        const text = liMatch[1].replace(/\*\*/g, "").trim();
        if (text.length > 5) items.push(text);
      }
    }
    if (title) sections.push({ title, items });
  }
  return sections;
}

interface SectionStyle {
  icon: React.ElementType;
  color: string;
  borderColor: string;
  bgClass: string;
}

function getSectionStyle(title: string): SectionStyle {
  const t = title.toLowerCase();
  if (t.includes("共识")) return { icon: CircleCheck, color: "text-emerald-500", borderColor: "border-emerald-300/50", bgClass: "bg-emerald-50/50 dark:bg-emerald-950/20" };
  if (t.includes("分歧")) return { icon: AlertTriangle, color: "text-amber-500", borderColor: "border-amber-300/50", bgClass: "bg-amber-50/50 dark:bg-amber-950/20" };
  if (t.includes("盲区")) return { icon: HelpCircle, color: "text-blue-500", borderColor: "border-blue-300/50", bgClass: "bg-blue-50/50 dark:bg-blue-950/20" };
  if (t.includes("矛盾")) return { icon: Zap, color: "text-red-500", borderColor: "border-red-300/50", bgClass: "bg-red-50/50 dark:bg-red-950/20" };
  if (t.includes("行动") || t.includes("纲领")) return { icon: Sparkles, color: "text-purple-500", borderColor: "border-purple-300/50", bgClass: "bg-purple-50/50 dark:bg-purple-950/20" };
  return { icon: CircleCheck, color: "text-muted-foreground", borderColor: "border-border", bgClass: "" };
}

const TOTAL_DIMENSIONS = 5;

export function SynthesisPanel({ content, cost }: SynthesisPanelProps) {
  const [expanded, setExpanded] = useState(true);
  const [collapsedSections, setCollapsedSections] = useState<Set<number>>(new Set());
  const [showRaw, setShowRaw] = useState(false);

  const sections = useMemo(() => (content ? extractSections(content) : []), [content]);
  const cleanedContent = useMemo(() => (content ? cleanContent(content) : ""), [content]);

  const sectionCount = sections.length;
  const progress = content ? Math.min(Math.round((sectionCount / TOTAL_DIMENSIONS) * 100), 100) : 0;

  const toggleSection = (idx: number) => {
    setCollapsedSections((prev) => {
      const next = new Set(prev);
      if (next.has(idx)) next.delete(idx);
      else next.add(idx);
      return next;
    });
  };

  if (!content && !cost) return null;

  return (
    <div className="border-t bg-background" data-testid="synthesis-panel">
      <div className="flex items-center justify-between px-4 py-3">
        <div className="flex items-center gap-2">
          <Brain className="h-4 w-4 text-primary" />
          <h3 className="text-sm font-semibold text-primary">辩证综合</h3>
        </div>
        <div className="flex items-center gap-3">
          <CostDisplay cost={cost ?? null} />
          <button
            onClick={() => setExpanded(!expanded)}
            className="p-1 rounded hover:bg-muted transition-colors"
          >
            {expanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
          </button>
        </div>
      </div>

      {expanded && (
        <div className="px-4 pb-4 space-y-3">
          {content && (
            <div className="rounded-lg bg-primary/5 p-3">
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-medium text-muted-foreground">综合进度</span>
                <span className="text-xs font-medium text-primary">{sectionCount}/{TOTAL_DIMENSIONS} 维度</span>
              </div>
              <div className="h-2 bg-muted rounded-full overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-primary to-emerald-500 transition-all duration-500"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          )}

          {sections.length > 0 ? (
            <div className="space-y-2">
              {sections.map((section, idx) => {
                const style = getSectionStyle(section.title);
                const Icon = style.icon;
                const isCollapsed = collapsedSections.has(idx);

                return (
                  <div key={idx} className="rounded-lg border border-border/50 overflow-hidden">
                    <button
                      onClick={() => toggleSection(idx)}
                      className={`w-full flex items-center gap-2 px-3 py-2 hover:bg-muted/50 transition-colors ${style.bgClass}`}
                    >
                      <Icon className={`h-3.5 w-3.5 shrink-0 ${style.color}`} />
                      <span className="text-xs font-semibold text-foreground/80">{section.title}</span>
                      {section.items.length > 0 && (
                        <span className="text-[10px] text-muted-foreground/60">({section.items.length})</span>
                      )}
                      <div className="flex-1" />
                      {isCollapsed ? (
                        <ChevronDown className="h-3 w-3 text-muted-foreground/40 shrink-0" />
                      ) : (
                        <ChevronUp className="h-3 w-3 text-muted-foreground/40 shrink-0" />
                      )}
                    </button>

                    {!isCollapsed && section.items.length > 0 && (
                      <div className="px-3 pb-3 space-y-1.5">
                        {section.items.map((item, j) => (
                          <div
                            key={j}
                            className={`text-sm leading-relaxed pl-3 border-l-2 ${style.borderColor}`}
                          >
                            <ReactMarkdown
                              remarkPlugins={[remarkGfm]}
                              components={{
                                p: ({ children }) => <span className="text-foreground/80">{children}</span>,
                                strong: ({ children }) => <strong className="font-semibold text-foreground">{children}</strong>,
                              }}
                            >
                              {item}
                            </ReactMarkdown>
                          </div>
                        ))}
                      </div>
                    )}

                    {!isCollapsed && section.items.length === 0 && (
                      <div className="px-3 pb-2">
                        <span className="text-xs text-muted-foreground/50 italic">暂无条目</span>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          ) : (
            content && (
              <div className="text-xs text-muted-foreground/60 text-center py-4">
                等待综合报告生成...
              </div>
            )
          )}

          {content && (
            <div className="pt-1">
              <button
                onClick={() => setShowRaw(!showRaw)}
                className="text-xs text-muted-foreground/50 hover:text-muted-foreground transition-colors flex items-center gap-1"
              >
                {showRaw ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
                查看原始综合报告
              </button>
              {showRaw && (
                <div className="mt-2 rounded-lg bg-muted/30 px-4 py-3 border border-border/50">
                  <div className="prose prose-slate prose-sm max-w-none
                    [&_h1]:text-base [&_h1]:font-bold [&_h1]:mt-4 [&_h1]:mb-2
                    [&_h2]:text-sm [&_h2]:font-semibold [&_h2]:mt-3 [&_h2]:mb-1.5
                    [&_h3]:text-sm [&_h3]:font-semibold [&_h3]:mt-3 [&_h3]:mb-1.5 [&_h3]:text-foreground/90
                    [&_p]:my-1.5 [&_p]:leading-relaxed [&_p]:text-sm
                    [&_ul]:my-1 [&_ol]:my-1
                    [&_li]:my-1 [&_li]:text-sm [&_li]:leading-relaxed
                    [&_blockquote]:my-1.5 [&_blockquote]:pl-3 [&_blockquote]:border-l-2 [&_blockquote]:border-primary/30 [&_blockquote]:text-muted-foreground
                    [&_strong]:font-semibold [&_strong]:text-foreground/90
                    [&_hr]:my-3 [&_hr]:border-border/50
                    [&_code]:bg-muted [&_code]:px-1 [&_code]:py-0.5 [&_code]:rounded [&_code]:text-xs
                    [&_pre]:my-2 [&_pre]:p-3 [&_pre]:bg-muted [&_pre]:rounded-lg [&_pre]:text-xs [&_pre]:overflow-x-auto
                    [&_a]:text-primary [&_a]:underline [&_a]:underline-offset-2
                  ">
                    <ReactMarkdown remarkPlugins={[remarkGfm]}>
                      {cleanedContent}
                    </ReactMarkdown>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
