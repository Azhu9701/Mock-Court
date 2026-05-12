"use client";

import { useState, useRef, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Send, Loader2, AlertCircle, CheckCircle2, StopCircle, ChevronDown, ChevronUp } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { API_BASE } from "@/lib/api";
import { cn } from "@/lib/utils";

const WS_HOST = API_BASE.replace("http://", "ws://").replace("/api/v1", "");
const FLUSH_INTERVAL_MS = 50;

interface LocalMsg {
  role: "user" | "assistant";
  content: string;
  reasoningContent: string;
  id: string;
  streaming?: boolean;
  error?: string;
}

export default function FollowUpInput({ sessionId }: { sessionId: string }) {
  const router = useRouter();
  const [followUp, setFollowUp] = useState(() => {
    if (typeof window === "undefined") return "";
    return localStorage.getItem(`followup-draft-${sessionId}`) || "";
  });
  const [sending, setSending] = useState(false);
  const [localMsgs, setLocalMsgs] = useState<LocalMsg[]>([]);
  const [error, setError] = useState("");
  const [expandedMsgId, setExpandedMsgId] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const contentRef = useRef("");
  const reasoningContentRef = useRef("");
  const currentMsgIdRef = useRef<string | null>(null);
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingFlushRef = useRef(false);
  const bottomRef = useRef<HTMLDivElement>(null);
  const lastScrollRef = useRef(0);
  const mountedRef = useRef(true);
  const sendingRef = useRef(false);
  const abortControllerRef = useRef<AbortController | null>(null);

  const log = (msg: string, ...args: any[]) => {
    console.log(`[FollowUp] ${msg}`, ...args);
  };

  const scheduleFlush = useCallback(() => {
    if (pendingFlushRef.current || !mountedRef.current) return;
    pendingFlushRef.current = true;
    flushTimerRef.current = setTimeout(() => {
      pendingFlushRef.current = false;
      if (!mountedRef.current || !currentMsgIdRef.current) return;
      setLocalMsgs((prev) =>
        prev.map((msg) =>
          msg.id === currentMsgIdRef.current
            ? {
                ...msg,
                content: contentRef.current,
                reasoningContent: reasoningContentRef.current,
                streaming: true,
              }
            : msg
        )
      );
      const now = Date.now();
      if (now - lastScrollRef.current > 200) {
        lastScrollRef.current = now;
        bottomRef.current?.scrollIntoView({ behavior: "smooth", block: "nearest" });
      }
    }, FLUSH_INTERVAL_MS);
  }, []);

  const flushImmediate = useCallback(() => {
    if (flushTimerRef.current) {
      clearTimeout(flushTimerRef.current);
      pendingFlushRef.current = false;
    }
    if (!mountedRef.current || !currentMsgIdRef.current) return;
    setLocalMsgs((prev) =>
      prev.map((msg) =>
        msg.id === currentMsgIdRef.current
          ? {
              ...msg,
              content: contentRef.current,
              reasoningContent: reasoningContentRef.current,
              streaming: false,
            }
          : msg
      )
    );
    bottomRef.current?.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }, []);

  const cleanup = useCallback(() => {
    log("Cleaning up...");
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    if (flushTimerRef.current) {
      clearTimeout(flushTimerRef.current);
    }
  }, []);

  const stopGeneration = useCallback(() => {
    log("Stopping generation...");
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    sendingRef.current = false;
    setSending(false);
    flushImmediate();
    cleanup();
  }, [cleanup, flushImmediate]);

  const onFollowUp = useCallback(async () => {
    if (!followUp.trim() || sending) return;

    const question = followUp.trim();
    log("Starting follow-up with question:", question);

    setMsgHistory((prev) => {
      const next = [question, ...prev.filter((m) => m !== question)].slice(0, 50);
      return next;
    });
    setMsgHistoryIdx(-1);
    setFollowUp("");
    setSending(true);
    sendingRef.current = true;
    setError("");
    contentRef.current = "";
    reasoningContentRef.current = "";

    const qId = `q-${Date.now()}`;
    const aId = `a-${Date.now()}`;
    currentMsgIdRef.current = aId;

    setLocalMsgs((prev) => [
      ...prev,
      { role: "user", content: question, reasoningContent: "", id: qId },
      {
        role: "assistant",
        content: "",
        reasoningContent: "",
        id: aId,
        streaming: true,
      },
    ]);

    const wsUrl = `${WS_HOST}/ws/possess/${sessionId}/main`;
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      log("WebSocket connected! Sending follow-up request...");
      fetch(`${API_BASE}/possess/${sessionId}/follow-up`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ question }),
      }).catch((err) => {
        log("Error sending follow-up request:", err);
        setError(`发送失败: ${err.message}`);
        sendingRef.current = false;
        setSending(false);
        cleanup();
      });
    };

    ws.onmessage = (event) => {
      log("Received WebSocket message:", event.data);
      try {
        const msg = JSON.parse(event.data);
        if (msg.event_type === "synthesis_chunk") {
          if (msg.reasoning_content) {
            reasoningContentRef.current += msg.reasoning_content;
          }
          if (msg.payload) {
            contentRef.current += msg.payload;
          }
          scheduleFlush();
        } else if (msg.event_type === "synthesis_done") {
          flushImmediate();
          setSending(false);
          cleanup();
          router.refresh();
        } else if (msg.event_type === "error") {
          setError(msg.payload);
          flushImmediate();
          setSending(false);
          cleanup();
        }
      } catch (err) {
        log("Error parsing message:", err);
      }
    };

    ws.onerror = () => {
      log("WebSocket error");
      setError("WebSocket 连接失败");
      sendingRef.current = false;
      flushImmediate();
      setSending(false);
    };

    ws.onclose = () => {
      log("WebSocket closed");
      if (sendingRef.current) {
        sendingRef.current = false;
        flushImmediate();
        setSending(false);
      }
    };
  }, [followUp, sending, sessionId, scheduleFlush, flushImmediate, cleanup, router]);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      cleanup();
    };
  }, [cleanup]);

  useEffect(() => {
    if (followUp) {
      localStorage.setItem(`followup-draft-${sessionId}`, followUp);
    } else {
      localStorage.removeItem(`followup-draft-${sessionId}`);
    }
  }, [followUp, sessionId]);

  const [msgHistory, setMsgHistory] = useState<string[]>([]);
  const [msgHistoryIdx, setMsgHistoryIdx] = useState(-1);

  const dismissError = () => setError("");

  return (
    <div className="space-y-4">
      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 dark:bg-red-950/30 p-4 flex items-start gap-3">
          <AlertCircle className="h-5 w-5 text-red-500 shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="text-red-600 dark:text-red-400 font-medium">发生错误</p>
            <p className="text-red-500 dark:text-red-300 text-sm mt-1">{error}</p>
          </div>
          <Button variant="ghost" size="sm" onClick={dismissError} className="text-red-600 hover:text-red-700 hover:bg-red-100 dark:hover:bg-red-900/30">
            关闭
          </Button>
        </div>
      )}

      {localMsgs.length > 0 && (
        <div className="space-y-4">
          {localMsgs.map((msg) => (
            <div key={msg.id} className="space-y-2">
              {msg.role === "assistant" && msg.reasoningContent && (
                <div className="mr-8">
                  <div
                    className={cn(
                      "rounded-t-xl border border-purple-100 dark:border-purple-800/50 overflow-hidden transition-all duration-300",
                      expandedMsgId === msg.id ? "bg-purple-50/50 dark:bg-purple-950/20" : "bg-transparent"
                    )}
                  >
                    <div className="flex items-center justify-between px-4 py-2">
                      <span className="text-xs font-semibold text-purple-500 dark:text-purple-400 flex items-center gap-1.5">
                        <Loader2 className="h-3 w-3 text-purple-400" />
                        思考过程
                      </span>
                      <button
                        onClick={() => setExpandedMsgId(expandedMsgId === msg.id ? null : msg.id)}
                        className="p-1 rounded hover:bg-purple-100 dark:hover:bg-purple-800/50 transition-colors"
                      >
                        {expandedMsgId === msg.id ? (
                          <ChevronUp className="h-4 w-4 text-purple-400" />
                        ) : (
                          <ChevronDown className="h-4 w-4 text-purple-400" />
                        )}
                      </button>
                    </div>
                    <div
                      className={cn(
                        "px-4 transition-all duration-300 overflow-hidden",
                        expandedMsgId === msg.id ? "max-h-40 py-2" : "max-h-6 py-1"
                      )}
                    >
                      <p
                        className={cn(
                          "text-sm text-muted-foreground/60 dark:text-muted-foreground/50 leading-relaxed",
                          expandedMsgId === msg.id ? "whitespace-pre-wrap" : "line-clamp-1"
                        )}
                      >
                        {msg.reasoningContent}
                        {msg.streaming && (
                          <span className="inline-block w-1 h-3 bg-purple-400/50 animate-pulse ml-0.5 align-text-bottom rounded-full" />
                        )}
                      </p>
                    </div>
                  </div>
                </div>
              )}

              <div
                className={cn(
                  "rounded-xl p-4 text-sm transition-all duration-200",
                  msg.role === "user"
                    ? "bg-primary/5 ml-8 border border-primary/10"
                    : "bg-purple-50 dark:bg-purple-950/30 border border-purple-200 dark:border-purple-800 mr-8"
                )}
              >
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-xs font-semibold text-muted-foreground flex items-center gap-1.5">
                    {msg.role === "user" ? (
                      <>
                        <CheckCircle2 className="h-3 w-3 text-green-500" />
                        用户
                      </>
                    ) : (
                      <>
                        {msg.streaming ? (
                          <Loader2 className="h-3 w-3 animate-spin text-purple-500" />
                        ) : (
                          <CheckCircle2 className="h-3 w-3 text-green-500" />
                        )}
                        追问回应
                      </>
                    )}
                  </span>
                </div>

                <div className="prose prose-slate prose-sm max-w-none [&_h1]:text-base [&_h1]:font-bold [&_h1]:mt-4 [&_h1]:mb-2 [&_h2]:text-sm [&_h2]:font-semibold [&_h2]:mt-3 [&_h2]:mb-1.5 [&_h3]:text-sm [&_h3]:font-semibold [&_h3]:mt-2.5 [&_h3]:mb-1 [&_p]:my-1.5 [&_p]:leading-relaxed [&_ul]:my-1.5 [&_ol]:my-1.5 [&_li]:my-0.5 [&_li]:leading-relaxed [&_blockquote]:my-2 [&_blockquote]:pl-3 [&_blockquote]:border-l-2 [&_blockquote]:border-purple-400 [&_blockquote]:text-muted-foreground [&_blockquote]:italic [&_strong]:font-semibold [&_code]:text-xs [&_code]:px-1 [&_code]:py-0.5 [&_code]:bg-muted [&_code]:rounded [&_pre]:my-2 [&_pre]:p-3 [&_pre]:bg-muted [&_pre]:rounded-lg [&_hr]:my-3 [&_hr]:border-border">
                  {msg.content ? (
                    <ReactMarkdown remarkPlugins={[remarkGfm]}>{msg.content.replace(/<[^>]+>/g, "")}</ReactMarkdown>
                  ) : msg.streaming ? (
                    <span className="inline-block text-muted-foreground">
                      思考中<span className="animate-pulse">...</span>
                    </span>
                  ) : null}
                  {msg.streaming && msg.content !== "" && (
                    <span className="inline-block w-1.5 h-4 bg-purple-500 animate-pulse ml-0.5 align-text-bottom rounded-full" />
                  )}
                </div>
              </div>
            </div>
          ))}
          <div ref={bottomRef} />
        </div>
      )}

      <div className="sticky bottom-0 bg-background/95 backdrop-blur-sm border-t pt-4 pb-2 mt-4">
        <div className="flex gap-2 items-end">
          <Textarea
            placeholder="输入您的追问... (按 Enter 发送，Shift+Enter 换行，↑ 历史)"
            value={followUp}
            onChange={(e) => {
              setFollowUp(e.target.value);
              setMsgHistoryIdx(-1);
            }}
            rows={2}
            className="flex-1 resize-none transition-all focus:ring-2 focus:ring-primary/20"
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                onFollowUp();
                return;
              }
              if (e.key === "ArrowUp" && msgHistory.length > 0) {
                const textarea = e.currentTarget as HTMLTextAreaElement;
                if (textarea.selectionStart !== textarea.selectionEnd) return;
                if (msgHistoryIdx === -1) {
                  const nextIdx = 0;
                  setMsgHistoryIdx(nextIdx);
                  setFollowUp(msgHistory[nextIdx]);
                } else if (msgHistoryIdx < msgHistory.length - 1) {
                  const nextIdx = msgHistoryIdx + 1;
                  setMsgHistoryIdx(nextIdx);
                  setFollowUp(msgHistory[nextIdx]);
                }
                e.preventDefault();
                return;
              }
              if (e.key === "ArrowDown" && msgHistoryIdx >= 0) {
                if (msgHistoryIdx === 0) {
                  setMsgHistoryIdx(-1);
                  setFollowUp("");
                } else {
                  const nextIdx = msgHistoryIdx - 1;
                  setMsgHistoryIdx(nextIdx);
                  setFollowUp(msgHistory[nextIdx]);
                }
                e.preventDefault();
              }
            }}
            disabled={sending}
            data-testid="follow-up-input"
          />
          {sending ? (
            <Button
              onClick={stopGeneration}
              data-testid="stop-btn"
              className="h-[66px] px-4 bg-red-500 hover:bg-red-600"
            >
              <StopCircle className="h-5 w-5" />
            </Button>
          ) : (
            <Button
              onClick={onFollowUp}
              disabled={!followUp.trim()}
              data-testid="follow-up-btn"
              className="h-[66px] px-4"
            >
              <Send className="h-5 w-5" />
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
