"use client";

import { useWebSocket, type ProcessStep, type LogEntry } from "@/hooks/use-websocket";
import { SingleView } from "@/components/single-view";
import { ConferenceView } from "@/components/conference-view";
import { DebateView } from "@/components/debate-view";
import { RelayView } from "@/components/relay-view";
import { LearnView } from "@/components/learn-view";
import { PracticeOpeningView } from "@/components/practice-opening-view";
import { SessionStatusBar } from "@/components/session-status-bar";
import { Brain, Loader2, AlertTriangle, Key, CheckCircle, Sparkles, Wifi, Zap, MessageCircle, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";

interface SessionRunnerProps {
  sessionId: string;
  mode: string;
  onDone?: () => void;
  onReview?: () => void;
  sessionDone?: boolean;
}

import { modeLabel } from "@/config/possession-modes";

const stepIcons: Record<string, React.ComponentType<{className?: string}>> = {
  Connected: Wifi,
  SessionStarted: Zap,
  EntryClassified: Brain,
  SoulStarted: Sparkles,
  SynthesisStarted: Brain,
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
          <div key={i} className={cn("flex items-center justify-center h-7 w-7 rounded-full bg-background border", color)} title={step.soulName || step.message}>
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

function WaitingSoulsView({ mode }: { mode: string }) {
  return (
    <div className="flex flex-col items-center justify-center flex-1 gap-4 text-muted-foreground">
      <Brain className="h-8 w-8 text-primary animate-pulse" />
      <div className="text-center space-y-2">
        <p className="text-lg font-medium">魂正在思考...</p>
        <p className="text-sm">模式：{modeLabel(mode)} | 正在调用 AI 生成回应</p>
        <p className="text-xs">首次调用可能需要 5-10 秒，请耐心等待</p>
      </div>
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

export function SessionRunner({ sessionId, mode, onDone, onReview, sessionDone }: SessionRunnerProps) {
  const { messages, synthesis, status, error, processSteps, cost, collisions, logs, reconnect } =
    useWebSocket(sessionId);

  const hasMessages = Object.keys(messages).length > 0;

  if (status === "done" && hasMessages && onDone && !sessionDone) {
    onDone();
  }

  return (
    <div className="flex flex-col flex-1" data-testid="session-runner">
      <SessionStatusBar status={status} error={error} onReconnect={reconnect} />
      <div className="flex flex-1 overflow-hidden">
        {/* Process Timeline - left sidebar during streaming */}
        {(status === "streaming" || status === "done") && (
          <ProcessTimeline steps={processSteps} />
        )}
        {/* Main content */}
        <div className="flex-1 overflow-hidden flex flex-col">
          {status === "connecting" && !hasMessages && <ConnectingView />}
          {status === "streaming" && !hasMessages && <WaitingSoulsView mode={mode} />}
          {status === "error" && !hasMessages && <ErrorView error={error || "未知错误"} onReconnect={reconnect} />}
          {status === "done" && !hasMessages && <RequireApiKeyView />}
          {hasMessages && mode === "single" && <SingleView messages={messages} />}
          {hasMessages && mode === "conference" && <ConferenceView messages={messages} synthesis={synthesis} collisions={collisions} cost={cost} />}
          {hasMessages && mode === "debate" && <DebateView messages={messages} />}
          {hasMessages && mode === "relay" && <RelayView messages={messages} />}
          {hasMessages && mode === "learn" && <LearnView messages={messages} />}
          {hasMessages && mode === "practice_opening" && <PracticeOpeningView messages={messages} />}
        </div>
      </div>

      {status === "done" && hasMessages && onReview && (
        <div className="border-t bg-green-50 dark:bg-green-950 p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-green-700 dark:text-green-300">附体完成</p>
              <p className="text-xs text-green-600 dark:text-green-400">知识不在魂里，在实践里。请完成反馈闭环。</p>
            </div>
            <button onClick={onReview} className="shrink-0 rounded-md bg-green-600 px-4 py-2 text-sm text-white hover:bg-green-700" data-testid="start-review-btn">
              反馈闭环 →
            </button>
          </div>
        </div>
      )}

      {/* Log panel */}
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
