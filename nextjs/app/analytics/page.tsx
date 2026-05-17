"use client";

import { useEffect, useState } from "react";
import {
  fetchSummonStats,
  fetchModeDistribution,
  fetchUnsummonedAlerts,
  fetchLowEffectiveness,
  fetchSessions,
  fetchPleasureStats,
  type SummonStatsResponse,
  type SessionSummary,
  type SoulAlert,
  type BoundaryReview,
  type PleasureStats,
} from "@/lib/api";
import { StatCard } from "@/components/stat-card";
import { ModeBarChart, SoulEffectivenessTable } from "@/components/dashboard-charts";
import { AlertPanel } from "@/components/alert-panel";
import { SessionTimeline } from "@/components/session-timeline";
import { Skeleton } from "@/components/ui/skeleton";

function pleasureLabel(pi: number): string {
  if (pi >= 70) return "蛇皮厚重 — 思辨已成消费习惯";
  if (pi >= 40) return "蛇皮中等 — 行动与消费参半";
  if (pi >= 15) return "蛇皮较薄 — 多数会话落地实践";
  return "接近蜕皮 — 思辨服务于行动";
}

function pleasureColor(pi: number): string {
  if (pi >= 70) return "text-red-500";
  if (pi >= 40) return "text-yellow-500";
  if (pi >= 15) return "text-yellow-400";
  return "text-green-500";
}

export default function DashboardPage() {
  const [stats, setStats] = useState<SummonStatsResponse | null>(null);
  const [modeDist, setModeDist] = useState<Record<string, number>>({});
  const [unsummoned, setUnsummoned] = useState<SoulAlert[]>([]);
  const [lowEff, setLowEff] = useState<BoundaryReview[]>([]);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [pleasure, setPleasure] = useState<PleasureStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      fetchSummonStats(),
      fetchModeDistribution(),
      fetchUnsummonedAlerts(),
      fetchLowEffectiveness(),
      fetchSessions(10),
      fetchPleasureStats(),
    ]).then(([s, m, u, l, ss, ps]) => {
      setStats(s);
      setModeDist(m);
      setUnsummoned(u);
      setLowEff(l);
      setSessions(ss);
      setPleasure(ps);
      setLoading(false);
    });
  }, []);

  if (loading || !stats || !pleasure) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold">蛇皮统计</h1>
          <p className="text-sm text-muted-foreground mt-1">你的思辨消费记录</p>
        </div>
        <div className="grid grid-cols-2 lg:grid-cols-5 gap-4">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton key={i} className="h-24 rounded-xl" />
          ))}
        </div>
      </div>
    );
  }

  const participationRate =
    stats.total_souls_available > 0
      ? ((stats.unique_souls_called / stats.total_souls_available) * 100).toFixed(0) + "%"
      : "0%";

  const totalEffective = stats.by_soul.reduce((acc, s) => acc + s.effective_count, 0);
  const totalAll = stats.by_soul.reduce((acc, s) => acc + s.call_count, 0);
  const effectiveRate = totalAll > 0 ? ((totalEffective / totalAll) * 100).toFixed(0) + "%" : "0%";

  const alertCount = unsummoned.length + lowEff.length;

  const totalTokens = stats.total_tokens || 0;
  const tokenDisplay = totalTokens > 1_000_000
    ? (totalTokens / 1_000_000).toFixed(1) + "M"
    : totalTokens > 1_000
    ? (totalTokens / 1_000).toFixed(1) + "K"
    : totalTokens.toString();

  const wastedTokenDisplay = pleasure.wasted_tokens > 1_000_000
    ? (pleasure.wasted_tokens / 1_000_000).toFixed(1) + "M"
    : pleasure.wasted_tokens > 1_000
    ? (pleasure.wasted_tokens / 1_000).toFixed(1) + "K"
    : pleasure.wasted_tokens.toString();

  // 按最贵模型输出价格 Opus $75/MTok ≈ ¥540/MTok
  const wastedCost = (pleasure.wasted_tokens / 1_000_000) * 540;
  const wastedCostDisplay = wastedCost >= 1
    ? `≈ ¥${wastedCost.toFixed(0).replace(/\B(?=(\d{3})+(?!\d))/g, ",")}`
    : "";

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">蛇皮统计</h1>
        <p className="text-sm text-muted-foreground mt-1">你的思辨消费记录</p>
      </div>

      {/* 蛇皮指数 — 核心指标 */}
      <div className="rounded-xl border bg-gradient-to-br from-red-50/40 to-yellow-50/20 dark:from-red-950/15 dark:to-yellow-950/5 p-5 space-y-4">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-sm font-semibold">蛇皮指数</h3>
          <span className="text-xs text-muted-foreground">
            基于 {pleasure.total_reviewed} 次会话 · 未走闭环 = 消费型
          </span>
        </div>
        <div className="flex items-baseline gap-2">
          <span className={`text-5xl font-bold tabular-nums ${pleasureColor(pleasure.pleasure_index)}`}>
            {pleasure.pleasure_index.toFixed(0)}
          </span>
          <span className="text-sm text-muted-foreground">/ 100</span>
        </div>
        <p className={`text-sm ${pleasureColor(pleasure.pleasure_index)}`}>
          {pleasureLabel(pleasure.pleasure_index)}
        </p>
        <div className="w-full bg-muted rounded-full h-2 overflow-hidden">
          <div
            className="h-full rounded-full transition-all"
            style={{
              width: `${Math.min(pleasure.pleasure_index, 100)}%`,
              background: pleasure.pleasure_index >= 70
                ? "linear-gradient(90deg, #f97316, #ef4444)"
                : pleasure.pleasure_index >= 40
                ? "linear-gradient(90deg, #eab308, #f97316)"
                : "linear-gradient(90deg, #22c55e, #eab308)",
            }}
          />
        </div>
      </div>

      {/* 蛇皮分解卡片 */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="消费型会话"
          value={pleasure.invalid_sessions}
          subtitle="未走闭环或写不出行动"
          icon="alert-triangle"
        />
        <StatCard
          title="意向型会话"
          value={pleasure.partial_sessions}
          subtitle="承诺模糊"
          icon="bar-chart"
        />
        <StatCard
          title="实践型会话"
          value={pleasure.effective_sessions}
          subtitle="有具体行动"
          icon="check-circle"
        />
        <StatCard
          title="浪费Token"
          value={wastedTokenDisplay}
          subtitle={`占已审查会话 ${(pleasure.waste_ratio * 100).toFixed(0)}%${wastedCostDisplay ? ` · 按 Opus 输出价 ¥540/MTok ${wastedCostDisplay}` : ""}`}
          icon="zap"
        />
      </div>

      {/* 原有统计 */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="总召唤次数"
          value={stats.total_calls}
          subtitle="全部历史"
          icon="bar-chart"
        />
        <StatCard
          title="魂参与率"
          value={participationRate}
          subtitle={`${stats.unique_souls_called}/${stats.total_souls_available} 魂`}
          icon="users"
        />
        <StatCard
          title="魂有效率"
          value={effectiveRate}
          subtitle={`${totalEffective} 有效 / ${totalAll} 次`}
          icon="check-circle"
        />
        <StatCard
          title="活跃告警"
          value={alertCount}
          subtitle={alertCount > 0 ? "需要关注" : "一切正常"}
          icon="alert-triangle"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <ModeBarChart data={modeDist} />
        <SoulEffectivenessTable stats={stats.by_soul} />
      </div>
      <AlertPanel unsummoned={unsummoned} lowEffectiveness={lowEff} />
      <SessionTimeline sessions={sessions} />
    </div>
  );
}
