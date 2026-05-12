"use client";

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { modeLabel, MODE_COLORS_HEX } from "@/config/possession-modes";
import type { PossessionMode } from "@/config/possession-modes";

interface ModeBarChartProps {
  data: Record<string, number>;
}

export function ModeBarChart({ data }: ModeBarChartProps) {
  const chartData = Object.entries(data)
    .filter(([key]) => modeLabel(key) !== key)
    .map(([key, count]) => ({
      name: modeLabel(key),
      count,
      fill: (MODE_COLORS_HEX as Record<string, string>)[key] || "#888",
    }));

  if (chartData.length === 0) return null;

  return (
    <div data-testid="mode-bar-chart" className="h-64">
      <h3 className="text-sm font-semibold mb-3">模式分布</h3>
      <ResponsiveContainer width="100%" height={200}>
        <BarChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
          <XAxis
            dataKey="name"
            tick={{ fontSize: 12, fill: "var(--muted-foreground)" }}
          />
          <YAxis
            tick={{ fontSize: 12, fill: "var(--muted-foreground)" }}
            allowDecimals={false}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "var(--background)",
              border: "1px solid var(--border)",
              borderRadius: "8px",
            }}
          />
          <Bar dataKey="count" fill="var(--primary)" radius={[4, 4, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
