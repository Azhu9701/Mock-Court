"use client";

import { useState, useCallback } from "react";
import {
  searchWeb,
  type SearxngResultItem,
  type SearxngSearchResponse,
} from "@/lib/api";
import { Search, Loader2, Globe, ExternalLink, ChevronLeft, ChevronRight } from "lucide-react";

const CATEGORY_OPTIONS = [
  { value: "", label: "全部" },
  { value: "general", label: "通用" },
  { value: "news", label: "新闻" },
  { value: "science", label: "科学" },
  { value: "files", label: "文件" },
  { value: "images", label: "图片" },
  { value: "videos", label: "视频" },
  { value: "social media", label: "社交媒体" },
];

export function SearxngSearch() {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("");
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<SearxngSearchResponse | null>(null);

  const doSearch = useCallback(
    async (q: string, p: number, cat: string) => {
      if (!q.trim()) return;
      setLoading(true);
      setError(null);
      try {
        const resp = await searchWeb({
          q,
          pageno: p,
          language: "zh",
          categories: cat || undefined,
        });
        setData(resp);
      } catch (e) {
        setError(e instanceof Error ? e.message : "搜索失败");
        setData(null);
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setPage(1);
    doSearch(query, 1, category);
  };

  const handlePageChange = (newPage: number) => {
    setPage(newPage);
    doSearch(query, newPage, category);
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  return (
    <div className="flex flex-col gap-6">
      <form onSubmit={handleSearch} className="flex flex-col gap-3">
        <div className="flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="通过 SearXNG 搜索互联网..."
              className="w-full rounded-lg border bg-background pl-10 pr-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary/30"
            />
          </div>
          <select
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            className="rounded-lg border bg-background px-3 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary/30"
          >
            {CATEGORY_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
          <button
            type="submit"
            disabled={loading || !query.trim()}
            className="flex items-center gap-2 rounded-lg bg-primary px-6 py-2.5 text-sm font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50"
          >
            {loading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Search className="h-4 w-4" />
            )}
            搜索
          </button>
        </div>
      </form>

      {error && (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {data && (
        <div className="flex flex-col gap-4">
          <div className="flex items-center justify-between text-sm text-muted-foreground">
            <span>
              找到 {data.number_of_results} 条结果
              {data.unresponsive_engines.length > 0 && (
                <span className="ml-2 text-xs text-muted-foreground/70">
                  (部分引擎无响应)
                </span>
              )}
            </span>
          </div>

          {data.results.length === 0 ? (
            <div className="text-sm text-muted-foreground py-12 text-center">
              未找到匹配结果
            </div>
          ) : (
            <>
              <div className="flex flex-col gap-3">
                {data.results.map((r, i) => (
                  <ResultCard key={i} result={r} />
                ))}
              </div>

              {data.number_of_results > 20 && (
                <div className="flex items-center justify-center gap-2 pt-2">
                  <button
                    onClick={() => handlePageChange(page - 1)}
                    disabled={page <= 1}
                    className="flex items-center gap-1 rounded-lg border px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-40"
                  >
                    <ChevronLeft className="h-4 w-4" />
                    上一页
                  </button>
                  <span className="text-sm text-muted-foreground px-2">
                    第 {page} 页
                  </span>
                  <button
                    onClick={() => handlePageChange(page + 1)}
                    className="flex items-center gap-1 rounded-lg border px-3 py-1.5 text-sm hover:bg-muted"
                  >
                    下一页
                    <ChevronRight className="h-4 w-4" />
                  </button>
                </div>
              )}
            </>
          )}

          {data.suggestions.length > 0 && (
            <div className="flex flex-col gap-2 pt-2">
              <span className="text-xs text-muted-foreground">相关搜索建议：</span>
              <div className="flex flex-wrap gap-2">
                {data.suggestions.map((s, i) => (
                  <button
                    key={i}
                    onClick={() => {
                      setQuery(s);
                      setPage(1);
                      doSearch(s, 1, category);
                    }}
                    className="rounded-full border px-3 py-1 text-xs hover:bg-muted transition-colors"
                  >
                    {s}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {!data && !loading && (
        <div className="flex flex-col items-center gap-3 py-16 text-muted-foreground">
          <Globe className="h-12 w-12 opacity-20" />
          <span className="text-sm">输入关键词，通过 SearXNG 搜索互联网</span>
        </div>
      )}
    </div>
  );
}

function ResultCard({ result }: { result: SearxngResultItem }) {
  const domain = (() => {
    try {
      return new URL(result.url).hostname;
    } catch {
      return "";
    }
  })();

  return (
    <a
      href={result.url}
      target="_blank"
      rel="noopener noreferrer"
      className="group flex flex-col gap-1.5 rounded-lg border bg-background p-4 transition-colors hover:border-primary/20 hover:bg-muted/30"
    >
      <div className="flex items-start justify-between gap-2">
        <h3 className="text-sm font-semibold text-primary group-hover:underline line-clamp-1">
          {result.title}
        </h3>
        <ExternalLink className="h-3.5 w-3.5 text-muted-foreground shrink-0 mt-0.5 opacity-0 group-hover:opacity-100 transition-opacity" />
      </div>
      {domain && (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Globe className="h-3 w-3" />
          <span>{domain}</span>
          <span className="text-muted-foreground/50">·</span>
          <span>{result.engines.join(", ")}</span>
        </div>
      )}
      {result.content && (
        <p className="text-xs text-muted-foreground line-clamp-3 mt-0.5 leading-relaxed">
          {result.content}
        </p>
      )}
    </a>
  );
}
