"use client";

import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { ArrowUpRight } from "lucide-react";
import { ArticleModal } from "./article-modal";

function cleanContent(raw: string): string {
  return raw.replace(/<[^>]+>/g, "").trim();
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

  return (
    <>
      <div
        className="group flex flex-col rounded-lg border bg-background overflow-hidden cursor-pointer transition-all duration-500 border-primary/30 shadow-md shadow-primary/10 hover:shadow-lg hover:-translate-y-0.5"
        onClick={() => setIsModalOpen(true)}
      >
        <div className="px-4 py-2 border-b flex items-center justify-between bg-primary/5">
          <div className="flex items-center gap-2">
            <div>
              <span className="font-semibold text-sm">辩证综合</span>
              <span className="ml-2 text-xs text-muted-foreground">
                {new Date(msg.created_at).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}
              </span>
            </div>
          </div>
          <ArrowUpRight className="h-3.5 w-3.5 text-muted-foreground/40 group-hover:text-muted-foreground transition-colors" />
        </div>

        <div className="flex-1 px-4 py-3 overflow-hidden">
          {cleanedContent ? (
            <div className="text-xs text-muted-foreground leading-relaxed line-clamp-5 prose prose-xs max-w-none dark:prose-invert [&_h3]:text-sm [&_h3]:font-semibold [&_h3]:text-foreground/90 [&_h3]:mt-0 [&_h3]:mb-1 [&_ul]:my-1 [&_li]:my-0 [&_p]:my-1">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {cleanedContent.slice(0, 800)}
              </ReactMarkdown>
            </div>
          ) : (
            <p className="text-xs text-muted-foreground/60 italic">暂无内容</p>
          )}
        </div>

        <div className="px-4 py-1.5 border-t border-border/20 text-center">
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
