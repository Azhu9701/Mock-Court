"use client";

import { useWebSocket, type ProcessStep, type LogEntry, type SoulMessage } from "@/hooks/use-websocket";
import { SingleView } from "@/components/single-view";
import { ConferenceView } from "@/components/conference-view";
import { DebateView } from "@/components/debate-view";
import { RelayView } from "@/components/relay-view";
import { LearnView } from "@/components/learn-view";
import { PracticeOpeningView } from "@/components/practice-opening-view";
import { SessionStatusBar } from "@/components/session-status-bar";
import { Brain, Loader2, AlertTriangle, Key, CheckCircle, Sparkles, Wifi, Zap, MessageCircle, ChevronRight, Globe } from "lucide-react";
import { cn } from "@/lib/utils";
import { PostSessionReview } from "@/components/post-session-review";
import FollowUpInput from "@/components/follow-up-input";
import Link from "next/link";
import { useState } from "react";

interface MatchedSoulInfo {
  name: string;
  field: string;
  ismism_code: string;
}

interface SessionRunnerProps {
  sessionId: string;
  mode: string;
  matchedSouls?: MatchedSoulInfo[];
  onDone?: () => void;
  sessionDone?: boolean;
}

import { modeLabel } from "@/config/possession-modes";

const stepIcons: Record<string, React.ComponentType<{className?: string}>> = {
  Connected: Wifi,
  SessionStarted: Zap,
  EntryClassified: Brain,
  SoulStarted: Sparkles,
  SynthesisStarted: Brain,
  SearchComplete: Globe,
  SoulDone: CheckCircle,
  SynthesisDone: CheckCircle,
  SessionComplete: CheckCircle,
  SoulError: AlertTriangle,
};

const stepColors: Record<string, string> = {
  Connected: "text-blue-400",
  SessionStarted: "text-green-500",
  EntryClassified: "text-purple-500",
  SoulStarted: "text-amber-500",
  SynthesisStarted: "text-indigo-500",
  SearchComplete: "text-blue-500",
  SoulDone: "text-green-500",
  SynthesisDone: "text-indigo-500",
  SessionComplete: "text-green-600",
  SoulError: "text-red-500",
};

const nonVisual = new Set(["Connected", "SessionComplete", "SynthesisDone", "SoulDone"]);

export function ProcessTimeline({ steps }: { steps: ProcessStep[] }) {
  if (steps.length === 0) return null;
  const visual = steps.filter((s) => !nonVisual.has(s.event));
  if (visual.length === 0) return null;
  return (
    <div className="w-10 shrink-0 border-r flex flex-col items-center gap-1.5 pt-4 bg-muted/20" data-testid="process-timeline">
      {visual.map((step, i) => {
        const Icon = stepIcons[step.event];
        const color = stepColors[step.event] || "text-muted-foreground";
        return (
          <div key={i} className={cn(
            "flex items-center justify-center h-7 w-7 rounded-full bg-background border transition-all duration-500",
            color,
            step.event === "SoulStarted" && "animate-in zoom-in duration-300"
          )} title={step.soulName || step.message}>
            {Icon ? <Icon className="h-3.5 w-3.5" /> : <span className="text-[10px] font-bold">{step.soulName?.charAt(0)}</span>}
          </div>
        );
      })}
    </div>
  );
}

function ConnectingView() {
  return (
    <div className="flex flex-col items-center justify-center flex-1 gap-4 text-muted-foreground">
      <Loader2 className="h-8 w-8 animate-spin text-primary" />
      <div className="text-center space-y-1">
        <p className="text-lg font-medium">正在建立连接...</p>
        <p className="text-sm">连接到魂服务，准备召唤</p>
      </div>
    </div>
  );
}

const SOUL_QUOTES = [
  "正在召唤 {name} 之魂…",
  "{name} 正在检视你的问题…",
  "{name} 开始构建分析框架…",
  "等待 {name} 的回应…",
];

function WaitingSoulsView({ mode, matchedSouls, processSteps, messages }: { mode: string; matchedSouls?: MatchedSoulInfo[]; processSteps?: ProcessStep[]; messages: Record<string, SoulMessage> }) {
  const steps = processSteps || [];
  const arrivedSouls = steps
    .filter((s) => s.event === "SoulStarted" || s.event === "SoulDone")
    .map((s) => s.soulName || "");

  const classified = steps.find((s) => s.event === "EntryClassified");
  const parsedFromClassified: string[] = [];
  if (classified) {
    const match = classified.message.match(/匹配魂[：:]\s*(.+)/);
    if (match) {
      match[1].split(/[,，、]\s*/).forEach((n) => { if (n.trim()) parsedFromClassified.push(n.trim()); });
    }
  }

  const allSoulNames = Array.from(new Set([...arrivedSouls, ...parsedFromClassified]));
  const dynamicSouls: MatchedSoulInfo[] = allSoulNames.map((name) => ({
    name, field: arrivedSouls.includes(name) ? "已召唤" : "等待召唤…", ismism_code: "",
  }));
  const souls = matchedSouls && matchedSouls.length > 0 ? matchedSouls : (dynamicSouls.length > 0 ? dynamicSouls : null);
  const arrivedSet = new Set(arrivedSouls);

  return (
    <div className="flex flex-col flex-1 p-4 gap-4 overflow-y-auto">
      <div className="text-center py-2">
        <p className="text-sm text-muted-foreground">
          模式：{modeLabel(mode)} | 正在召唤 AI 生成回应…
        </p>
      </div>
      {souls ? (
        <div className="grid gap-3 grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {souls.map((soul) => {
            const hasArrived = arrivedSet.has(soul.name);
            const quote = SOUL_QUOTES[Math.floor(Math.random() * SOUL_QUOTES.length)].replace("{name}", soul.name);
            const soulMsg = messages[soul.name];
            const streamingContent = soulMsg?.content || "";
            return (
              <div
                key={soul.name}
                className={cn(
                  "flex flex-col rounded-lg border bg-background overflow-hidden h-40 transition-all duration-500",
                  hasArrived && "border-primary/30 shadow-md shadow-primary/10"
                )}
              >
                <div className={cn(
                  "px-4 py-2 border-b flex items-center justify-between transition-colors duration-500",
                  hasArrived ? "bg-primary/5" : "bg-muted/30"
                )}>
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-sm">{soul.name}</span>
                    {soul.ismism_code && (
                      <span className="text-xs text-muted-foreground font-mono">{soul.ismism_code}</span>
                    )}
                  </div>
                  {hasArrived && (
                    <CheckCircle className="h-3.5 w-3.5 text-emerald-500 animate-in zoom-in duration-300" />
                  )}
                </div>
                <div className={cn(
                  "flex items-center gap-2 px-4 py-2 border-b transition-colors duration-500",
                  hasArrived ? "bg-emerald-50/50 dark:bg-emerald-950/10" : "bg-muted/10"
                )}>
                  {hasArrived ? (
                    <>
                      <CheckCircle className="h-4 w-4 text-emerald-500" />
                      <span className="text-xs text-emerald-600 dark:text-emerald-400 font-medium">已到达</span>
                    </>
                  ) : (
                    <>
                      <Brain className="h-4 w-4 text-primary animate-pulse" />
                      <span className="text-xs text-muted-foreground">
                        {quote}
                      </span>
                    </>
                  )}
                  <div className="flex-1" />
                  {!hasArrived && (
                    <div className="h-1 w-12 bg-muted rounded-full overflow-hidden">
                      <div className="h-full bg-primary/20 animate-pulse rounded-full" style={{ width: "40%" }} />
                    </div>
                  )}
                </div>
                <div className="flex-1 flex items-center px-4 overflow-hidden">
                  {streamingContent ? (
                    <p className="text-xs text-muted-foreground line-clamp-2 leading-relaxed">
                      {streamingContent}
                      {soulMsg?.isStreaming && (
                        <span className="inline-block w-1.5 h-3.5 bg-primary animate-pulse ml-0.5 align-middle rounded-full" />
                      )}
                    </p>
                  ) : (
                    <p className="text-xs text-muted-foreground/60 italic truncate">
                      {soul.field || "正在加载思维框架…"}
                    </p>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center flex-1 gap-4 text-muted-foreground">
          <Brain className="h-8 w-8 text-primary animate-pulse" />
          <div className="text-center space-y-2">
            <p className="text-lg font-medium">魂正在思考...</p>
            <p className="text-sm">首次调用可能需要 5-10 秒，请耐心等待</p>
          </div>
        </div>
      )}
    </div>
  );
}

function RequireApiKeyView() {
  return (
    <div className="flex flex-col items-center justify-center flex-1 gap-4 p-8">
      <div className="max-w-md text-center space-y-4">
        <Key className="h-10 w-10 text-yellow-500 mx-auto" />
        <h3 className="text-lg font-semibold">魂无法回应</h3>
        <p className="text-sm text-muted-foreground">需要配置 LLM API Key 才能驱动魂</p>
        <div className="text-xs text-left space-y-1 bg-muted rounded-lg p-3 font-mono">
          <p>export ANTHROPIC_API_KEY=sk-ant-...</p>
          <p>export OPENAI_API_KEY=sk-...</p>
          <p>export DEEPSEEK_API_KEY=sk-...</p>
        </div>
        <p className="text-xs text-muted-foreground">设置后重启 API 服务即可</p>
      </div>
    </div>
  );
}

function ErrorView({ error, onReconnect }: { error: string; onReconnect: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center flex-1 gap-4 p-8">
      <AlertTriangle className="h-10 w-10 text-red-500" />
      <h3 className="text-lg font-semibold">连接错误</h3>
      <p className="text-sm text-muted-foreground text-center max-w-md">{error}</p>
      <button onClick={onReconnect} className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground" data-testid="reconnect-btn">
        重新连接
      </button>
    </div>
  );
}

export function SessionRunner({ sessionId, mode, matchedSouls, onDone, sessionDone }: SessionRunnerProps) {
  const { messages, synthesis, status, error, processSteps, cost, collisions, toolCalls, logs, reconnect } =
    useWebSocket(sessionId);

  const hasMessages = Object.keys(messages).length > 0;
  const [reviewDone, setReviewDone] = useState(false);

  const streamingSouls = Object.entries(messages)
    .filter(([, m]) => m.isStreaming)
    .map(([name]) => name);

  const lastStep = processSteps[processSteps.length - 1];
  const classifiedStep = processSteps.find((s) => s.event === "EntryClassified");
  let progressText = "";
  if (status === "streaming") {
    if (streamingSouls.length > 0) {
      progressText = `${streamingSouls.join("、")} 生成中…`;
    } else if (classifiedStep) {
      const soulsInMsg = classifiedStep.message.match(/匹配魂[：:]\s*(.+)/);
      const soulCount = soulsInMsg ? soulsInMsg[1].split(/[,，、]/).length : 0;
      progressText = `已匹配 ${soulCount} 魂，等待 DeepSeek 回应…`;
    } else if (lastStep) {
      progressText = lastStep.message;
    } else {
      progressText = "初始化中…";
    }
  } else if (status === "done" && hasMessages) {
    progressText = "附体完成";
  } else if (status === "error") {
    progressText = "连接中断";
  }

  if (status === "done" && hasMessages && onDone && !sessionDone) {
    onDone();
  }

  return (
    <div className="flex flex-col flex-1" data-testid="session-runner">
      <Link href="/possess" className="text-sm text-muted-foreground hover:text-foreground mb-2">
        ← 返回讨论
      </Link>
      <SessionStatusBar status={status} error={error} onReconnect={reconnect} />

      {progressText && (
        <div className={cn(
          "px-4 py-2 text-sm border-b transition-colors",
          status === "done" ? "bg-green-50 dark:bg-green-950/20 text-green-700 dark:text-green-300 border-green-200 dark:border-green-800" :
          status === "error" ? "bg-red-50 dark:bg-red-950/20 text-red-700 dark:text-red-300 border-red-200 dark:border-red-800" :
          "bg-primary/5 text-primary border-primary/20"
        )}>
          <span className="inline-flex items-center gap-2">
            {status === "streaming" && <Loader2 className="h-3 w-3 animate-spin" />}
            {status === "done" && <CheckCircle className="h-3 w-3 text-green-500" />}
            {status === "error" && <AlertTriangle className="h-3 w-3 text-red-500" />}
            <span className="font-medium">{progressText}</span>
          </span>
        </div>
      )}

      <div className="flex flex-1 overflow-hidden">
        {(status === "streaming" || status === "done") && (
          <ProcessTimeline steps={processSteps} />
        )}
        <div className="flex-1 overflow-hidden flex flex-col">
          {status === "connecting" && !hasMessages && <ConnectingView />}
          {status === "streaming" && <WaitingSoulsView mode={mode} matchedSouls={matchedSouls} processSteps={processSteps} messages={messages} />}
          {status === "error" && !hasMessages && <ErrorView error={error || "未知错误"} onReconnect={reconnect} />}
          {status === "done" && error && !hasMessages && <ErrorView error={error} onReconnect={reconnect} />}
          {status === "done" && !error && !hasMessages && <RequireApiKeyView />}
          {status === "done" && hasMessages && mode === "single" && <SingleView messages={messages} />}
          {status === "done" && hasMessages && mode === "conference" && <ConferenceView messages={messages} synthesis={synthesis} collisions={collisions} cost={cost} toolCalls={toolCalls} />}
          {status === "done" && hasMessages && mode === "debate" && <DebateView messages={messages} />}
          {status === "done" && hasMessages && mode === "relay" && <RelayView messages={messages} />}
          {status === "done" && hasMessages && mode === "learn" && <LearnView messages={messages} />}
          {status === "done" && hasMessages && mode === "practice_opening" && <PracticeOpeningView messages={messages} />}
        </div>
      </div>

      {status === "done" && hasMessages && (
        <div className="border-t p-4 space-y-6">
          {!reviewDone ? (
            <PostSessionReview
              sessionId={sessionId}
              onComplete={() => setReviewDone(true)}
            />
          ) : (
            <FollowUpInput sessionId={sessionId} />
          )}
        </div>
      )}

      {logs.length > 0 && (
        <div className="border-t bg-muted/20 p-4">
          <h4 className="text-sm font-semibold text-muted-foreground mb-3 flex items-center gap-2">
            <span>执行日志</span>
            <span className="text-xs bg-muted px-2 py-0.5 rounded-full">{logs.length} 条</span>
          </h4>
          <div className="bg-background rounded-lg p-3 max-h-48 overflow-y-auto font-mono text-xs space-y-1">
            {logs.map((log: LogEntry, i: number) => (
              <p
                key={i}
                className={cn(
                  "break-all leading-relaxed",
                  log.type === "success" ? "text-green-600 dark:text-green-400" :
                  log.type === "error" ? "text-red-600 dark:text-red-400" :
                  log.type === "warning" ? "text-yellow-600 dark:text-yellow-400" :
                  "text-muted-foreground"
                )}
              >
                [{log.timestamp.toLocaleTimeString()}] {log.message}
              </p>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
