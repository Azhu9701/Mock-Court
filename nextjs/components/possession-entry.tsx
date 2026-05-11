"use client";

import { useState, useCallback, useRef } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { analyzeTask, startPossession, searchWeb, type SearxngResultItem } from "@/lib/api";
import { MODE_LABELS_LONG } from "@/config/possession-modes";
import { triggerSessionsUpdate } from "@/components/sidebar-sessions";
import { AttachmentUpload } from "@/components/attachment-upload";
import { PracticeOpeningDialog } from "@/components/practice-opening-dialog";
import { storePendingSession } from "@/lib/pending-session";
import { cn } from "@/lib/utils";
import {
  Brain, Loader2, Globe, Search, AlertCircle,
  Sparkles, ShieldCheck, Zap, Play, CheckCircle2, ChevronDown, ChevronUp,
} from "lucide-react";

type Phase = "input" | "classifying" | "matching" | "reviewing" | "adjusting" | "starting" | "practice_opening";

const PHASES: { key: Phase; icon: React.ComponentType<{ className?: string }>; label: string; desc: string }[] = [
  { key: "classifying", icon: Brain, label: "入口分流", desc: "分析任务类型" },
  { key: "matching", icon: Sparkles, label: "匹配魂", desc: "智能匹配思想者" },
  { key: "reviewing", icon: ShieldCheck, label: "审查", desc: "审查魂组合" },
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

export function PossessionEntry() {
  const router = useRouter();
  const [task, setTask] = useState("");
  const [phase, setPhase] = useState<Phase>("input");
  const [error, setError] = useState("");
  const [searchTopic, setSearchTopic] = useState(true);
  const [judgment, setJudgment] = useState("");
  const [worry, setWorry] = useState("");
  const [unknown, setUnknown] = useState("");
  const [progressLine, setProgressLine] = useState("");
  const [log, setLog] = useState<string[]>([]);

  const [matchedSouls, setMatchedSouls] = useState<MatchedSoul[]>([]);
  const [review, setReview] = useState<ReviewResult | null>(null);
  const [mode, setMode] = useState("conference");
  const [showDetail, setShowDetail] = useState(true);

  const [searchQuery, setSearchQuery] = useState("");
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchResults, setSearchResults] = useState<SearxngResultItem[]>([]);
  const [searchContext, setSearchContext] = useState("");
  const [showSearch, setShowSearch] = useState(false);
  const [isCancelled, setIsCancelled] = useState(false);
  const cancelledRef = useRef(false);

  const canStart = task.trim().length > 0;

  const addLog = useCallback((msg: string) => {
    setLog((p) => [...p, `[${new Date().toLocaleTimeString()}] ${msg}`]);
  }, []);

  const getModeLabel = (m: string) => (MODE_LABELS_LONG as Record<string, string>)[m] || m;

  const getVerdictLabel = (v: string) => {
    const labels: Record<string, string> = { "pass": "✅ 通过", "conditional": "⚠️ 条件通过", "reject": "❌ 拒绝" };
    return labels[v] || v;
  };

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

  const onStart = async () => {
    if (!canStart || phase !== "input") return;

    cancelledRef.current = false;
    setIsCancelled(false);
    setError("");
    setLog([]);
    setMatchedSouls([]);
    setReview(null);
    setShowDetail(true);

    try {
      // ── Step 1: Classify ──
      setPhase("classifying");
      setProgressLine("正在分析任务，入口分流中…");
      addLog("开始分析任务...");

      const reviewer = localStorage.getItem("aionui-banner-lord") || undefined;
      const data = await analyzeTask(task, reviewer);

      if (cancelledRef.current) { setPhase("input"); return; }
      addLog("✅ analyzeTask 完成");

      if (data.entry_type === "practice_opening") {
        setPhase("practice_opening");
        setProgressLine("检测到实践者在场，进入实践开口流程");
        return;
      }

      // ── Step 2: Matching ──
      setPhase("matching");
      const souls = data.matched_souls || [];
      const matchedMode = data.recommended_mode || "conference";
      const cards = data.task_cards || {};
      setMatchedSouls(souls);
      setMode(matchedMode);
      addLog(`✅ 匹配完成: ${souls.length} 个魂`);
      addLog(`推荐模式: ${getModeLabel(matchedMode)}`);
      setProgressLine(`已匹配 ${souls.length} 个魂：${souls.map((s) => s.name).join("、")}`);

      // ── Step 3: Review ──
      setPhase("reviewing");
      const reviewData = data.review || {};
      setReview(reviewData);

      if (reviewData.reviewer) {
        setProgressLine(`审查官 ${reviewData.reviewer} 正在审查魂组合…`);
        addLog(`🔍 审查官: ${reviewData.reviewer} | 裁决: ${getVerdictLabel(reviewData.verdict)}`);
        (reviewData.checks || []).forEach((c: string) => addLog(`   - ${c}`));
        if (reviewData.notes) addLog(`📝 备注: ${reviewData.notes}`);
      }

      if (reviewData.verdict === "reject") {
        setPhase("adjusting");
        addLog("🔄 审查未通过 → 重新调整魂组合...");
        setProgressLine("审查未通过，正在调整魂组合…");
      }

      if (cancelledRef.current) { setPhase("input"); return; }

      // ── Step 4: Start ──
      setPhase("starting");
      addLog("🚀 启动附体会话...");
      setProgressLine("正在启动附体会话…");

      const { session_id } = await startPossession({
        mode: matchedMode,
        task,
        souls: souls.map((s) => s.name),
        task_cards: Object.keys(cards).length > 0 ? cards : undefined,
        search_topic: searchTopic,
        judgment: judgment || undefined,
        worry: worry || undefined,
        unknown: unknown || undefined,
      });

      if (cancelledRef.current) { setPhase("input"); return; }

      addLog("🎉 附体会话已启动");

      storePendingSession({
        sessionId: session_id,
        task,
        mode: matchedMode,
        matchedSouls: souls,
        review: reviewData.reviewer ? reviewData : null,
      });

      triggerSessionsUpdate();
      router.push(`/sessions/${session_id}`);
    } catch (e: any) {
      console.error("启动失败:", e);
      if (!cancelledRef.current) {
        const errorMsg = e.message || e.toString() || "启动失败";
        setError(errorMsg);
        addLog(`❌ 错误: ${errorMsg}`);
        setPhase("input");
      }
    }
  };

  const onCancel = () => {
    cancelledRef.current = true;
    setIsCancelled(true);
    setPhase("input");
    setProgressLine("");
    addLog("⏹️ 用户取消了操作");
  };

  // ── Flow view ──
  if (phase !== "input" && phase !== "practice_opening") {
    const currentPhaseIndex = PHASES.findIndex((p) => p.key === phase);
    const totalPhases = PHASES.length;

    return (
      <div className="max-w-2xl mx-auto space-y-6" data-testid="possession-entry">
        {error && (
          <div className="rounded-xl border-2 border-red-200 bg-red-50 dark:bg-red-950/50 p-6 text-center shadow-sm">
            <AlertCircle className="h-10 w-10 text-red-500 mx-auto mb-3" />
            <h3 className="font-semibold text-lg text-red-700 dark:text-red-300 mb-2">发生错误</h3>
            <p className="text-red-600 dark:text-red-400">{error}</p>
            <Button variant="outline" className="mt-4" onClick={() => setPhase("input")}>
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
                  <span>{Math.min(currentPhaseIndex + 1, totalPhases)}/{totalPhases} 步骤</span>
                  <span>{currentPhaseIndex >= 0 && currentPhaseIndex < totalPhases ? PHASES[currentPhaseIndex].desc : ""}</span>
                </div>
                <div className="h-2 bg-muted rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-primary to-emerald-500 transition-all duration-700 ease-out rounded-full"
                    style={{ width: `${Math.round(((currentPhaseIndex + 1) / totalPhases) * 100)}%` }}
                  />
                </div>
              </div>

              <div className="flex items-center justify-center gap-0 mt-5 mb-2 overflow-x-auto">
                {PHASES.map((p, i) => {
                  const Icon = p.icon;
                  const isCurrent = i === currentPhaseIndex;
                  const isDone = i < currentPhaseIndex;
                  return (
                    <div key={p.key} className="flex items-center">
                      <div className="flex flex-col items-center gap-1.5">
                        <div className={cn(
                          "flex items-center justify-center w-10 h-10 rounded-full transition-all duration-300",
                          isCurrent ? "bg-primary text-primary-foreground shadow-lg shadow-primary/25 scale-110" :
                          isDone ? "bg-emerald-500 text-white shadow-sm" :
                          "bg-muted text-muted-foreground/40"
                        )}>
                          {isDone ? <CheckCircle2 className="h-5 w-5" /> :
                           isCurrent ? <Loader2 className="h-5 w-5 animate-spin" /> :
                           <Icon className="h-4 w-4" />}
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

              <Button
                variant="ghost"
                size="sm"
                onClick={onCancel}
                className="mt-4 w-full text-muted-foreground hover:text-destructive"
              >
                取消
              </Button>
            </div>

            {/* Matched souls preview */}
            {matchedSouls.length > 0 && (
              <div className="rounded-xl border bg-gradient-to-br from-muted/40 to-background p-4 shadow-sm">
                <div className="flex items-center justify-between">
                  <div>
                    <h2 className="text-lg font-semibold">{task}</h2>
                    <p className="text-sm text-muted-foreground mt-1 flex items-center gap-2">
                      <span>模式：{getModeLabel(mode)}</span>
                      <span>·</span>
                      <span>{matchedSouls.length} 魂</span>
                      {review?.reviewer && (
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
                  <Button variant="ghost" size="sm" onClick={() => setShowDetail(!showDetail)}>
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

                    {review?.reviewer && (
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
                                <span>→</span>
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
            )}

            {/* Log */}
            <div className="rounded-xl border bg-muted/20 p-4 shadow-sm">
              <h4 className="text-sm font-semibold text-muted-foreground mb-3">
                执行日志 <span className="text-xs bg-muted px-2 py-0.5 rounded-full">{log.length} 条</span>
              </h4>
              <div className="bg-background rounded-lg p-3 max-h-48 overflow-y-auto font-mono text-xs space-y-1">
                {progressLine && (
                  <div className="text-primary font-medium flex items-center gap-2 pb-2 mb-2 border-b border-primary/10">
                    <Loader2 className="h-3 w-3 animate-spin shrink-0" />
                    <span>{progressLine}</span>
                  </div>
                )}
                {log.map((l, i) => (
                  <p key={i} className={cn(
                    "break-all leading-relaxed",
                    l.includes("🚀") || l.includes("🎉") || l.includes("❌") ? "text-foreground font-medium" :
                    l.includes("魂") || l.includes("匹配") ? "text-blue-600 dark:text-blue-400" :
                    l.includes("审查") ? "text-purple-600 dark:text-purple-400" :
                    "text-muted-foreground"
                  )}>
                    {l}
                  </p>
                ))}
              </div>
            </div>
          </>
        )}
      </div>
    );
  }

  // ── Practice opening view ──
  if (phase === "practice_opening") {
    return (
      <PracticeOpeningDialog
        open={true}
        onStart={async (j: string, w: string, u: string) => {
          setJudgment(j); setWorry(w); setUnknown(u);
          setProgressLine("正在启动实践开口流程…");
          addLog("🚀 启动实践开口附体会话...");
          try {
            const { session_id } = await startPossession({
              mode: "practice_opening", task, souls: [],
              judgment: j || undefined, worry: w || undefined, unknown: u || undefined,
            });
            storePendingSession({ sessionId: session_id, task, mode: "practice_opening", matchedSouls: [], review: null });
            triggerSessionsUpdate();
            router.push(`/sessions/${session_id}`);
          } catch (e: any) {
            setError(e.message || "启动失败");
            setPhase("input");
          }
        }}
        onCancel={() => setPhase("input")}
      />
    );
  }

  // ── Input view ──
  return (
    <div className="max-w-2xl mx-auto space-y-6" data-testid="possession-entry">
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
                <button onClick={clearSearchContext} className="ml-auto text-destructive hover:underline">清除</button>
              </div>
            )}

            {!showSearch && (
              <button onClick={() => setShowSearch(true)} className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors">
                <Search className="h-4 w-4" />
                搜索背景资料（通过 SearXNG）
              </button>
            )}

            {showSearch && (
              <div className="rounded-lg border bg-muted/20 p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">SearXNG 搜索背景</span>
                  <button onClick={() => { setShowSearch(false); setSearchResults([]); }} className="text-xs text-muted-foreground hover:text-foreground">收起</button>
                </div>
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <input type="text" value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)}
                      onKeyDown={(e) => { if (e.key === "Enter") handleSearch(); }}
                      placeholder={task ? `搜索: ${task.slice(0, 40)}...` : "输入搜索关键词..."}
                      className="w-full rounded-lg border bg-background pl-10 pr-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/30"
                    />
                  </div>
                  <Button onClick={handleSearch} disabled={searchLoading || !searchQuery.trim()} size="sm">
                    {searchLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
                    搜索
                  </Button>
                </div>
                {searchResults.length > 0 && (
                  <>
                    <div className="max-h-64 overflow-y-auto space-y-2">
                      {searchResults.slice(0, 8).map((r, i) => (
                        <div key={i} className="rounded-lg border border-transparent bg-background hover:border-primary/20 p-3 transition-all text-sm group">
                          <div className="flex items-start justify-between gap-2">
                            <div className="flex-1 min-w-0">
                              <a href={r.url} target="_blank" rel="noopener noreferrer" className="font-medium line-clamp-1 hover:underline text-primary">{r.title}</a>
                              {r.content && <p className="text-xs text-muted-foreground mt-1 line-clamp-2">{r.content}</p>}
                            </div>
                            <Button size="sm" variant="outline" onClick={() => applyResultContext(r)} className="shrink-0 h-7 text-xs opacity-0 group-hover:opacity-100 transition-opacity">引用</Button>
                          </div>
                        </div>
                      ))}
                    </div>
                    <div className="text-xs text-muted-foreground pt-1">共 {searchResults.length} 条结果，点击「引用」添加到问题背景</div>
                  </>
                )}
              </div>
            )}

            <div className="flex items-center justify-between">
              <label className="flex items-center gap-2 cursor-pointer select-none">
                <input type="checkbox" checked={searchTopic} onChange={(e) => setSearchTopic(e.target.checked)}
                  className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary" />
                <Globe className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm text-muted-foreground">启动时搜索议题背景</span>
              </label>
              <Button onClick={onStart} disabled={!canStart} className="w-full sm:w-auto" size="lg" data-testid="start-possession-btn">
                <Brain className="mr-2 h-5 w-5" />
                我想问
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
