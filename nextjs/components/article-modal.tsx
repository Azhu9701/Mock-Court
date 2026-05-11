"use client";

import { useEffect, useMemo, useCallback } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { X, Bot, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ArticleModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  ismismCode?: string;
  content: string;
}

function cleanContent(content: string): string {
  // 移除角括号格式的文本，如 <未明子从一片哲学硝烟中抬起头，目光如刀>
  return content.replace(/<[^>]+>/g, "").trim();
}

export function ArticleModal({ isOpen, onClose, title, ismismCode, content }: ArticleModalProps) {
  const cleanedContent = useMemo(() => cleanContent(content), [content]);

  const handleEscape = useCallback((e: KeyboardEvent) => {
    if (e.key === "Escape") onClose();
  }, [onClose]);

  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = "hidden";
      document.addEventListener("keydown", handleEscape);
    }
    
    return () => {
      document.body.style.overflow = "";
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, handleEscape]);

  const markdownElement = useMemo(() => (
    <ReactMarkdown remarkPlugins={[remarkGfm]}>{cleanedContent}</ReactMarkdown>
  ), [cleanedContent]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 lg:p-8">
      {/* 背景遮罩 */}
      <div 
        className="absolute inset-0 bg-black/60 backdrop-blur-md"
        onClick={onClose}
      />
      
      {/* 弹窗内容 */}
      <div className="relative bg-background rounded-2xl shadow-2xl max-w-3xl w-full max-h-[90vh] overflow-hidden animate-in fade-in zoom-in-95 duration-200">
        {/* 头部 */}
        <div className="sticky top-0 z-10 px-6 py-5 border-b bg-background/95 backdrop-blur-sm flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center">
              <Bot className="h-5 w-5 text-primary" />
            </div>
            <div>
              <h2 className="font-semibold text-lg leading-tight">{title}</h2>
              {ismismCode && (
                <div className="flex items-center gap-2 mt-0.5">
                  <span className="text-xs font-mono text-muted-foreground bg-muted/50 px-2 py-0.5 rounded">{ismismCode}</span>
                  <span className="text-xs text-muted-foreground">主义主义坐标</span>
                </div>
              )}
            </div>
          </div>
          <Button variant="ghost" size="icon" onClick={onClose} className="h-9 w-9 rounded-full hover:bg-muted/50 transition-colors">
            <X className="h-4 w-4" />
          </Button>
        </div>

        {/* 内容区域 */}
        <div className="p-6 sm:p-8 overflow-y-auto max-h-[calc(90vh-100px)]">
          <article className="prose prose-slate prose-lg max-w-none leading-7">
            {/* 自定义样式覆盖 */}
            <style>{`
              .prose h1 {
                font-size: 1.75rem;
                font-weight: 700;
                color: var(--color-foreground);
                margin-top: 2rem;
                margin-bottom: 1rem;
                line-height: 1.25;
                letter-spacing: -0.025em;
              }
              .prose h2 {
                font-size: 1.375rem;
                font-weight: 600;
                color: var(--color-foreground);
                margin-top: 1.75rem;
                margin-bottom: 0.875rem;
                line-height: 1.375;
                letter-spacing: -0.01em;
                padding-bottom: 0.375rem;
                border-bottom: 1px solid var(--color-border);
              }
              .prose h3 {
                font-size: 1.125rem;
                font-weight: 600;
                color: var(--color-foreground);
                margin-top: 1.5rem;
                margin-bottom: 0.75rem;
                line-height: 1.4;
              }
              .prose h4 {
                font-size: 1rem;
                font-weight: 600;
                color: var(--color-foreground);
                margin-top: 1.25rem;
                margin-bottom: 0.5rem;
              }
              .prose p {
                margin-top: 1rem;
                margin-bottom: 1rem;
                line-height: 1.75;
                color: var(--color-foreground);
              }
              .prose ul, .prose ol {
                margin-top: 1rem;
                margin-bottom: 1rem;
                padding-left: 1.5rem;
              }
              .prose li {
                margin-top: 0.5rem;
                line-height: 1.75;
              }
              .prose li > p {
                margin-top: 0.25rem;
                margin-bottom: 0.25rem;
              }
              .prose blockquote {
                margin-top: 1.25rem;
                margin-bottom: 1.25rem;
                padding: 1rem 1.25rem;
                background-color: var(--color-muted);
                border-left: 3px solid var(--color-primary);
                border-radius: 0 0.375rem 0.375rem 0;
                font-style: italic;
                color: var(--color-muted-foreground);
              }
              .prose code {
                padding: 0.125rem 0.375rem;
                background-color: var(--color-muted);
                border-radius: 0.25rem;
                font-size: 0.875em;
                font-weight: 500;
              }
              .prose pre {
                margin-top: 1.25rem;
                margin-bottom: 1.25rem;
                padding: 1rem;
                background-color: var(--color-muted);
                border-radius: 0.5rem;
                overflow-x: auto;
              }
              .prose pre code {
                padding: 0;
                background-color: transparent;
                border-radius: 0;
              }
              .prose strong {
                font-weight: 600;
                color: var(--color-foreground);
              }
              .prose hr {
                margin-top: 2rem;
                margin-bottom: 2rem;
                border: none;
                height: 1px;
                background-color: var(--color-border);
              }
              .prose a {
                color: var(--color-primary);
                text-decoration: underline;
                text-underline-offset: 0.125em;
                font-weight: 500;
              }
              .prose a:hover {
                color: var(--color-primary);
                text-decoration-thickness: 2px;
              }
              .prose table {
                width: 100%;
                margin-top: 1.25rem;
                margin-bottom: 1.25rem;
                border-collapse: collapse;
                border-spacing: 0;
              }
              .prose th {
                padding: 0.75rem 1rem;
                text-align: left;
                font-weight: 600;
                background-color: var(--color-muted);
                border-bottom: 2px solid var(--color-border);
              }
              .prose td {
                padding: 0.75rem 1rem;
                border-bottom: 1px solid var(--color-border);
              }
              .prose tr:hover td {
                background-color: var(--color-muted);
              }
            `}</style>
            
            {markdownElement}
          </article>
        </div>

        {/* 底部 */}
        <div className="sticky bottom-0 px-6 py-4 border-t bg-background/95 backdrop-blur-sm flex items-center justify-between">
          <div className="text-xs text-muted-foreground flex items-center gap-1">
            <span>按 Esc 关闭</span>
            <span className="text-border">|</span>
            <span>点击外部关闭</span>
          </div>
          <Button onClick={onClose} className="gap-2">
            <X className="h-4 w-4" />
            关闭
          </Button>
        </div>
      </div>
    </div>
  );
}
