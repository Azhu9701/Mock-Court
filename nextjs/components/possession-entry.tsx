"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { SessionRunner } from "@/components/session-runner";
import FollowUpInput from "@/components/follow-up-input";
import { 
  Brain, Loader2, Sparkles, ShieldCheck, Zap, Play, User, ChevronDown, ChevronUp,
  CheckCircle2, AlertCircle, ArrowRightCircle
} from "lucide-react";
import { analyzeTask, startPossession } from "@/lib/api";
import { MODE_LABELS_LONG } from "@/config/possession-modes";
import { triggerSessionsUpdate } from "@/components/sidebar-sessions";
import { AttachmentUpload } from "@/components/attachment-upload";

type Phase = "input" | "classifying" | "matching" | "reviewing" | "adjusting" | "starting" | "running" | "practice_opening";

const PHASES: { key: Phase; icon: React.ComponentType<{ className?: string }>; label: string }[] = [
  { key: "classifying", icon: Brain, label: "入口分流中..." },
  { key: "matching", icon: Sparkles, label: "智能匹配魂..." },
  { key: "reviewing", icon: ShieldCheck, label: "幡主审查中..." },
  { key: "adjusting", icon: Zap, label: "调整魂组合..." },
  { key: "starting", icon: Play, label: "启动附体..." },
];

interface MatchedSoul {
  name: string;

  field: string;
  ismism_code: string;
  rationale: string;
}

interface ReviewResult {
  verdict: string;
  checks: string[];
  notes: string;
  reviewer: string;
}

export function PossessionEntry() {
  const [task, setTask] = useState("");
  const [phase, setPhase] = useState<Phase>("input");
  const [log, setLog] = useState<string[]>([]);
  const [error, setError] = useState("");
  const [sessionId, setSessionId] = useState("");
  const [mode, setMode] = useState("conference");
  const [matchedSouls, setMatchedSouls] = useState<MatchedSoul[]>([]);
  const [review, setReview] = useState<ReviewResult | null>(null);
  const [showDetail, setShowDetail] = useState(false);
  const [sessionDone, setSessionDone] = useState(false);
  const [isCancelled, setIsCancelled] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);

  const addLog = useCallback((msg: string) => {
    setLog((p) => [...p, `[${new Date().toLocaleTimeString()}] ${msg}`]);
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [log]);

  // 会话完成时触发更新
  useEffect(() => {
    if (sessionDone) {
      triggerSessionsUpdate();
    }
  }, [sessionDone]);

  const handleOcrResults = useCallback((texts: string[]) => {
    const block = texts.join("\n\n");
    setTask((prev) => {
      const trimmed = prev.trim();
      return trimmed ? `${block}\n\n---\n\n${trimmed}` : block;
    });
  }, []);

  const onStart = async () => {
    if (!task.trim() || phase !== "input") return;
    
    console.log("=== 开始讨论流程");
    setIsCancelled(false);
    setPhase("classifying");
    setLog([]);
    setError("");
    setMatchedSouls([]);
    setReview(null);
    setSessionDone(false);

    try {
      addLog("开始分析任务...");
      setPhase("matching");
      console.log("=== 调用 analyzeTask API...");
      const data = await analyzeTask(task);
      console.log("=== analyzeTask 完成:", data);
      addLog("✅ analyzeTask 完成");

      if (isCancelled) {
        console.log("=== 用户取消");
        setPhase("input");
        return;
      }

      if (data.entry_type === "practice_opening") {
        setPhase("practice_opening");
        addLog("✨ 检测到实践者在场 → 进入实践开口流程");
        return;
      }

      const souls = data.matched_souls || [];
      const matchedMode = data.recommended_mode || "conference";
      const reviewData = data.review || {};
      setMatchedSouls(souls);
      setMode(matchedMode);
      setReview(reviewData);
      
      addLog(`✅ 匹配完成: ${souls.length} 个魂`);
      addLog(`推荐模式: ${getModeLabel(matchedMode)}`);
      setPhase("reviewing");

      if (reviewData.reviewer) {
        addLog(`🔍 审查官: ${reviewData.reviewer} | 裁决: ${getVerdictLabel(reviewData.verdict)}`);
        (reviewData.checks || []).forEach((c: string) => addLog(`   - ${c}`));
        if (reviewData.notes) {
          addLog(`📝 备注: ${reviewData.notes}`);
        }
      }

      if (reviewData.verdict === "reject") {
        setPhase("adjusting");
        addLog("🔄 审查未通过 → 重新调整魂组合...");
      }

      if (isCancelled) {
        console.log("=== 用户取消");
        setPhase("input");
        return;
      }

      setPhase("starting");
      addLog("🚀 启动附体会话...");

      console.log("=== 调用 startPossession API...");
      const { session_id } = await startPossession({
        mode: matchedMode, task, souls: souls.map((s: any) => s.name),
      });
      console.log("=== startPossession 完成, session_id:", session_id);
      
      if (isCancelled) {
        console.log("=== 用户取消");
        setPhase("input");
        return;
      }
      
      setSessionId(session_id);
      setPhase("running");
      addLog("🎉 附体会话已启动");
      triggerSessionsUpdate();
    } catch (e: any) {
      console.error("=== 发生错误:", e);
      if (!isCancelled) {
        const errorMsg = e.message || e.toString() || "分析失败";
        setError(errorMsg);
        addLog(`❌ 错误: ${errorMsg}`);
        setPhase("input");
      }
    }
  };

  const onCancel = () => {
    setIsCancelled(true);
    setPhase("input");
    addLog("⏹️ 用户取消了操作");
  };

  const getModeLabel = (m: string) => (MODE_LABELS_LONG as Record<string, string>)[m] || m;

  const getVerdictLabel = (v: string) => {
    const labels: Record<string, string> = {
      "pass": "✅ 通过",
      "conditional": "⚠️ 条件通过",
      "reject": "❌ 拒绝"
    };
    return labels[v] || v;
  };

  if (phase === "running" && sessionId) {
    return (
      <div className="max-w-4xl mx-auto space-y-4" data-testid="possession-entry">
        {/* Header: matched souls + review summary */}
        <div className="rounded-xl border bg-gradient-to-br from-muted/40 to-background p-4 shadow-sm">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold">{task}</h2>
              <p className="text-sm text-muted-foreground mt-1 flex items-center gap-2">
                <span>模式：{getModeLabel(mode)}</span>
                <span>·</span>
                <span>{matchedSouls.length} 魂</span>
                {review && (
                  <>
                    <span>·</span>
                    <span>审查：{review.reviewer}</span>
                    <span className={review.verdict === "pass" ? "text-green-600" : review.verdict === "conditional" ? "text-yellow-600" : "text-red-600"}>
                      [{review.verdict}]
                    </span>
                  </>
                )}
              </p>
            </div>
            <Button variant="ghost" size="sm" onClick={() => setShowDetail(!showDetail)} className="transition-all hover:bg-muted">
              {showDetail ? <ChevronUp className="h-4 w-4 mr-1" /> : <ChevronDown className="h-4 w-4 mr-1" />}
              {showDetail ? "收起" : "详情"}
            </Button>
          </div>

          {showDetail && (
            <div className="mt-4 space-y-4 text-sm border-t pt-4">
              {/* Matched souls */}
              <div>
                <h4 className="font-medium text-muted-foreground mb-3 flex items-center gap-2">
                  <Sparkles className="h-4 w-4" />
                  匹配魂
                </h4>
                <div className="grid gap-3">
                  {matchedSouls.map((s) => (
                    <div key={s.name} className="rounded-lg border p-3 bg-background transition-all hover:shadow-sm">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="font-semibold text-base">{s.name}</span>
                        <span className="text-xs bg-muted px-2 py-0.5 rounded">{s.field}</span>
                        <span className="text-xs text-muted-foreground font-mono">{s.ismism_code}</span>
                      </div>
                      <p className="text-muted-foreground mt-2 text-sm leading-relaxed">{s.rationale}</p>
                    </div>
                  ))}
                </div>
              </div>

              {/* Review */}
              {review && (
                <div>
                  <h4 className="font-medium text-muted-foreground mb-3 flex items-center gap-2">
                    <ShieldCheck className="h-4 w-4" />
                    审查 · {review.reviewer}
                  </h4>
                  <div className={`rounded-lg border p-3 ${
                    review.verdict === "pass" ? "border-green-200 bg-green-50 dark:bg-green-950/20" :
                    review.verdict === "conditional" ? "border-yellow-200 bg-yellow-50 dark:bg-yellow-950/20" :
                    "border-red-200 bg-red-50 dark:bg-red-950/20"
                  }`}>
                    <div className="font-medium mb-2">裁决: {getVerdictLabel(review.verdict)}</div>
                    <ul className="space-y-1">
                      {review.checks.map((c, i) => (
                        <li key={i} className="text-sm flex items-start gap-2">
                          <ArrowRightCircle className="h-4 w-4 mt-0.5 shrink-0 text-muted-foreground" />
                          <span>{c}</span>
                        </li>
                      ))}
                    </ul>
                    {review.notes && (
                      <p className="text-sm mt-2 italic text-muted-foreground border-t pt-2">
                        📝 {review.notes}
                      </p>
                    )}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Live streaming */}
        <SessionRunner
          sessionId={sessionId}
          mode={mode}
          onDone={() => setSessionDone(true)}
          onReview={() => setPhase("input")}
          sessionDone={sessionDone}
        />
        
        {/* Follow-up input (shown after session is done) */}
        {sessionDone && (
          <div className="mt-4">
            <FollowUpInput sessionId={sessionId} />
          </div>
        )}
      </div>
    );
  }

  const activePhases = PHASES.filter((p) => {
    const idx = PHASES.findIndex((x) => x.key === phase);
    const myIdx = PHASES.findIndex((x) => x.key === p.key);
    return myIdx <= idx && idx >= 0;
  });

  return (
    <div className="max-w-2xl mx-auto space-y-6" data-testid="possession-entry">
      {phase === "input" && (
        <div className="space-y-6">
          <div className="text-center space-y-2">
            <h2 className="text-2xl font-bold bg-gradient-to-r from-primary to-purple-600 bg-clip-text text-transparent">
              开始讨论
            </h2>
            <p className="text-sm text-muted-foreground">
              输入你的问题，系统将自动完成全流程
            </p>
          </div>
          
          <div className="rounded-xl border bg-background p-6 shadow-sm">
            <div className="space-y-4">
              <AttachmentUpload onOcrResults={handleOcrResults} />
              <Textarea
                placeholder="描述你的问题或任务..."
                value={task}
                onChange={(e) => setTask(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) onStart(); }}
                rows={5}
                data-testid="task-input"
                className="resize-none transition-all focus:ring-2 focus:ring-primary/20"
              />
              <div className="flex items-center justify-end">
                <Button 
                  onClick={onStart} 
                  disabled={!task.trim()} 
                  className="w-full sm:w-auto" 
                  size="lg" 
                  data-testid="start-possession-btn"
                >
                  <Brain className="mr-2 h-5 w-5" />
                  我想问
                </Button>
              </div>
            </div>
          </div>

          {/* Pipeline preview */}
          <div className="rounded-xl border bg-muted/30 p-4">
            <h3 className="text-sm font-medium mb-3">流程</h3>
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <div className="flex items-center gap-2">
                <Brain className="h-4 w-4" />
                <span>提问</span>
              </div>
              <ArrowRightCircle className="h-4 w-4" />
              <div className="flex items-center gap-2">
                <Sparkles className="h-4 w-4" />
                <span>匹配</span>
              </div>
              <ArrowRightCircle className="h-4 w-4" />
              <div className="flex items-center gap-2">
                <ShieldCheck className="h-4 w-4" />
                <span>审查</span>
              </div>
              <ArrowRightCircle className="h-4 w-4" />
              <div className="flex items-center gap-2">
                <Play className="h-4 w-4" />
                <span>启动</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {phase === "practice_opening" && (
        <div className="space-y-4">
          <div className="rounded-xl border-2 border-orange-300 bg-gradient-to-br from-orange-50 to-orange-100 dark:from-orange-950/50 dark:to-orange-950 p-8 text-center shadow-lg">
            <User className="h-16 w-16 text-orange-500 mx-auto mb-4" />
            <h3 className="font-bold text-2xl text-orange-700 dark:text-orange-300 mb-2">实践开口</h3>
            <p className="text-orange-600 dark:text-orange-400 mb-6">
              检测到在场者经验，进入 P1-P4 实践流程
            </p>
            <Button variant="default" onClick={() => setPhase("input")} className="bg-orange-600 hover:bg-orange-700">
              重新开始
            </Button>
          </div>
        </div>
      )}

      {phase !== "input" && phase !== "practice_opening" && phase !== "running" && (
        <div className="space-y-6">
          {error && (
            <div className="rounded-xl border-2 border-red-200 bg-gradient-to-br from-red-50 to-red-100 dark:from-red-950/50 dark:to-red-950 p-6 text-center shadow-sm">
              <AlertCircle className="h-10 w-10 text-red-500 mx-auto mb-3" />
              <h3 className="font-semibold text-lg text-red-700 dark:text-red-300 mb-2">发生错误</h3>
              <p className="text-red-600 dark:text-red-400">{error}</p>
              <Button variant="outline" className="mt-4 border-red-300 text-red-600 hover:bg-red-50" onClick={() => setPhase("input")}>
                重新开始
              </Button>
            </div>
          )}

          {!error && (
            <>
              {/* Process timeline */}
              <div className="rounded-xl border bg-background p-6 shadow-sm">
                <h2 className="text-lg font-semibold flex items-center gap-2 mb-4">
                  <Loader2 className="h-5 w-5 animate-spin text-primary" />
                  附体流程
                </h2>
                <div className="space-y-3">
                  {activePhases.map((p, i) => {
                    const Icon = p.icon;
                    const isCurrent = p.key === phase;
                    const isDone = activePhases.length > i + 1 || phase === "starting";
                    return (
                      <div key={p.key} className={`flex items-center gap-3 px-4 py-3 rounded-lg text-sm transition-all ${
                        isCurrent 
                          ? "bg-primary/10 text-primary font-medium border border-primary/20 shadow-sm" 
                          : isDone 
                            ? "text-green-600 bg-green-50 dark:bg-green-950/20 border border-green-200 dark:border-green-800" 
                            : "text-muted-foreground bg-muted/30"
                      }`}>
                        <Icon className={`h-5 w-5 shrink-0 ${isCurrent ? "animate-pulse" : ""} ${isDone ? "text-green-500" : ""}`} />
                        <span className="flex-1">{p.label}</span>
                        {isDone && <CheckCircle2 className="h-5 w-5 text-green-500" />}
                        {isCurrent && <Loader2 className="h-5 w-5 animate-spin" />}
                      </div>
                    );
                  })}
                </div>
                
                <Button 
                  variant="ghost" 
                  size="sm" 
                  onClick={onCancel} 
                  className="mt-4 w-full text-muted-foreground hover:text-destructive"
                >
                  取消
                </Button>
              </div>

              {/* Log panel */}
              <div className="rounded-xl border bg-muted/20 p-4 shadow-sm">
                <h4 className="text-sm font-semibold text-muted-foreground mb-3 flex items-center gap-2">
                  <span>执行日志</span>
                  <span className="text-xs bg-muted px-2 py-0.5 rounded-full">{log.length} 条</span>
                </h4>
                <div className="bg-background rounded-lg p-3 max-h-64 overflow-y-auto font-mono text-xs space-y-1">
                  {log.map((l, i) => (
                    <p key={i} className="text-muted-foreground break-all leading-relaxed">
                      {l}
                    </p>
                  ))}
                  {phase === "starting" && (
                    <p className="text-primary animate-pulse flex items-center gap-2">
                      <Loader2 className="h-3 w-3 animate-spin" />
                      正在启动附体会话...
                    </p>
                  )}
                  <div ref={logEndRef} />
                </div>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}
