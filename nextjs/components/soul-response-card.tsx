"use client";

import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Bot, ArrowUpRight } from "lucide-react";
import { ArticleModal } from "./article-modal";
import { getSoulAccent, getSoulAvatarBg } from "@/lib/soul-utils";

function cleanContent(raw: string): string {
  return raw.replace(/<[^>]+>/g, "").replace(/^\s+/, "").trim();
}

function extractViewpoint(raw: string): string {
  const cleaned = cleanContent(raw);
  const paragraphs = cleaned.split(/\n\n+/).filter(Boolean);

  const boldRe = /\*\*(.+?)\*\*/g;
  const boldPoints: string[] = [];
  let m: RegExpExecArray | null;
  while ((m = boldRe.exec(cleaned)) !== null) {
    const t = m[1].trim();
    if (t.length > 6 && t.length < 120 && !t.startsWith("第") && !t.startsWith("步")) {
      boldPoints.push(t);
    }
  }
  if (boldPoints.length >= 1) {
    return boldPoints.slice(0, 3).map(b => `• ${b}`).join("\n");
  }

  const h3Parts = cleaned.split(/^###\s+/m).slice(1);
  if (h3Parts.length > 0) {
    const titles = h3Parts.map(p => p.split("\n")[0].replace(/\*\*/g, "").trim()).filter(t => t.length > 3);
    if (titles.length > 0) return titles.slice(0, 3).map(t => `• ${t}`).join("\n");
  }

  for (const p of paragraphs) {
    const trimmed = p.trim();
    if (
      trimmed.length > 20 &&
      !trimmed.startsWith("好，") &&
      !trimmed.startsWith("嗯") &&
      !trimmed.startsWith("你的问题") &&
      !trimmed.startsWith("看来") &&
      !trimmed.startsWith("收到") &&
      !trimmed.startsWith("好的") &&
      !trimmed.startsWith("明白了") &&
      !trimmed.startsWith("（")
    ) {
      return trimmed.length > 150 ? trimmed.substring(0, 150) + "..." : trimmed;
    }
  }

  const fallback = paragraphs[0] || cleaned;
  return fallback.length > 150 ? fallback.substring(0, 150) + "..." : fallback;
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
  const viewpoint = extractViewpoint(content);
  const hasMore = cleanedContent.length > 150;
  const accent = getSoulAccent(ismismCode);
  const avatarBg = getSoulAvatarBg(ismismCode);

  return (
    <>
      <div
        className="group relative flex flex-col rounded-xl border bg-background overflow-hidden cursor-pointer transition-all duration-300 ease-out hover:shadow-lg hover:shadow-primary/5 hover:-translate-y-0.5 border-border/60 hover:border-primary/40 h-40"
        onClick={() => !isStreaming && hasMore && setIsModalOpen(true)}
      >
        <div className={`h-1 ${accent}`} />

        <div className="px-4 py-2.5 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className={`w-8 h-8 rounded-lg flex items-center justify-center ${avatarBg}`}>
              <Bot className="h-4 w-4" />
            </div>
            <span className="font-semibold text-sm">{name}</span>
          </div>
          {hasMore && !isStreaming && (
            <ArrowUpRight className="h-4 w-4 text-muted-foreground/0 group-hover:text-muted-foreground transition-colors" />
          )}
        </div>

        <div className="flex-1 px-4 pb-3 pt-1 overflow-hidden">
          {isStreaming ? (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <div className="w-2 h-2 rounded-full bg-primary animate-pulse" />
              <span>正在回应...</span>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground/80 leading-relaxed whitespace-pre-line line-clamp-4">
              {viewpoint}
            </p>
          )}
        </div>

        {hasMore && !isStreaming && (
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
