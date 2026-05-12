import type { SoulCallStats } from "@/lib/api";

interface SoulEffectivenessTableProps {
  stats: SoulCallStats[];
}

function EffectiveRateBar({ effective, total }: { effective: number; total: number }) {
  const pct = total > 0 ? (effective / total) * 100 : 0;
  return (
    <div className="flex items-center gap-2">
      <div className="flex-1 h-2 bg-muted rounded-full overflow-hidden">
        <div
          className="h-full bg-green-500 rounded-full transition-all"
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-xs text-muted-foreground w-10 text-right">
        {pct.toFixed(0)}%
      </span>
    </div>
  );
}

export function SoulEffectivenessTable({ stats }: SoulEffectivenessTableProps) {
  if (stats.length === 0) {
    return (
      <div data-testid="effectiveness-table">
        <h3 className="text-sm font-semibold mb-3">魂有效性</h3>
        <p className="text-sm text-muted-foreground">暂无调用数据</p>
      </div>
    );
  }

  return (
    <div data-testid="effectiveness-table">
      <h3 className="text-sm font-semibold mb-3">魂有效性</h3>
      <div className="flex items-center gap-3 px-2 py-1 text-xs text-muted-foreground border-b mb-1">
        <span className="w-24 font-medium">魂名</span>
        <span className="w-12 text-right">次数</span>
        <span className="flex-1">有效率</span>
        <span className="w-16 text-right">Tokens</span>
      </div>
      <div className="space-y-1 max-h-80 overflow-y-auto">
        {stats
          .sort((a, b) => {
            const ar = a.call_count > 0 ? a.effective_count / a.call_count : 0;
            const br = b.call_count > 0 ? b.effective_count / b.call_count : 0;
            return br - ar;
          })
          .map((s) => (
            <div
              key={s.soul_name}
              className="flex items-center gap-3 rounded-md px-2 py-1.5 text-sm"
            >
              <span className="w-24 truncate font-medium">{s.soul_name}</span>
              <span className="w-12 text-xs text-muted-foreground text-right">
                {s.call_count}次
              </span>
              <div className="flex-1">
                <EffectiveRateBar
                  effective={s.effective_count}
                  total={s.call_count}
                />
              </div>
              <span className="w-16 text-xs text-muted-foreground text-right tabular-nums">
                {(s.total_tokens ?? 0) > 1000
                  ? ((s.total_tokens ?? 0) / 1000).toFixed(1) + "K"
                  : (s.total_tokens ?? 0).toLocaleString()}
              </span>
            </div>
          ))}
      </div>
    </div>
  );
}
