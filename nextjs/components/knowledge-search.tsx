"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { useRouter } from "next/navigation";
import { searchKnowledge, rebuildFts, type KnowledgeResult } from "@/lib/api";
import { Search, RefreshCw, Loader2 } from "lucide-react";
import { ModeBarChart } from "@/components/mode-bar-chart";

export function KnowledgeSearch() {
  const router = useRouter();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<KnowledgeResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [rebuilding, setRebuilding] = useState(false);
  const [rebuildCount, setRebuildCount] = useState<number | null>(null);
  const [searched, setSearched] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const debounceRef = useRef<NodeJS.Timeout | null>(null);
  const initialLoadedRef = useRef(false);

  const doSearch = useCallback(async (q: string) => {
    setLoading(true);
    setError(null);
    try {
      const data = await searchKnowledge(q, 30);
      setResults(data);
      setSearched(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : "搜索失败");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    doSearch("");
    initialLoadedRef.current = true;
  }, [doSearch]);

  useEffect(() => {
    if (!initialLoadedRef.current) return;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => doSearch(query), 300);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [query, doSearch]);

  useEffect(() => {
    if (query.trim()) setRebuildCount(null);
  }, [query]);

  useEffect(() => {
    if (results.length > 0) setRebuildCount(null);
  }, [results]);

  const handleRebuild = async () => {
    setRebuilding(true);
    setRebuildCount(null);
    try {
      const res = await rebuildFts();
      setError(null);
      setRebuildCount(res.indexed);
      doSearch(query);
    } catch (e) {
      setError(e instanceof Error ? e.message : "重建失败");
    } finally {
      setRebuilding(false);
    }
  };

  const modeCounts = searched
    ? results.reduce<Record<string, number>>((acc, r) => {
        acc[r.mode] = (acc[r.mode] || 0) + 1;
        return acc;
      }, {})
    : {};

  return (
    <div className="flex flex-col gap-4">
      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索魂输出、综合报告、会话记录..."
            className="w-full rounded-lg border bg-background pl-10 pr-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary/30"
          />
        </div>
        <button
          onClick={handleRebuild}
          disabled={rebuilding}
          className="flex items-center gap-2 rounded-lg border px-4 py-2.5 text-sm hover:bg-muted transition-colors disabled:opacity-50"
        >
          {rebuilding ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="h-4 w-4" />
          )}
          重建索引
        </button>
      </div>

      {error && (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {rebuildCount !== null && (
        <div className="rounded-lg border border-emerald-500/50 bg-emerald-500/10 p-3 text-sm text-emerald-700 dark:text-emerald-300">
          索引已重建（共 {rebuildCount} 条），当前展示全部记录
        </div>
      )}

      {loading && (
        <div className="flex items-center gap-2 text-sm text-muted-foreground py-4">
          <Loader2 className="h-4 w-4 animate-spin" />
          搜索中...
        </div>
      )}

      {searched && !loading && results.length === 0 && (
        <div className="text-sm text-muted-foreground py-8 text-center">
          {query.trim()
            ? "未找到匹配结果"
            : "暂无内容，请先进行附体对话以生成记录"}
        </div>
      )}

      {results.length > 0 && (
        <>
          <div className="flex items-center gap-4 text-xs text-muted-foreground">
            <span>{results.length} 条结果</span>
            {Object.keys(modeCounts).length > 0 && (
              <div className="flex gap-2">
                {Object.entries(modeCounts).map(([mode, count]) => (
                  <span key={mode} className="rounded-full bg-muted px-2 py-0.5">
                    {mode}: {count}
                  </span>
                ))}
              </div>
            )}
          </div>

          <ModeBarChart data={modeCounts} />

          <div className="flex flex-col gap-3">
            {results.map((r, i) => (
              <div
                key={i}
                role="button"
                tabIndex={0}
                onClick={() => router.push(`/sessions/${r.session_id}`)}
                onKeyDown={(e) => { if (e.key === 'Enter') router.push(`/sessions/${r.session_id}`); }}
                className="rounded-lg border bg-background p-4 transition-colors hover:border-primary/20 cursor-pointer"
              >
                <div className="flex items-center gap-2 mb-2">
                  {r.soul_name && (
                    <span className="text-sm font-semibold">{r.soul_name}</span>
                  )}
                  <span className="text-xs rounded-full bg-muted px-2 py-0.5 text-muted-foreground">
                    {r.mode}
                  </span>
                  <span className="text-xs text-muted-foreground">
                    {r.created_at?.slice(0, 10)}
                  </span>
                </div>
                {r.task_summary && (
                  <p className="text-xs text-muted-foreground mb-1">
                    任务: {r.task_summary}
                  </p>
                )}
                <p
                  className="text-sm leading-relaxed"
                  dangerouslySetInnerHTML={{ __html: r.content_snippet }}
                />
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
