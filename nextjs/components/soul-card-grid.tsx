import type { SoulListEntry } from "@/lib/api";
import { SoulCard } from "@/components/soul-card";

interface SoulCardGridProps {
  souls: SoulListEntry[];
}

export function SoulCardGrid({ souls }: SoulCardGridProps) {
  if (souls.length === 0) {
    return (
      <div
        className="flex flex-col items-center justify-center py-20 text-muted-foreground"
        data-testid="soul-empty-state"
      >
        <p className="text-lg">未找到匹配的角色</p>
        <p className="text-sm mt-1">尝试调整筛选条件</p>
      </div>
    );
  }

  return (
    <div
      className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
      data-testid="soul-card-grid"
    >
      {souls.map((soul) => (
        <SoulCard key={soul.name} soul={soul} />
      ))}
    </div>
  );
}
