"use client";

import { useState } from "react";
import { SoulResponseCard } from "@/components/soul-response-card";

const INITIAL_COUNT = 20;

interface SoulResponsesGridProps {
  responses: Record<string, string>;
}

export function SoulResponsesGrid({ responses }: SoulResponsesGridProps) {
  const entries = Object.entries(responses);
  const [showAll, setShowAll] = useState(false);

  const displayed = showAll ? entries : entries.slice(0, INITIAL_COUNT);
  const hasMore = entries.length > INITIAL_COUNT;

  return (
    <div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {displayed.map(([name, content]) => (
          <SoulResponseCard
            key={name}
            name={name}
            content={content}
          />
        ))}
      </div>
      {hasMore && !showAll && (
        <div className="mt-4 text-center">
          <button
            onClick={() => setShowAll(true)}
            className="inline-flex items-center gap-2 rounded-lg border bg-background px-4 py-2 text-sm font-medium text-primary hover:bg-primary/5 transition-colors"
          >
            显示全部 {entries.length} 个回应
          </button>
        </div>
      )}
    </div>
  );
}
