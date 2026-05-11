"use client";

import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Brain, CheckCircle, ArrowUpRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { ArticleModal } from "./article-modal";

function cleanContent(raw: string): string {
  return raw.replace(/<[^>]+>/g, "").replace(/^\s+/, "").trim();
}

interface SoulResponseCardProps {
  name: string;
  content: string;
  ismismCode?: string;
  isStreaming?: boolean;
}

export function SoulResponseCard({ name, content, ismismCode = "", isStreaming = false }: SoulResponseCardProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const cleanedContent = cleanContent(content);
  const hasContent = cleanedContent.length > 0;
  const isArrived = hasContent || !isStreaming;

  return (
    <>
      <div
        className={cn(
          "flex flex-col rounded-lg border bg-background overflow-hidden h-40 transition-all duration-500",
          isArrived && hasContent && "border-primary/30 shadow-md shadow-primary/10 cursor-pointer hover:shadow-lg hover:-translate-y-0.5"
        )}
        onClick={() => hasContent && setIsModalOpen(true)}
      >
        <div className={cn(
          "px-4 py-2 border-b flex items-center justify-between transition-colors duration-500",
          isArrived && hasContent ? "bg-primary/5" : "bg-muted/30"
        )}>
          <div className="flex items-center gap-2">
            <span className="font-semibold text-sm">{name}</span>
            {ismismCode && (
              <span className="text-xs text-muted-foreground font-mono">{ismismCode}</span>
            )}
          </div>
          {isArrived && hasContent && (
            <ArrowUpRight className="h-3.5 w-3.5 text-muted-foreground/40 group-hover:text-muted-foreground transition-colors" />
          )}
        </div>
        <div className={cn(
          "flex items-center gap-2 px-4 py-2 border-b transition-colors duration-500",
          isArrived && hasContent ? "bg-emerald-50/50 dark:bg-emerald-950/10" : "bg-muted/10"
        )}>
          {isArrived && hasContent ? (
            <>
              <CheckCircle className="h-4 w-4 text-emerald-500" />
              <span className="text-xs text-emerald-600 dark:text-emerald-400 font-medium">已回应</span>
            </>
          ) : isStreaming ? (
            <>
              <div className="w-2 h-2 rounded-full bg-primary animate-pulse" />
              <span className="text-xs text-primary font-medium">回应中…</span>
              <div className="flex-1" />
              <div className="h-1 w-12 bg-muted rounded-full overflow-hidden">
                <div className="h-full bg-primary/20 animate-pulse rounded-full" style={{ width: "60%" }} />
              </div>
            </>
          ) : (
            <>
              <Brain className="h-4 w-4 text-primary animate-pulse" />
              <span className="text-xs text-muted-foreground">等待回应…</span>
              <div className="flex-1" />
              <div className="h-1 w-12 bg-muted rounded-full overflow-hidden">
                <div className="h-full bg-primary/20 animate-pulse rounded-full" style={{ width: "40%" }} />
              </div>
            </>
          )}
        </div>
        <div className="flex-1 flex items-center px-4 overflow-hidden">
          {hasContent ? (
            <p className="text-xs text-muted-foreground line-clamp-2 leading-relaxed">
              {cleanedContent.slice(0, 200)}
              {cleanedContent.length > 200 && "…"}
              {isStreaming && (
                <span className="inline-block w-1.5 h-3.5 bg-primary animate-pulse ml-0.5 align-middle rounded-full" />
              )}
            </p>
          ) : isStreaming ? (
            <p className="text-xs text-muted-foreground/60 italic">正在生成回应…</p>
          ) : (
            <p className="text-xs text-muted-foreground/60 italic">暂无内容</p>
          )}
        </div>
        {hasContent && !isStreaming && (
          <div className="px-4 py-1.5 border-t border-border/20 text-center">
            <span className="text-xs text-primary/60 group-hover:text-primary/80 transition-colors">
              点击阅读全文 →
            </span>
          </div>
        )}
      </div>

      <ArticleModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={name}
        ismismCode={ismismCode}
        content={cleanedContent}
      />
    </>
  );
}
