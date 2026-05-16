"use client";

import { useState } from "react";
import { Sparkles, Loader2, CheckCircle, UserPlus } from "lucide-react";
import { autoCreateSoul } from "@/lib/api";
import type { SoulRecommendation } from "@/hooks/use-websocket";

interface SoulRecommendationCardProps {
  recommendations: SoulRecommendation[];
}

function RecommendationItem({
  rec,
  onCreate,
  loading,
  done,
}: {
  rec: SoulRecommendation;
  onCreate: () => void;
  loading: boolean;
  done: boolean;
}) {
  return (
    <div className="flex items-start gap-3 p-3 rounded-lg border bg-background/50 hover:bg-muted/30 transition-colors">
      <div className="shrink-0 mt-0.5">
        <Sparkles className="h-4 w-4 text-amber-500" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium text-sm">{rec.name}</span>
        </div>
        <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
          {rec.rationale}
        </p>
      </div>
      <button
        onClick={onCreate}
        disabled={loading || done}
        className="shrink-0 flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {done ? (
          <>
            <CheckCircle className="h-3 w-3" />
            已入幡
          </>
        ) : loading ? (
          <>
            <Loader2 className="h-3 w-3 animate-spin" />
            炼化中…
          </>
        ) : (
          <>
            <UserPlus className="h-3 w-3" />
            收魂炼化
          </>
        )}
      </button>
    </div>
  );
}

export function SoulRecommendationCard({
  recommendations,
}: SoulRecommendationCardProps) {
  const [loadingMap, setLoadingMap] = useState<Record<string, boolean>>({});
  const [doneMap, setDoneMap] = useState<Record<string, boolean>>({});
  const [errorMap, setErrorMap] = useState<Record<string, string>>({});

  const handleCreate = async (name: string) => {
    setLoadingMap((prev) => ({ ...prev, [name]: true }));
    setErrorMap((prev) => ({ ...prev, [name]: "" }));
    try {
      await autoCreateSoul(name);
      setDoneMap((prev) => ({ ...prev, [name]: true }));
    } catch (e: any) {
      setErrorMap((prev) => ({
        ...prev,
        [name]: e.message || "收魂炼化失败",
      }));
    } finally {
      setLoadingMap((prev) => ({ ...prev, [name]: false }));
    }
  };

  return (
    <div className="mt-4 p-4 rounded-lg border border-amber-200 dark:border-amber-800 bg-amber-50/30 dark:bg-amber-950/20">
      <div className="flex items-center gap-2 mb-3">
        <Sparkles className="h-4 w-4 text-amber-500" />
        <h3 className="text-sm font-semibold text-amber-700 dark:text-amber-300">
          综合官推荐补充魂
        </h3>
        <span className="text-xs text-muted-foreground">
          {recommendations.length} 个
        </span>
      </div>
      <div className="space-y-2">
        {recommendations.map((rec) => (
          <div key={rec.name}>
            <RecommendationItem
              rec={rec}
              onCreate={() => handleCreate(rec.name)}
              loading={loadingMap[rec.name] || false}
              done={doneMap[rec.name] || false}
            />
            {errorMap[rec.name] && (
              <p className="text-xs text-red-500 mt-1 ml-10">
                {errorMap[rec.name]}
              </p>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
