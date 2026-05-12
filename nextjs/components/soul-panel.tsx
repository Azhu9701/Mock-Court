"use client";

import { useEffect, useRef, useState, useMemo } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Brain, AlertCircle, CheckCircle2, Zap, ChevronRight, ChevronDown } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { useCleanContent } from "@/hooks/use-clean-content";

interface SoulPanelProps {
  name: string;
  content: string;
  isStreaming: boolean;
  error?: string | null;
  hasCollision?: boolean;
  ismismCode?: string;
  isExpanded?: boolean;
  onToggleExpand?: () => void;
}

export function SoulPanel({
  name,
  content,
  isStreaming,
  error,
  hasCollision = false,
  ismismCode = "",
  isExpanded = false,
  onToggleExpand,
}: SoulPanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [showTooltip, setShowTooltip] = useState(false);
  const cleanedContent = useCleanContent(content);

  useEffect(() => {
    if (scrollRef.current && isStreaming && isExpanded) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [content, isStreaming, isExpanded]);

  const status = error ? "error" : isStreaming ? "streaming" : content ? "done" : "pending";

  const statusIcon = {
    pending: <Brain className="h-4 w-4 text-muted-foreground animate-pulse" />,
    streaming: <Brain className="h-4 w-4 text-primary animate-pulse" />,
    done: <CheckCircle2 className="h-4 w-4 text-emerald-500" />,
    error: <AlertCircle className="h-4 w-4 text-destructive" />,
  }[status];

  const progress = content.length > 0 ? Math.min(100, (content.length / 3000) * 100) : 0;
  const { firstParagraph, hasMore } = useMemo(() => {
    const paragraphs = content.split("\n\n");
    return {
      firstParagraph: paragraphs[0]?.substring(0, 100) || "",
      hasMore: content.length > 100 || paragraphs.length > 1,
    };
  }, [content]);

  return (
    <div
      className={`flex flex-col rounded-lg border bg-background overflow-hidden transition-all duration-300 cursor-pointer hover:shadow-md ${
        isExpanded ? "flex-1 min-h-[300px]" : "h-40"
      }`}
      onClick={onToggleExpand}
    >
      {/* 头部 */}
      <div className="px-4 py-2 border-b bg-muted/30 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-sm">{name}</span>
          {ismismCode && (
            <div
              className="relative"
              onMouseEnter={() => setShowTooltip(true)}
              onMouseLeave={() => setShowTooltip(false)}
            >
              <span className="text-xs text-muted-foreground font-mono flex items-center gap-1">
                {ismismCode}
                <ChevronRight className="h-3 w-3" />
              </span>
              {showTooltip && (
                <div className="absolute top-full left-0 mt-1 z-10 bg-background border rounded-lg shadow-lg p-3 text-xs whitespace-nowrap">
                  <p className="font-medium text-muted-foreground">主义主义坐标</p>
                  <p className="text-muted-foreground mt-1">点击查看魂详情</p>
                </div>
              )}
            </div>
          )}
        </div>
        
        {/* 操作按钮 */}
        <div className="flex items-center gap-2">
          {hasCollision && (
            <Badge variant="destructive" className="animate-pulse">
              <Zap className="h-3 w-3 mr-1" />
              碰撞
            </Badge>
          )}
          {content && hasMore && (
            <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform duration-300 ${isExpanded ? "rotate-180" : ""}`} />
          )}
        </div>
      </div>

      {/* 状态和进度条 */}
      <div className="flex items-center gap-2 px-4 py-2 border-b bg-muted/10">
        {statusIcon}
        <span className="text-xs text-muted-foreground">
          {status === "pending" && "等待召唤..."}
          {status === "streaming" && "正在回应..."}
          {status === "done" && "回应完成"}
          {status === "error" && "发生错误"}
        </span>
        
        {/* 进度条 */}
        <div className="flex-1 h-1 bg-muted rounded-full overflow-hidden ml-auto">
          <div
            className="h-full bg-primary transition-all duration-300"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>

      {/* 内容区域 */}
      <div
        ref={scrollRef}
        className={`overflow-y-auto p-4 text-sm leading-relaxed flex-1 ${!isExpanded ? "h-16" : ""}`}
      >
        {status === "pending" && !content && (
          <p className="text-muted-foreground italic text-center py-4">等待召唤...</p>
        )}
        {status === "error" && error && (
          <p className="text-destructive">{error}</p>
        )}
        {content && (
          <div className="prose prose-sm max-w-none">
            {!isExpanded && !isStreaming ? (
              <p className="line-clamp-2 text-muted-foreground">{firstParagraph}...</p>
            ) : (
              <>
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{cleanedContent}</ReactMarkdown>
                {isStreaming && <span className="inline-block w-2 h-4 bg-primary animate-pulse ml-0.5 align-middle" />}
              </>
            )}
          </div>
        )}
      </div>

      {/* 提示点击展开 */}
      {!isExpanded && content && hasMore && !isStreaming && (
        <div className="px-4 py-2 border-t bg-muted/10 text-center">
          <span className="text-xs text-muted-foreground">点击展开查看完整内容</span>
        </div>
      )}
    </div>
  );
}
