"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { SessionRunner } from "@/components/session-runner";
import FollowUpInput from "@/components/follow-up-input";
import { 
  Brain, Loader2, Sparkles, ShieldCheck, Zap, Play, ChevronDown, ChevronUp,
  CheckCircle2, AlertCircle, ArrowRightCircle, Globe, Search, Copy, Check
} from "lucide-react";
import { analyzeTask, startPossession, searchWeb, type SearxngResultItem } from "@/lib/api";
import { MODE_LABELS_LONG } from "@/config/possession-modes";
import { triggerSessionsUpdate } from "@/components/sidebar-sessions";
import { AttachmentUpload } from "@/components/attachment-upload";
import { PostSessionReview } from "@/components/post-session-review";
import { SoulCarousel } from "@/components/soul-carousel";
import { PracticeOpeningDialog } from "@/components/practice-opening-dialog";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { cn } from "@/lib/utils";

type Phase = "input" | "classifying" | "matching" | "reviewing" | "adjusting" | "starting" | "running" | "practice_opening";

const PHASES: { key: Phase; icon: React.ComponentType<{ className?: string }>; label: string; desc: string }[] = [
  { key: "classifying", icon: Brain, label: "入口分流", desc: "分析任务类型" },
  { key: "matching", icon: Sparkles, label: "匹配魂", desc: "智能匹配思想者" },
  { key: "reviewing", icon: ShieldCheck, label: "审查", desc: "幡主审查魂组合" },
  { key: "adjusting", icon: Zap, label: "调整", desc: "优化魂搭配" },
  { key: "starting", icon: Play, label: "启动", desc: "启动讨论会话" },
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

const LOG_FILTERS = ["全部", "关键", "魂匹配", "审查"] as const;
type LogFilter = typeof LOG_FILTERS[number];

function classifyLogType(line: string): "key" | "soul" | "review" | "other" {
  if (line.includes("🚀") || line.includes("🎉") || line.includes("❌") || line.includes("⏹️")) return "key";
  if (line.includes("魂") || line.includes("匹配")) return "soul";
  if (line.includes("审查") || line.includes("幡主")) return "review";
  return "other";
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
  const [showDetail, setShowDetail] = useState(true);
  const [sessionDone, setSessionDone] = useState(false);
  const [reviewDone, setReviewDone] = useState(false);
  const [isCancelled, setIsCancelled] = useState(false);
  const [searchTopic, setSearchTopic] = useState(true);
  const [taskCards, setTaskCards] = useState<Record<string, string>>({});
  const [judgment, setJudgment] = useState("");
  const [worry, setWorry] = useState("");
  const [unknown, setUnknown] = useState("");
  const [showPresetsDialog, setShowPresetsDialog] = useState(false);
  const [progressLine, setProgressLine] = useState("");
  const [logFilter, setLogFilter] = useState<LogFilter>("全部");
  const [logsCollapsed, setLogsCollapsed] = useState(true);
  const [copiedLogs, setCopiedLogs] = useState(false);
  const [sessionStats, setSessionStats] = useState<{ elapsed: number; soulCount: number; synthesisLen: number } | null>(null);
  const [startTime, setStartTime] = useState(0);

  const [searchQuery, setSearchQuery] = useState("");
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchResults, setSearchResults] = useState<SearxngResultItem[]>([]);
  const [searchContext, setSearchContext] = useState("");
  const [showSearch, setShowSearch] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const logEndRef = useRef<HTMLDivElement>(null);

  const addLog = useCallback((msg: string) => {
    setLog((p) => [...p, `[${new Date().toLocaleTimeString()}] ${msg}`]);
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [log]);

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

  const handleSearch = useCallback(async () => {
    if (!searchQuery.trim()) return;
    setSearchLoading(true);
    setSearchResults([]);
    try {
      const data = await searchWeb({ q: searchQuery, language: "zh" });
      setSearchResults(data.results);
    } catch (e) {
      console.error("搜索失败:", e);
    } finally {
      setSearchLoading(false);
    }
  }, [searchQuery]);

  const applyResultContext = useCallback((result: SearxngResultItem) => {
    const ctx = `## ${result.title}\n${result.content || ""}\n来源: ${result.url}`;
    setSearchContext(ctx);
    setTask((prev) => {
      const header = "> 以下是通过 SearXNG 搜索获取的背景信息\n\n";
      const trimmed = prev.replace(/^> 以下是通过 SearXNG 搜索获取的背景信息\n\n[\s\S]*?\n\n---\n\n/gm, "").trim();
      return `${header}${ctx}\n\n---\n\n${trimmed}`;
    });
    setShowSearch(false);
    setSearchResults([]);
  }, []);

  const clearSearchContext = useCallback(() => {
    setSearchContext("");
    setTask((prev) => prev.replace(/^> 以下是通过 SearXNG 搜索获取的背景信息\n\n[\s\S]*?\n\n---\n\n/gm, "").trim());
  }, []);

  const canStart = task.trim().length > 0;

  const onStart = async () => {
    if (!canStart || phase !== "input") return;
    
    console.log("=== 开始讨论流程");
    setIsCancelled(false);
    setPhase("classifying");
    setLog([]);
    setError("");
    setMatchedSouls([]);
    setReview(null);
    setSessionDone(false);
    setSessionStats(null);
    setStartTime(Date.now());

    try {
      addLog("开始分析任务...");
      setProgressLine("正在分析任务，入口分流中…");
      setPhase("matching");
      console.log("=== 调用 analyzeTask API...");
      const reviewer = localStorage.getItem("aionui-banner-lord") || undefined;
      const data = await analyzeTask(task, reviewer);
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
        setProgressLine("检测到实践者在场，进入实践开口流程");
        return;
      }

      const souls = data.matched_souls || [];
      const matchedMode = data.recommended_mode || "conference";
      const reviewData = data.review || {};
      const cards = data.task_cards || {};
      setMatchedSouls(souls);
      setMode(matchedMode);
      setReview(reviewData);
      setTaskCards(cards);
      
      addLog(`✅ 匹配完成: ${souls.length} 个魂`);
      addLog(`推荐模式: ${getModeLabel(matchedMode)}`);
      setProgressLine(`已匹配 ${souls.length} 个魂：${souls.map((s: any) => s.name).join("、")}`);
      setPhase("reviewing");

      if (reviewData.reviewer) {
        setProgressLine(`审查官 ${reviewData.reviewer} 正在审查魂组合…`);
        addLog(`🔍 审查官: ${reviewData.reviewer} | 裁决: ${getVerdictLabel(reviewData.verdict)}`);
        (reviewData.checks || []).forEach((c: string) => addLog(`   - ${c}`));
        if (reviewData.notes) {
          addLog(`📝 备注: ${reviewData.notes}`);
        }
      }

      if (reviewData.verdict === "reject") {
        setPhase("adjusting");
        addLog("🔄 审查未通过 → 重新调整魂组合...");
        setProgressLine("审查未通过，正在调整魂组合…");
      }

      if (isCancelled) {
        console.log("=== 用户取消");
        setPhase("input");
        return;
      }

      setPhase("starting");
      addLog("🚀 启动附体会话...");
      setProgressLine("正在启动附体会话…");

      console.log("=== 调用 startPossession API...");
      const { session_id } = await startPossession({
        mode: matchedMode, task, souls: souls.map((s: any) => s.name),
        task_cards: Object.keys(cards).length > 0 ? cards : undefined,
        search_topic: searchTopic,
        judgment: judgment || undefined,
        worry: worry || undefined,
        unknown: unknown || undefined,
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
      setProgressLine("附体会话已启动，魂正在思考…");
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
    setProgressLine("");
    addLog("⏹️ 用户取消了操作");
  };

  const handlePracticeOpeningStart = async (j: string, w: string, u: string) => {
    setJudgment(j);
    setWorry(w);
    setUnknown(u);
    setPhase("starting");
    setProgressLine("正在启动实践开口流程…");
    addLog("🚀 启动实践开口附体会话...");

    try {
      const { session_id } = await startPossession({
        mode: "practice_opening",
        task,
        souls: [],
        judgment: j || undefined,
        worry: w || undefined,
        unknown: u || undefined,
      });
      setSessionId(session_id);
      setMode("practice_opening");
      setPhase("running");
      addLog("🎉 实践开口附体会话已启动");
      setProgressLine("实践开口进行中…");
      triggerSessionsUpdate();
    } catch (e: any) {
      const errorMsg = e.message || e.toString() || "启动失败";
      setError(errorMsg);
      addLog(`❌ 错误: ${errorMsg}`);
      setPhase("input");
    }
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

  const copyLogs = () => {
    const text = log.join("\n");
    navigator.clipboard.writeText(text).then(() => {
      setCopiedLogs(true);
      setTimeout(() => setCopiedLogs(false), 2000);
    }).catch(() => {});
  };

  const handleSessionDone = () => {
    setSessionDone(true);
    const elapsed = Math.round((Date.now() - startTime) / 1000);
    setSessionStats({
      elapsed,
      soulCount: matchedSouls.length,
      synthesisLen: 0,
    });
  };

  const filteredLogs = log.filter((l) => {
    if (logFilter === "全部") return true;
    const type = classifyLogType(l);
    if (logFilter === "关键") return type === "key";
    if (logFilter === "魂匹配") return type === "soul";
    if (logFilter === "审查") return type === "review";
    return true;
  });

  const presetsFilled = (judgment.trim() ? 1 : 0) + (worry.trim() ? 1 : 0) + (unknown.trim() ? 1 : 0);
  const presetsTitle = presetsFilled === 0 ? "使用者预设 ⚠️ 请填写判断" :
    presetsFilled === 3 ? "使用者预设 ✅" :
    `使用者预设（${presetsFilled}/3 完成）`;

  const currentPhaseIndex = PHASES.findIndex((p) => p.key === phase);
  const totalPhases = PHASES.length;

  if (phase === "running" && sessionId) {
    return (
      <div className="max-w-4xl mx-auto space-y-4 animate-in fade-in duration-500" data-testid="possession-entry">
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

        <SessionRunner
          sessionId={sessionId}
          mode={mode}
          matchedSouls={matchedSouls}
          onDone={handleSessionDone}
          onReview={() => setPhase("input")}
          sessionDone={sessionDone}
        />
        
        {sessionDone && (
          <div className="mt-4 space-y-6">
            {sessionStats && (
              <div className="rounded-xl border bg-gradient-to-br from-emerald-50 to-background dark:from-emerald-950/20 p-5 animate-in fade-in slide-in-from-top-4 duration-500">
                <h3 className="text-sm font-semibold text-emerald-700 dark:text-emerald-300 mb-3 flex items-center gap-2">
                  <CheckCircle2 className="h-4 w-4" />
                  讨论完成
                </h3>
                <div className="flex items-center gap-6 text-sm text-muted-foreground">
                  <div className="flex items-center gap-1.5">
                    <span className="text-emerald-600 dark:text-emerald-400 font-mono font-medium">{sessionStats.elapsed}s</span>
                    <span>耗时</span>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <span className="text-emerald-600 dark:text-emerald-400 font-mono font-medium">{sessionStats.soulCount}</span>
                    <span>个魂参与</span>
                  </div>
                </div>
              </div>
            )}
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
      </div>
    );
  }

  const activePhases = PHASES.filter((p, i) => {
    const idx = currentPhaseIndex;
    return i <= idx && idx >= 0;
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

              {searchContext && (
                <div className="flex items-center gap-2 text-xs text-muted-foreground bg-muted/30 rounded-lg px-3 py-1.5">
                  <Globe className="h-3.5 w-3.5 text-green-500" />
                  <span>已添加 SearXNG 搜索背景</span>
                  <button onClick={clearSearchContext} className="ml-auto text-destructive hover:underline">
                    清除
                  </button>
                </div>
              )}

              {!showSearch && (
                <button
                  onClick={() => setShowSearch(true)}
                  className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  <Search className="h-4 w-4" />
                  搜索背景资料（通过 SearXNG）
                </button>
              )}

              {showSearch && (
                <div className="rounded-lg border bg-muted/20 p-4 space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">SearXNG 搜索背景</span>
                    <button
                      onClick={() => { setShowSearch(false); setSearchResults([]); }}
                      className="text-xs text-muted-foreground hover:text-foreground"
                    >
                      收起
                    </button>
                  </div>
                  <div className="flex gap-2">
                    <div className="relative flex-1">
                      <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                      <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        onKeyDown={(e) => { if (e.key === "Enter") handleSearch(); }}
                        placeholder={task ? `搜索: ${task.slice(0, 40)}...` : "输入搜索关键词..."}
                        className="w-full rounded-lg border bg-background pl-10 pr-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/30"
                      />
                    </div>
                    <Button
                      onClick={handleSearch}
                      disabled={searchLoading || !searchQuery.trim()}
                      size="sm"
                    >
                      {searchLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
                      搜索
                    </Button>
                  </div>

                  {searchResults.length > 0 && (
                    <>
                      <div className="max-h-64 overflow-y-auto space-y-2">
                        {searchResults.slice(0, 8).map((r, i) => (
                          <div
                            key={i}
                            className="rounded-lg border border-transparent bg-background hover:border-primary/20 p-3 transition-all text-sm group"
                          >
                            <div className="flex items-start justify-between gap-2">
                              <div className="flex-1 min-w-0">
                                <a
                                  href={r.url}
                                  target="_blank"
                                  rel="noopener noreferrer"
                                  className="font-medium line-clamp-1 hover:underline text-primary"
                                >
                                  {r.title}
                                </a>
                                {r.content && (
                                  <p className="text-xs text-muted-foreground mt-1 line-clamp-2">{r.content}</p>
                                )}
                              </div>
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => applyResultContext(r)}
                                className="shrink-0 h-7 text-xs opacity-0 group-hover:opacity-100 transition-opacity"
                              >
                                引用
                              </Button>
                            </div>
                          </div>
                        ))}
                      </div>
                      <div className="text-xs text-muted-foreground pt-1">
                        共 {searchResults.length} 条结果，点击「引用」添加到问题背景
                      </div>
                    </>
                  )}
                </div>
              )}

              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <label className="flex items-center gap-2 cursor-pointer select-none">
                    <input
                      type="checkbox"
                      checked={searchTopic}
                      onChange={(e) => setSearchTopic(e.target.checked)}
                      className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                    />
                    <Globe className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm text-muted-foreground">启动时搜索议题背景</span>
                  </label>
                </div>
                <div className="flex flex-col items-end gap-1">
                  <Button 
                    onClick={onStart} 
                    disabled={!canStart} 
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
          </div>
        </div>
      )}

      {phase === "practice_opening" && (
        <PracticeOpeningDialog
          open={true}
          onStart={handlePracticeOpeningStart}
          onCancel={() => setPhase("input")}
        />
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
              <div className="rounded-xl border bg-background p-6 shadow-sm">
                <h2 className="text-lg font-semibold flex items-center gap-2 mb-2">
                  <Loader2 className="h-5 w-5 animate-spin text-primary" />
                  讨论流程
                </h2>

                <div className="mb-1 mt-3">
                  <div className="flex items-center justify-between text-xs text-muted-foreground mb-1.5">
                    <span>{currentPhaseIndex + 1}/{totalPhases} 步骤</span>
                    <span>
                      {currentPhaseIndex >= 0 && currentPhaseIndex < totalPhases
                        ? PHASES[currentPhaseIndex].desc
                        : ""}
                    </span>
                  </div>
                  <div className="h-2 bg-muted rounded-full overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-primary to-emerald-500 transition-all duration-700 ease-out rounded-full"
                      style={{ width: `${Math.round(((currentPhaseIndex + 1) / totalPhases) * 100)}%` }}
                    />
                  </div>
                </div>

                {progressLine && (
                  <p className="text-sm text-primary mb-4 px-3 py-2 bg-primary/5 rounded-lg border border-primary/20 animate-in fade-in">
                    {progressLine}
                  </p>
                )}

                <div className="flex items-center justify-center gap-0 mt-5 mb-2 overflow-x-auto">
                  {PHASES.map((p, i) => {
                    const Icon = p.icon;
                    const isCurrent = i === currentPhaseIndex;
                    const isDone = i < currentPhaseIndex;
                    const isPending = i > currentPhaseIndex;

                    return (
                      <div key={p.key} className="flex items-center">
                        <div className="flex flex-col items-center gap-1.5">
                          <div className={cn(
                            "flex items-center justify-center w-10 h-10 rounded-full transition-all duration-300",
                            isCurrent ? "bg-primary text-primary-foreground shadow-lg shadow-primary/25 scale-110" :
                            isDone ? "bg-emerald-500 text-white shadow-sm" :
                            "bg-muted text-muted-foreground/40"
                          )}>
                            {isDone ? (
                              <CheckCircle2 className="h-5 w-5" />
                            ) : isCurrent ? (
                              <Loader2 className="h-5 w-5 animate-spin" />
                            ) : (
                              <Icon className="h-4 w-4" />
                            )}
                          </div>
                          <span className={cn(
                            "text-[10px] leading-tight text-center max-w-[60px]",
                            isCurrent ? "text-primary font-semibold" :
                            isDone ? "text-emerald-600 font-medium" :
                            "text-muted-foreground/40"
                          )}>
                            {p.label}
                          </span>
                        </div>
                        {i < PHASES.length - 1 && (
                          <div className={cn(
                            "h-0.5 w-8 sm:w-12 mx-0.5 rounded-full transition-colors duration-500",
                            i < currentPhaseIndex ? "bg-emerald-400" : "bg-muted"
                          )} />
                        )}
                      </div>
                    );
                  })}
                </div>

                {phase === "matching" && (
                  <div className="mt-4">
                    <SoulCarousel />
                  </div>
                )}
                
                <Button 
                  variant="ghost" 
                  size="sm" 
                  onClick={onCancel} 
                  className="mt-4 w-full text-muted-foreground hover:text-destructive"
                >
                  取消
                </Button>
              </div>

              <div className="rounded-xl border bg-muted/20 p-4 shadow-sm">
                <div className="flex items-center justify-between mb-3">
                  <h4 className="text-sm font-semibold text-muted-foreground flex items-center gap-2">
                    <button
                      onClick={() => setLogsCollapsed(!logsCollapsed)}
                      className="flex items-center gap-2 hover:text-foreground transition-colors"
                    >
                      <span>执行日志</span>
                      <span className="text-xs bg-muted px-2 py-0.5 rounded-full">{log.length} 条</span>
                      {logsCollapsed ? <ChevronDown className="h-3 w-3" /> : <ChevronUp className="h-3 w-3" />}
                    </button>
                  </h4>
                  <div className="flex items-center gap-1.5">
                    <div className="flex items-center gap-0.5 bg-muted rounded-md p-0.5">
                      {LOG_FILTERS.map((f) => (
                        <button
                          key={f}
                          onClick={() => setLogFilter(f)}
                          className={cn(
                            "text-[10px] px-2 py-0.5 rounded transition-colors",
                            logFilter === f ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
                          )}
                        >
                          {f}
                        </button>
                      ))}
                    </div>
                    <button
                      onClick={copyLogs}
                      className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
                      title="复制日志"
                    >
                      {copiedLogs ? <Check className="h-3.5 w-3.5 text-emerald-500" /> : <Copy className="h-3.5 w-3.5" />}
                    </button>
                  </div>
                </div>
                {!logsCollapsed && (
                  <div className="bg-background rounded-lg p-3 max-h-64 overflow-y-auto font-mono text-xs space-y-1">
                    {filteredLogs.map((l, i) => {
                      const type = classifyLogType(l);
                      return (
                        <p key={i} className={cn(
                          "break-all leading-relaxed",
                          type === "key" ? "text-foreground font-medium" :
                          type === "soul" ? "text-blue-600 dark:text-blue-400" :
                          type === "review" ? "text-purple-600 dark:text-purple-400" :
                          "text-muted-foreground"
                        )}>
                          {l}
                        </p>
                      );
                    })}
                    {phase === "starting" && (
                      <p className="text-primary animate-pulse flex items-center gap-2">
                        <Loader2 className="h-3 w-3 animate-spin" />
                        正在启动附体会话...
                      </p>
                    )}
                    <div ref={logEndRef} />
                  </div>
                )}
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}
