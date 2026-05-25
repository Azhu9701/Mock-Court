"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { useSearchParams } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { SessionRunner } from "@/components/session-runner";
import { 
  Brain, Loader2, Sparkles, ShieldCheck, Zap, Play, ChevronDown, ChevronUp,
  CheckCircle2, AlertCircle, Globe, Search, Copy, Check,
  MessageCircle, Wand2
} from "lucide-react";
import {
  analyzeTask, startPossession, searchWeb, startInterrogation, submitInterrogation,
  type SearxngResultItem, type InterrogationQuestion, type InterrogationVerdictResponse,
} from "@/lib/api";
import { 
  MODE_LABELS_LONG, 
  type PossessionMode,
  MODES,
  MODE_COLORS_BG,
  MODE_COLORS_TEXT,
  iconMap
} from "@/config/possession-modes";
import { triggerSessionsUpdate } from "@/components/sidebar-sessions";
import { AttachmentUpload } from "@/components/attachment-upload";
import { SoulCarousel } from "@/components/soul-carousel";
import { cn } from "@/lib/utils";
import { SessionContextHeader } from "@/components/session-context-header";

type Phase = "input" | "interrogation" | "classifying" | "matching" | "reviewing" | "adjusting" | "starting" | "running";

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
  const searchParams = useSearchParams();
  const initialTaskFromUrl = searchParams?.get("task") || "";
  const initialSoulsFromUrl = (searchParams?.get("souls") || "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  const [task, setTask] = useState(initialTaskFromUrl);
  const prefilledSouls = initialSoulsFromUrl;
  const [taskHistory, setTaskHistory] = useState<string[]>(() => {
    if (typeof window === "undefined") return [];
    try { return JSON.parse(localStorage.getItem("possess-task-history") || "[]"); } catch { return []; }
  });
  const [taskHistoryIdx, setTaskHistoryIdx] = useState(-1);
  const [phase, setPhase] = useState<Phase>("input");
  const [log, setLog] = useState<string[]>([]);
  const [error, setError] = useState("");
  const [sessionId, setSessionId] = useState("");
  const [mode, setMode] = useState<PossessionMode>("conference");
  const [isManualMode, setIsManualMode] = useState(false);
  const [matchedSouls, setMatchedSouls] = useState<MatchedSoul[]>([]);
  const [review, setReview] = useState<ReviewResult | null>(null);
  const [showDetail, setShowDetail] = useState(true);
  const [sessionDone, setSessionDone] = useState(false);
  const [isCancelled, setIsCancelled] = useState(false);
  const [searchTopic, setSearchTopic] = useState(true);
  const [taskCards, setTaskCards] = useState<Record<string, string>>({});
  // Interrogation gate states
  const [igGateId, setIgGateId] = useState("");
  const [igQuestions, setIgQuestions] = useState<InterrogationQuestion[]>([]);
  const [igAnswers, setIgAnswers] = useState<string[]>([]);
  const [igReason, setIgReason] = useState("");
  const [igSubmitting, setIgSubmitting] = useState(false);
  const [progressLine, setProgressLine] = useState("");
  const [logFilter, setLogFilter] = useState<LogFilter>("全部");
  const [logsCollapsed, setLogsCollapsed] = useState(true);
  const [copiedLogs, setCopiedLogs] = useState(false);
  const [streamingContent, setStreamingContent] = useState<string>("");
  const [currentStreamSource, setCurrentStreamSource] = useState<string>("");
  const [isStreaming, setIsStreaming] = useState(false);
  const isStreamingRef = useRef(false);

  const [searchQuery, setSearchQuery] = useState("");
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchResults, setSearchResults] = useState<SearxngResultItem[]>([]);
  const [searchContext, setSearchContext] = useState("");
  const [showSearch, setShowSearch] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const abortRef = useRef<AbortController | null>(null);

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

  const skipInterrogationRef = useRef(false);
  const interrogationContextRef = useRef("");

  /** 审查官审讯：提交使用者对反问的逐条回答 → 获得裁决 */
  const onInterrogationSubmit = async () => {
    if (igSubmitting) return;
    setIgSubmitting(true);
    setError("");

    const answers = igQuestions
      .map((_q, i) => ({ question_index: i, answer: igAnswers[i]?.trim() ?? "" }))
      .filter((a) => a.answer.length > 0);

    if (answers.length === 0) {
      setError("请至少回答一个反问。");
      setIgSubmitting(false);
      return;
    }

    try {
      addLog(`📝 提交 ${answers.length}/${igQuestions.length} 条反问回答…`);
      const verdict: InterrogationVerdictResponse = await submitInterrogation(igGateId, answers);

      if (verdict.passed) {
        // 通过 → 继续合议流程
        addLog("✅ 审查官放行：" + verdict.reason);
        if (verdict.refined_task) {
          addLog("📝 审查官改写议题：" + verdict.refined_task);
          setTask(verdict.refined_task);
        }
        // 保存审查官 Q&A 供注入魂共享上下文
        const qaText = igQuestions
          .map((q, i) => `Q${i + 1}: ${q.text}\nA${i + 1}: ${igAnswers[i]?.trim() || "（未答）"}`)
          .join("\n\n");
        interrogationContextRef.current = qaText ? `审查官入场审讯了使用者的提问动机和行动承诺。以下为审讯记录：\n\n${qaText}` : "";
        setIgQuestions([]);
        setIgReason("");
        setPhase("classifying");
        skipInterrogationRef.current = true;
        onStart();
      } else {
        // 驳回 → 显示原因 + 更新反问列表
        setIgReason(verdict.reason);
        addLog(`❌ 审查官驳回：${verdict.reason}`);
        if (verdict.questions && verdict.questions.length > 0) {
          setIgQuestions(verdict.questions);
          setIgAnswers(new Array(verdict.questions.length).fill(""));
          addLog(`🗣️ 审查官追加 ${verdict.questions.length} 个反问`);
        }
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "审讯失败");
    } finally {
      setIgSubmitting(false);
    }
  };

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
    if (!canStart) return;
    if (phase !== "input" && !skipInterrogationRef.current) return;

    // 保存到历史
    const taskText = task.trim();
    setTaskHistory((prev) => {
      const next = [taskText, ...prev.filter((t) => t !== taskText)].slice(0, 50);
      localStorage.setItem("possess-task-history", JSON.stringify(next));
      return next;
    });
    setTaskHistoryIdx(-1);
    
    setIsCancelled(false);
    setLog([]);
    setError("");
    setMatchedSouls([]);
    setReview(null);
    setSessionDone(false);
    setStreamingContent("");
    setCurrentStreamSource("");
    setIsStreaming(false);
    isStreamingRef.current = false;

    // ── 审查官入场审讯 ──
    if (skipInterrogationRef.current) {
      skipInterrogationRef.current = false;
      // 已经通过了审查，直接跳过
    } else try {
      setPhase("interrogation");
      setProgressLine("审查官正在审问你的提问意图…");
      addLog("🔍 审查官正在分析你的提问『意图』…");
      const igResp = await startInterrogation(task);
      setIgGateId(igResp.gate_id);
      setIgQuestions(igResp.questions);
      setIgAnswers(new Array(igResp.questions.length).fill(""));
      setIgReason("");

      if (igResp.questions.length === 0) {
        // 审查官直接放行
        addLog("✅ 审查官判定：无需反问，直接入场");
        setPhase("classifying");
      } else {
        addLog(`🗣️ 审查官提出 ${igResp.questions.length} 个反问 —— 请逐一回应后继续`);
        return; // 暂停，等用户填写反问
      }
    } catch (e: unknown) {
      addLog(`❌ 审查官审讯失败: ${e instanceof Error ? e.message : String(e)}——请稍后重试`);
      setPhase("input");
      setError(`审查官审讯出错: ${e instanceof Error ? e.message : String(e)}`);
      return;
    }

    try {
      // Fast path: souls pre-selected via URL (e.g. from soul-recommendation card)
      // → skip analyzeTask and start session directly with these souls
      if (prefilledSouls.length > 0) {
        addLog(`🎯 已预选魂：${prefilledSouls.join("、")}`);
        setProgressLine(`正在启动附体会话（已预选 ${prefilledSouls.length} 个魂）…`);
        setPhase("starting");
        setMode("conference");

        const { session_id } = await startPossession({
          mode: "conference",
          task,
          souls: prefilledSouls,
          search_topic: searchTopic,
          interrogation_context: interrogationContextRef.current || undefined,
        });

        if (isCancelled) {
          setPhase("input");
          return;
        }

        setSessionId(session_id);
        setPhase("running");
        triggerSessionsUpdate();
        addLog("🎉 附体会话已启动");
        return;
      }

      addLog("开始分析任务...");
      setProgressLine("正在分析任务，入口分流中…");
      const reviewer = localStorage.getItem("aionui-banner-lord") || undefined;
      abortRef.current = new AbortController();

      const data = await analyzeTask(task, reviewer, abortRef.current.signal, (event) => {
        if (isCancelled) return;

        if (event.phase === "classifying") {
          setProgressLine("入口分流完成，正在算法匹配…");
        }
        if (event.phase === "matching") {
          setProgressLine("正在多因子匹配魂…");
        }
        if (event.phase === "matched" && event.souls && event.souls.length > 0) {
          setPhase("reviewing");
          setMatchedSouls(event.souls);
          if (!isManualMode) {
            setMode((event.mode || "conference") as PossessionMode);
          }
          addLog(`✅ 匹配完成: ${event.souls.length} 个魂`);
          addLog(`推荐模式: ${getModeLabel(event.mode || "conference")}`);
          setProgressLine(`已匹配 ${event.souls.length} 个魂：${event.souls.map((s) => s.name).join("、")}`);
        }
        if (event.phase === "reviewing") {
          addLog(`🔍 审查官 ${event.reviewer} 正在审查魂组合…`);
          setProgressLine(`审查官 ${event.reviewer} 正在审查魂组合…`);
        }
        if (event.phase === "review_done" && event.review) {
          setReview(event.review);
          if (event.task_cards && Object.keys(event.task_cards).length > 0) {
            setTaskCards(event.task_cards);
          }
          addLog(`🔍 审查完成 | 裁决: ${getVerdictLabel(event.review.verdict)}`);
          (event.review.checks || []).forEach((c: string) => addLog(`   - ${c}`));
          if (event.review.notes) addLog(`📝 备注: ${event.review.notes}`);
        }
        if (event.phase === "adjusting") {
          setPhase("adjusting");
          addLog("🔄 审查未通过 → 重新调整魂组合...");
          setProgressLine("审查未通过，正在调整魂组合…");
        }
        if (event.phase === "analysis_content") {
          if (!isStreamingRef.current) {
            isStreamingRef.current = true;
            setIsStreaming(true);
            setCurrentStreamSource(event.source || "");
            setStreamingContent("");
            addLog(`📝 ${event.source} 正在生成内容...`);
          }
          if (event.is_done) {
            isStreamingRef.current = false;
            setIsStreaming(false);
            addLog(`📝 ${event.source} 生成完成`);
            setStreamingContent("");
            setCurrentStreamSource("");
          } else if (event.content) {
            setStreamingContent((prev) => prev + event.content);
          }
        }
      });

      if (isCancelled) {
        setPhase("input");
        return;
      }

      if (data.entry_type === "practice_opening") {
        setPhase("starting");
        setProgressLine("检测到实践者在场，启动实践开口流程…");
        addLog("✨ 启动实践开口附体会话...");
        try {
          const { session_id } = await startPossession({
            mode: "practice_opening",
            task,
            souls: [],
            interrogation_context: interrogationContextRef.current || undefined,
          });
          setSessionId(session_id);
          setMode("practice_opening");
          setPhase("running");
          addLog("🎉 实践开口附体会话已启动");
          setProgressLine("实践开口进行中…");
          triggerSessionsUpdate();
        } catch (e: unknown) {
          const errorMsg = e instanceof Error ? e.message : String(e);
          setError(errorMsg);
          addLog(`❌ 错误: ${errorMsg}`);
          setPhase("input");
        }
        return;
      }

      const cards = data.task_cards || {};
      const souls = data.matched_souls || [];
      setTaskCards(cards);

      if (isCancelled) {
        setPhase("input");
        return;
      }

      const recommendedMode = data.recommended_mode || "conference";
      const finalMode: PossessionMode = isManualMode ? mode : (recommendedMode as PossessionMode);
      if (isManualMode) {
        addLog(`🎯 手动模式：使用选定的 ${getModeLabel(finalMode)}，跳过模式推荐`);
      }
      setMode(finalMode);
      setPhase("starting");
      addLog("🚀 启动附体会话...");
      setProgressLine("正在启动附体会话…");

      const { session_id } = await startPossession({
        mode: finalMode, task, souls: souls.map((s: any) => s.name),
        task_cards: Object.keys(cards).length > 0 ? cards : undefined,
        search_topic: searchTopic,
        interrogation_context: interrogationContextRef.current || undefined,
      });

      if (isCancelled) {
        setPhase("input");
        return;
      }

      setSessionId(session_id);
      setPhase("running");
      addLog("🎉 附体会话已启动");
      setProgressLine("附体会话已启动，魂正在思考…");
      triggerSessionsUpdate();
    } catch (e: unknown) {
      console.error("=== 发生错误:", e);
      abortRef.current?.abort();
      const isAbort = e instanceof Error && e.name === "AbortError";
      if (isAbort || isCancelled) {
        setPhase("input");
        return;
      }
      const errorMsg = e instanceof Error ? e.message : String(e || "分析失败");
      setError(errorMsg);
      addLog(`❌ 错误: ${errorMsg}`);
      setPhase("input");
    }
  };

  const onCancel = () => {
    abortRef.current?.abort();
    setIsCancelled(true);
    setPhase("input");
    setProgressLine("");
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

  const copyLogs = () => {
    const text = log.join("\n");
    navigator.clipboard.writeText(text).then(() => {
      setCopiedLogs(true);
      setTimeout(() => setCopiedLogs(false), 2000);
    }).catch(() => {});
  };

  const handleSessionDone = () => {
    setSessionDone(true);
  };

  const filteredLogs = log.filter((l) => {
    if (logFilter === "全部") return true;
    const typeMap: Record<string, string> = { "关键": "key", "魂匹配": "soul", "审查": "review" };
    return classifyLogType(l) === typeMap[logFilter];
  });

  const currentPhaseIndex = PHASES.findIndex((p) => p.key === phase);
  const totalPhases = PHASES.length;

  if (phase === "running" && sessionId) {
    return (
      <div className="max-w-5xl mx-auto space-y-4 animate-in fade-in duration-500" data-testid="possession-entry">
        <SessionContextHeader task={task} mode={mode} matchedSouls={matchedSouls} review={review} />

        <SessionRunner
          sessionId={sessionId}
          mode={mode}
          matchedSouls={matchedSouls}
          onDone={handleSessionDone}
          sessionDone={sessionDone}
        />
      </div>
    );
  }

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
              {prefilledSouls.length > 0 && (
                <div className="flex items-center gap-2 px-3 py-2 rounded-md bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800">
                  <Sparkles className="h-4 w-4 text-amber-500 shrink-0" />
                  <span className="text-xs text-amber-700 dark:text-amber-300">
                    <strong>已预选魂：</strong>{prefilledSouls.join("、")} · 跳过自动匹配，直接以此召唤合议
                  </span>
                </div>
              )}
              <AttachmentUpload onOcrResults={handleOcrResults} />
              <Textarea
                placeholder="描述你的问题或任务..."
                value={task}
                onChange={(e) => {
                  setTask(e.target.value);
                  setTaskHistoryIdx(-1);
                }}
                onKeyDown={(e) => {
                  if (e.nativeEvent.isComposing || e.keyCode === 229) return;
                  if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) { onStart(); return; }
                  if (e.key === "ArrowUp" && taskHistory.length > 0) {
                    const textarea = e.currentTarget as HTMLTextAreaElement;
                    if (textarea.selectionStart !== textarea.selectionEnd) return;
                    if (taskHistoryIdx === -1) {
                      const nextIdx = 0;
                      setTaskHistoryIdx(nextIdx);
                      setTask(taskHistory[nextIdx]);
                    } else if (taskHistoryIdx < taskHistory.length - 1) {
                      const nextIdx = taskHistoryIdx + 1;
                      setTaskHistoryIdx(nextIdx);
                      setTask(taskHistory[nextIdx]);
                    }
                    e.preventDefault();
                    return;
                  }
                  if (e.key === "ArrowDown" && taskHistoryIdx >= 0) {
                    if (taskHistoryIdx === 0) {
                      setTaskHistoryIdx(-1);
                      setTask("");
                    } else {
                      const nextIdx = taskHistoryIdx - 1;
                      setTaskHistoryIdx(nextIdx);
                      setTask(taskHistory[nextIdx]);
                    }
                    e.preventDefault();
                  }
                }}
                rows={5}
                data-testid="task-input"
                className="resize-none transition-all focus:ring-2 focus:ring-primary/20"
              />

              {searchContext && (
                <div className="flex items-center gap-2 text-xs text-muted-foreground bg-muted/30 rounded-lg px-3 py-1.5">
                  <Globe className="h-3.5 w-3.5 text-green-500" />
                  <span>已添加 SearXNG 搜索背景</span>
                  <button
                    type="button"
                    onClick={clearSearchContext}
                    className="ml-auto text-destructive hover:underline"
                    aria-label="清除搜索背景"
                  >
                    清除
                  </button>
                </div>
              )}

              {!showSearch && (
                <button
                  type="button"
                  onClick={() => setShowSearch(true)}
                  className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
                  aria-label="搜索背景资料"
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
                      type="button"
                      onClick={() => { setShowSearch(false); setSearchResults([]); }}
                      className="text-xs text-muted-foreground hover:text-foreground"
                      aria-label="收起搜索"
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
                        onKeyDown={(e) => {
                          if (e.nativeEvent.isComposing || e.keyCode === 229) return;
                          if (e.key === "Enter") handleSearch();
                        }}
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

              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Wand2 className="h-4 w-4 text-primary" />
                    <span className="text-sm font-medium">选择模式</span>
                  </div>
                  <button
                    type="button"
                    onClick={() => setIsManualMode(!isManualMode)}
                    className={cn(
                      "text-xs px-2 py-1 rounded-md transition-colors",
                      isManualMode 
                        ? "bg-primary/10 text-primary" 
                        : "text-muted-foreground hover:text-foreground"
                    )}
                  >
                    {isManualMode ? "手动选择" : "自动匹配"}
                  </button>
                </div>

                {isManualMode && (
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                    {MODES.map((m) => {
                      const Icon = iconMap[m.icon];
                      const isSelected = mode === m.key;
                      return (
                        <button
                          key={m.key}
                          type="button"
                          onClick={() => setMode(m.key)}
                          className={cn(
                            "group rounded-lg border p-3 text-left transition-all",
                            isSelected 
                              ? "border-primary bg-primary/5 shadow-sm" 
                              : "hover:border-primary/30 hover:bg-muted/30"
                          )}
                        >
                          <div className="flex items-center gap-2 mb-1">
                            <div className={cn(
                              "flex items-center justify-center w-8 h-8 rounded-md",
                              isSelected 
                                ? `${MODE_COLORS_BG[m.key]} text-white` 
                                : "bg-muted text-muted-foreground group-hover:bg-primary/10 group-hover:text-primary"
                            )}>
                              {Icon && <Icon className="h-4 w-4" />}
                            </div>
                            <span className={cn(
                              "text-sm font-medium",
                              isSelected ? MODE_COLORS_TEXT[m.key] : ""
                            )}>
                              {m.label}
                            </span>
                          </div>
                          <p className="text-[11px] text-muted-foreground line-clamp-2">
                            {m.description}
                          </p>
                        </button>
                      );
                    })}
                  </div>
                )}

                {!isManualMode && (
                  <div className="rounded-lg border border-dashed bg-muted/20 p-3 text-center">
                    <p className="text-xs text-muted-foreground">
                      系统将自动分析任务，匹配最合适的模式与思想家
                    </p>
                  </div>
                )}
              </div>

              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <label className="flex items-center gap-2 cursor-pointer select-none">
                    <input
                      type="checkbox"
                      checked={searchTopic}
                      onChange={(e) => setSearchTopic(e.target.checked)}
                      className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                      id="search-topic-checkbox"
                    />
                    <Globe className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
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
                    {isManualMode ? (
                      <>
                        <MessageCircle className="mr-2 h-5 w-5" />
                        进入 {MODES.find(m => m.key === mode)?.label}
                      </>
                    ) : (
                      <>
                        <Brain className="mr-2 h-5 w-5" />
                        我想问
                      </>
                    )}
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 审查官入场审讯 — 反问卡片 */}
      {phase === "interrogation" && igQuestions.length > 0 && (
        <div className="space-y-6">
          <div className="text-center space-y-2">
            <h2 className="text-2xl font-bold bg-gradient-to-r from-red-500 to-orange-500 bg-clip-text text-transparent">
              审查官有话说
            </h2>
            <p className="text-sm text-muted-foreground max-w-md mx-auto">
              在进入合议前，审查官要求你回应以下问题。
              <br/>无法跳过——你不能绕过ta的脸。
            </p>
          </div>

          {igReason && (
            <div className="rounded-lg border border-red-200 dark:border-red-800 bg-red-50/50 dark:bg-red-950/20 p-3">
              <p className="text-sm text-red-700 dark:text-red-300">{igReason}</p>
            </div>
          )}

          <div className="space-y-4">
            {igQuestions.map((q, i) => (
              <div key={i} className="rounded-lg border bg-background p-4 space-y-2">
                <div className="flex items-start gap-2">
                  <span className="text-xs font-mono text-muted-foreground mt-1">#{i + 1}</span>
                  <label className="text-sm font-semibold flex-1">{q.text}</label>
                  {q.required && (
                    <span className="text-[10px] text-red-500 font-medium">必答</span>
                  )}
                </div>
                <textarea
                  className="w-full min-h-[80px] text-sm rounded-md border bg-muted/30 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/30 resize-y"
                  placeholder="你的回应..."
                  value={igAnswers[i] ?? ""}
                  onChange={(e) => {
                    const next = [...igAnswers];
                    next[i] = e.target.value;
                    setIgAnswers(next);
                  }}
                />
              </div>
            ))}
          </div>

          <div className="flex justify-end gap-3">
            <Button
              onClick={onInterrogationSubmit}
              disabled={igSubmitting || igAnswers.every((a) => !a?.trim())}
            >
              {igSubmitting ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  审查中...
                </>
              ) : (
                <>
                  <CheckCircle2 className="mr-2 h-4 w-4" />
                  提交回答
                </>
              )}
            </Button>
          </div>
        </div>
      )}

      {phase !== "input" && phase !== "running" && (
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

                {review && (
                  <div className="mt-4 space-y-3">
                    <div className="flex items-center gap-2 text-sm">
                      <ShieldCheck className="h-4 w-4 text-purple-500" />
                      <span className="font-medium text-purple-600 dark:text-purple-400">
                        {review.reviewer} 的审查
                      </span>
                      <span className={cn(
                        "text-xs px-2 py-0.5 rounded-full",
                        review.verdict === "pass" ? "bg-emerald-100 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-400" :
                        review.verdict === "conditional" ? "bg-amber-100 text-amber-700 dark:bg-amber-950/30 dark:text-amber-400" :
                        "bg-red-100 text-red-700 dark:bg-red-950/30 dark:text-red-400"
                      )}>
                        {getVerdictLabel(review.verdict)}
                      </span>
                    </div>
                    {review.checks && review.checks.length > 0 && (
                      <div className="text-xs text-muted-foreground space-y-1 pl-6">
                        {review.checks.map((c: string, i: number) => (
                          <div key={i}>• {c}</div>
                        ))}
                      </div>
                    )}
                    {review.notes && (
                      <div className="text-xs text-muted-foreground pl-6">
                        📝 {review.notes}
                      </div>
                    )}
                  </div>
                )}

                {Object.keys(taskCards).length > 0 && (
                  <div className="mt-4 space-y-2">
                    <div className="flex items-center gap-2 text-sm">
                      <Brain className="h-4 w-4 text-emerald-500" />
                      <span className="font-medium text-emerald-600 dark:text-emerald-400">
                        专属子问题
                      </span>
                    </div>
                    <div className="space-y-2 pl-6">
                      {Object.entries(taskCards).map(([soul, question]) => (
                        <div key={soul} className="text-xs bg-muted/30 rounded-md p-2">
                          <span className="font-medium text-foreground">{soul}：</span>
                          <span className="text-muted-foreground">{question}</span>
                        </div>
                      ))}
                    </div>
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
                      type="button"
                      onClick={() => setLogsCollapsed(!logsCollapsed)}
                      className="flex items-center gap-2 hover:text-foreground transition-colors"
                      aria-expanded={!logsCollapsed}
                      aria-controls="execution-logs"
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
                          type="button"
                          key={f}
                          onClick={() => setLogFilter(f)}
                          className={cn(
                            "text-[10px] px-2 py-0.5 rounded transition-colors",
                            logFilter === f ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
                          )}
                          aria-pressed={logFilter === f}
                        >
                          {f}
                        </button>
                      ))}
                    </div>
                    <button
                      type="button"
                      onClick={copyLogs}
                      className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
                      title="复制日志"
                      aria-label="复制日志"
                    >
                      {copiedLogs ? <Check className="h-3.5 w-3.5 text-emerald-500" /> : <Copy className="h-3.5 w-3.5" />}
                    </button>
                  </div>
                </div>
                {!logsCollapsed && (
                  <div className="bg-background rounded-lg p-3 max-h-64 overflow-y-auto font-mono text-xs space-y-1">
                    {progressLine && (
                      <div className="text-primary font-medium flex items-center gap-2 pb-2 mb-2 border-b border-primary/10">
                        <Loader2 className="h-3 w-3 animate-spin shrink-0" />
                        <span>{progressLine}</span>
                      </div>
                    )}
                    {isStreaming && streamingContent && (
                      <div className="mt-2 mb-2 p-2 rounded-md border border-purple-200 dark:border-purple-800 bg-purple-50/50 dark:bg-purple-950/20">
                        <div className="flex items-center gap-2 mb-1">
                          <Loader2 className="h-3 w-3 animate-spin text-purple-500" />
                          <span className="text-purple-600 dark:text-purple-400 font-medium text-[10px]">
                            {currentStreamSource} 正在生成...
                          </span>
                        </div>
                        <div className="text-muted-foreground whitespace-pre-wrap leading-relaxed text-[11px]">
                          {streamingContent}
                          <span className="animate-pulse">▌</span>
                        </div>
                      </div>
                    )}
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
                        正在启动附体会话…
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
