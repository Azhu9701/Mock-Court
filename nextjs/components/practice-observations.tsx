"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import type { PracticeObservation } from "@/lib/api";

interface PracticeObservationsProps {
  observations: PracticeObservation[];
}

const labelMap: Record<string, string> = {
  Confirmed: "确认",
  Modified: "修改",
  Overturned: "推翻",
};

export function PracticeObservations({ observations }: PracticeObservationsProps) {
  const [showAll, setShowAll] = useState(false);
  const visible = showAll ? observations : observations.slice(0, 5);

  if (observations.length === 0) {
    return (
      <div data-testid="practice-observations">
        <h3 className="text-sm font-semibold mb-2">实践记录</h3>
        <p className="text-sm text-muted-foreground">暂无实践记录</p>
      </div>
    );
  }

  return (
    <div data-testid="practice-observations">
      <h3 className="text-sm font-semibold mb-2">
        实践记录 ({observations.length})
      </h3>
      <div className="space-y-2">
        {visible.map((obs, i) => (
          <div key={i} className="rounded-md border p-3 text-sm">
            <div className="flex items-center justify-between mb-1">
              <span className="text-xs text-muted-foreground">{obs.date}</span>
              <span className="text-xs font-medium">
                {labelMap[obs.revision_type] || obs.revision_type}
              </span>
            </div>
            <p className="text-foreground">{obs.observation}</p>
          </div>
        ))}
      </div>
      {observations.length > 5 && (
        <Button
          variant="ghost"
          size="sm"
          className="mt-2"
          onClick={() => setShowAll(!showAll)}
          data-testid="observations-toggle"
        >
          {showAll ? "收起" : `展开全部 (${observations.length} 条)`}
        </Button>
      )}
    </div>
  );
}
