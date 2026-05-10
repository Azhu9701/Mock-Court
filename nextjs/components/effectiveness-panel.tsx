import type { EffectivenessTrend } from "@/lib/api";

interface EffectivenessPanelProps {
  trend: EffectivenessTrend;
}

function Bar({ value, total, color }: { value: number; total: number; color: string }) {
  const pct = total > 0 ? (value / total) * 100 : 0;
  return (
    <div className="flex items-center gap-2">
      <span className="w-16 text-xs text-muted-foreground">
        {color === "bg-green-500" ? "有效" : color === "bg-yellow-500" ? "部分" : "无效"}
      </span>
      <div className="flex-1 h-4 bg-muted rounded-full overflow-hidden">
        <div
          className={`h-full ${color} transition-all`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="w-10 text-xs text-right text-muted-foreground">{value}</span>
    </div>
  );
}

export function EffectivenessPanel({ trend }: EffectivenessPanelProps) {
  const total = trend.total_calls;
  return (
    <div data-testid="effectiveness-panel" className="space-y-2">
      <h3 className="text-sm font-semibold">
        调用有效性
        <span className="ml-2 text-xs font-normal text-muted-foreground">
          (共 {total} 次)
        </span>
      </h3>
      <Bar value={trend.effective} total={total} color="bg-green-500" />
      <Bar value={trend.partial} total={total} color="bg-yellow-500" />
      <Bar value={trend.invalid} total={total} color="bg-red-500" />
      <p className="text-xs text-muted-foreground text-right">
        有效率: {(trend.effective_rate * 100).toFixed(1)}%
      </p>
    </div>
  );
}
