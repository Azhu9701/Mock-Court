"use client";

import { useState, useCallback, useRef } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { analyzeTask, startPossession, fetchSouls, searchWeb, type SearxngResultItem } from "@/lib/api";
import { triggerSessionsUpdate } from "@/components/sidebar-sessions";
import { AttachmentUpload } from "@/components/attachment-upload";
import { PracticeOpeningDialog } from "@/components/practice-opening-dialog";
import { storePendingSession } from "@/lib/pending-session";
import { Brain, Loader2, Globe, Search, AlertCircle } from "lucide-react";

type Phase = "input" | "loading" | "practice_opening";

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

  const [searchQuery, setSearchQuery] = useState("");
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchResults, setSearchResults] = useState<SearxngResultItem[]>([]);
  const [searchContext, setSearchContext] = useState("");
  const [showSearch, setShowSearch] = useState(false);

  const canStart = task.trim().length > 0;

  const handleOcrResults = useCallback((texts: string[]) => {
    const block = texts.join("\n\n");
    setTask((prev) => {
      const trimmed = prev.trim();
      return trimmed ? `${block}\n\n---\n\n${trimmed}` : block;
    });
  }, []);

  const handleSearch = useCallback(async () => {
    if (!searchQuery.trim()) return;
    setSearchLoading(true); setSearchResults([]);
    try { const data = await searchWeb({ q: searchQuery, language: "zh" }); setSearchResults(data.results); }
    catch (e) { console.error("搜索失败:", e); }
    finally { setSearchLoading(false); }
  }, [searchQuery]);

  const applyResultContext = useCallback((result: SearxngResultItem) => {
    const ctx = `## ${result.title}\n${result.content || ""}\n来源: ${result.url}`;
    setSearchContext(ctx);
    setTask((prev) => {
      const header = "> 以下是通过 SearXNG 搜索获取的背景信息\n\n";
      const trimmed = prev.replace(/^> 以下是通过 SearXNG 搜索获取的背景信息\n\n[\s\S]*?\n\n---\n\n/gm, "").trim();
      return `${header}${ctx}\n\n---\n\n${trimmed}`;
    });
    setShowSearch(false); setSearchResults([]);
  }, []);

  const clearSearchContext = useCallback(() => {
    setSearchContext("");
    setTask((prev) => prev.replace(/^> 以下是通过 SearXNG 搜索获取的背景信息\n\n[\s\S]*?\n\n---\n\n/gm, "").trim());
  }, []);

  const onStart = async () => {
    if (!canStart || phase !== "input") return;

    setPhase("loading");
    setError("");
    setProgressLine("正在分析任务，入口分流中…");

    let souls: { name: string; field: string; ismism_code: string; rationale: string }[] = [];
    let matchedMode = "conference";
    let cards: Record<string, string> = {};
    let reviewData: any = {};
    const completedPhases: string[] = [];

    // Try analyzeTask with 12s timeout
    const controller = new AbortController();
    const analyzeTimeout = setTimeout(() => controller.abort(), 12000);
    const reviewer = localStorage.getItem("aionui-banner-lord") || undefined;
    try {
      const data = await analyzeTask(task, reviewer, controller.signal);
      clearTimeout(analyzeTimeout);

      if (data.entry_type === "practice_opening") {
        setPhase("practice_opening");
        return;
      }

      souls = data.matched_souls || [];
      matchedMode = data.recommended_mode || "conference";
      cards = data.task_cards || {};
      reviewData = data.review || {};
      completedPhases.push("classifying", "matching", "reviewing");
      if (reviewData.verdict === "reject") completedPhases.push("adjusting");
    } catch (e: any) {
      clearTimeout(analyzeTimeout);
      if (e.name === "AbortError") {
        setProgressLine("分析超时，使用默认魂组合…");
      }
      // Fallback: use a few souls from registry
      try {
        const allSouls = await fetchSouls();
        souls = allSouls.slice(0, 3).map(s => ({
          name: s.name, field: s.field, ismism_code: s.ismism_code, rationale: s.self_declare || ""
        }));
      } catch {}
    }

    completedPhases.push("starting");

    const uniquePhases = Array.from(new Set(completedPhases));
    setProgressLine(`已匹配 ${souls.length} 个魂，正在启动讨论…`);

    try {
      const { session_id } = await startPossession({
        mode: matchedMode, task,
        souls: souls.map((s) => s.name),
        task_cards: Object.keys(cards).length > 0 ? cards : undefined,
        search_topic: searchTopic,
        judgment: judgment || undefined, worry: worry || undefined, unknown: unknown || undefined,
      });

      storePendingSession({
        sessionId: session_id, task, mode: matchedMode,
        matchedSouls: souls,
        review: reviewData.reviewer ? reviewData : null,
        phases: uniquePhases,
      });

      triggerSessionsUpdate();
      router.push(`/sessions/${session_id}`);
    } catch (e: any) {
      setError(e.message || "启动失败");
      setPhase("input");
    }
  };

  // ── Loading ──
  if (phase === "loading") {
    return (
      <div className="max-w-2xl mx-auto space-y-6" data-testid="possession-entry">
        <div className="rounded-xl border bg-background p-6 shadow-sm space-y-6">
          <div className="flex items-center gap-3">
            <Loader2 className="h-5 w-5 animate-spin text-primary" />
            <div>
              <h2 className="text-lg font-semibold">{task}</h2>
              <p className="text-sm text-muted-foreground">{progressLine}</p>
            </div>
          </div>
          <div className="h-2 bg-muted rounded-full overflow-hidden">
            <div className="h-full bg-gradient-to-r from-primary to-emerald-500 animate-pulse rounded-full w-2/3" />
          </div>
        </div>
        {error && (
          <div className="rounded-xl border-2 border-red-200 bg-red-50 p-6 text-center">
            <AlertCircle className="h-10 w-10 text-red-500 mx-auto mb-3" />
            <h3 className="font-semibold text-lg text-red-700 mb-2">启动失败</h3>
            <p className="text-red-600">{error}</p>
            <Button variant="outline" className="mt-4" onClick={() => setPhase("input")}>重新开始</Button>
          </div>
        )}
      </div>
    );
  }

  if (phase === "practice_opening") {
    return (
      <PracticeOpeningDialog
        open={true}
        onStart={async (j, w, u) => {
          setJudgment(j); setWorry(w); setUnknown(u);
          setProgressLine("正在启动…");
          try {
            const { session_id } = await startPossession({
              mode: "practice_opening", task, souls: [],
              judgment: j || undefined, worry: w || undefined, unknown: u || undefined,
            });
            storePendingSession({ sessionId: session_id, task, mode: "practice_opening", matchedSouls: [], review: null, phases: ["starting"] });
            triggerSessionsUpdate();
            router.push(`/sessions/${session_id}`);
          } catch (e: any) { setError(e.message || "启动失败"); setPhase("input"); }
        }}
        onCancel={() => setPhase("input")}
      />
    );
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6" data-testid="possession-entry">
      <div className="text-center space-y-2">
        <h2 className="text-2xl font-bold bg-gradient-to-r from-primary to-purple-600 bg-clip-text text-transparent">开始讨论</h2>
        <p className="text-sm text-muted-foreground">输入你的问题，系统将自动完成全流程</p>
      </div>
      <div className="rounded-xl border bg-background p-6 shadow-sm">
        <div className="space-y-4">
          <AttachmentUpload onOcrResults={handleOcrResults} />
          <Textarea
            placeholder="描述你的问题或任务..." value={task} onChange={(e) => setTask(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) onStart(); }}
            rows={5} data-testid="task-input" className="resize-none transition-all focus:ring-2 focus:ring-primary/20"
          />
          {searchContext && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground bg-muted/30 rounded-lg px-3 py-1.5">
              <Globe className="h-3.5 w-3.5 text-green-500" /><span>已添加 SearXNG 搜索背景</span>
              <button onClick={clearSearchContext} className="ml-auto text-destructive hover:underline">清除</button>
            </div>
          )}
          {!showSearch && (
            <button onClick={() => setShowSearch(true)} className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground">
              <Search className="h-4 w-4" />搜索背景资料（通过 SearXNG）
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
                  {searchLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}搜索
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
              <input type="checkbox" checked={searchTopic} onChange={(e) => setSearchTopic(e.target.checked)} className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary" />
              <Globe className="h-4 w-4 text-muted-foreground" /><span className="text-sm text-muted-foreground">启动时搜索议题背景</span>
            </label>
            <Button onClick={onStart} disabled={!canStart} className="w-full sm:w-auto" size="lg" data-testid="start-possession-btn">
              <Brain className="mr-2 h-5 w-5" />我想问
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
