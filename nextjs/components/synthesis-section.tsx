"use client";

import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Sparkles, ArrowUpRight } from "lucide-react";
import { ArticleModal } from "./article-modal";

function cleanContent(raw: string): string {
  return raw.replace(/<[^>]+>/g, "").trim();
}

function extractSections(raw: string): { title: string; items: string[] }[] {
  const cleaned = cleanContent(raw);
  const sections: { title: string; items: string[] }[] = [];
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

interface SynthMsg {
  id: string;
  content: string;
  created_at: string;
}

interface SynthesisSectionProps {
  messages: SynthMsg[];
}

export function SynthesisSection({ messages }: SynthesisSectionProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);

  const msg = messages[0];
  const cleanedContent = cleanContent(msg.content);
  const sections = extractSections(msg.content);

  return (
    <>
      <div
        className="group rounded-xl border bg-background overflow-hidden cursor-pointer transition-all duration-300 hover:shadow-lg hover:shadow-purple-500/5 hover:-translate-y-0.5 hover:border-purple-300"
        onClick={() => setIsModalOpen(true)}
      >
        <div className="h-1 bg-gradient-to-r from-purple-500 via-pink-500 to-purple-500" />

        <div className="px-5 py-3 flex items-center justify-between border-b border-border/30">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-purple-50 text-purple-600 flex items-center justify-center">
              <Sparkles className="h-4 w-4" />
            </div>
            <div>
              <span className="font-semibold text-sm">辩证综合</span>
              <span className="ml-2 text-xs text-muted-foreground">
                {new Date(msg.created_at).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}
              </span>
            </div>
          </div>
          <ArrowUpRight className="h-4 w-4 text-muted-foreground/0 group-hover:text-muted-foreground transition-colors" />
        </div>

        <div className="p-5 space-y-3">
          {sections.length > 0 ? (
            sections.slice(0, 5).map((sec, i) => (
              <div key={i}>
                <h4 className="text-sm font-semibold text-foreground/90 mb-1">{sec.title}</h4>
                {sec.items.length > 0 && (
                  <ul className="space-y-0.5">
                    {sec.items.slice(0, 3).map((item, j) => (
                      <li key={j} className="text-xs text-muted-foreground leading-relaxed flex gap-1.5">
                        <span className="text-muted-foreground/40 shrink-0">•</span>
                        <span className="line-clamp-2">{item.length > 80 ? item.substring(0, 80) + "..." : item}</span>
                      </li>
                    ))}
                    {sec.items.length > 3 && (
                      <li className="text-xs text-primary/60">...还有 {sec.items.length - 3} 条</li>
                    )}
                  </ul>
                )}
              </div>
            ))
          ) : (
            <p className="text-sm text-muted-foreground leading-relaxed line-clamp-4">
              {cleanedContent.substring(0, 200)}...
            </p>
          )}
        </div>

        <div className="px-5 py-2 border-t border-border/20 text-center">
          <span className="text-xs text-primary/60 group-hover:text-primary/80 transition-colors">
            点击阅读完整综合报告 →
          </span>
        </div>
      </div>

      <ArticleModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title="辩证综合"
        content={cleanedContent}
      />
    </>
  );
}
