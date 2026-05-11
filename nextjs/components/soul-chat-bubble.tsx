"use client";

import { memo, useEffect, useRef } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn } from "@/lib/utils";
import type { SoulMessage } from "@/hooks/use-websocket";
import { useCleanContent } from "@/hooks/use-clean-content";

interface SoulChatBubbleProps {
  message: SoulMessage;
  autoScroll?: boolean;
}

export const SoulChatBubble = memo(function SoulChatBubble({
  message,
  autoScroll,
}: SoulChatBubbleProps) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const lastScrollRef = useRef(0);

  useEffect(() => {
    if (!autoScroll || !message.isStreaming || !bottomRef.current) return;
    const now = Date.now();
    // Throttle scroll to ~200ms to avoid layout thrashing on high-frequency token updates
    if (now - lastScrollRef.current < 200) return;
    lastScrollRef.current = now;
    bottomRef.current.scrollIntoView({ behavior: "smooth" });
  }, [message.content, autoScroll, message.isStreaming]);

  const cleanedContent = useCleanContent(message.content);
  const initial = message.soulName.charAt(0).toUpperCase();

  return (
    <div
      data-testid={`soul-bubble-${message.soulName}`}
      className={cn(
        "flex gap-3 mb-4",
        message.error && "opacity-80"
      )}
    >
      <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-bold">
        {initial}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-sm font-semibold">{message.soulName}</span>
          {message.isStreaming && (
            <span className="inline-block w-2 h-2 rounded-full bg-green-500 animate-pulse" aria-hidden="true" />
          )}
          {message.error && (
            <span className="text-xs text-red-500">错误</span>
          )}
        </div>
        <div
          className={cn(
            "rounded-lg px-4 py-3 text-sm leading-relaxed",
            message.error
              ? "border border-red-200 bg-red-50 dark:bg-red-950"
              : "bg-muted"
          )}
        >
          <div className="prose prose-slate prose-sm max-w-none [&_p]:my-1 [&_p]:leading-relaxed [&_ul]:my-1 [&_ol]:my-1 [&_li]:my-0.5 [&_blockquote]:my-1.5 [&_blockquote]:pl-3 [&_blockquote]:border-l-2 [&_blockquote]:border-primary/30 [&_strong]:font-semibold [&_h3]:text-sm [&_h3]:font-semibold [&_h3]:mt-2 [&_h3]:mb-1">
            {message.content ? (
              <ReactMarkdown remarkPlugins={[remarkGfm]}>{cleanedContent}</ReactMarkdown>
            ) : null}
          </div>
          {message.isStreaming && (
            <span className="inline-block w-1.5 h-4 bg-foreground animate-pulse ml-0.5 align-text-bottom" aria-hidden="true" />
          )}
        </div>
        <div ref={bottomRef} />
      </div>
    </div>
  );
}, (prev, next) => {
  // Only re-render if content, streaming state, or error actually changed
  return (
    prev.message.content === next.message.content &&
    prev.message.isStreaming === next.message.isStreaming &&
    prev.message.error === next.message.error &&
    prev.autoScroll === next.autoScroll
  );
});
