import {
  fetchSummonStats,
  fetchModeDistribution,
  fetchUnsummonedAlerts,
  fetchLowEffectiveness,
  fetchSessions,
} from "@/lib/api";
import { StatCard } from "@/components/stat-card";
import { ModeBarChart, SoulEffectivenessTable } from "@/components/dashboard-charts";
import { AlertPanel } from "@/components/alert-panel";
import { SessionTimeline } from "@/components/session-timeline";

export const dynamic = "force-dynamic";

export default async function DashboardPage() {
  const [stats, modeDist, unsummoned, lowEff, sessions] = await Promise.all([
    fetchSummonStats(),
    fetchModeDistribution(),
    fetchUnsummonedAlerts(),
    fetchLowEffectiveness(),
    fetchSessions(10),
  ]);

  const participationRate =
    stats.total_souls_available > 0
      ? ((stats.unique_souls_called / stats.total_souls_available) * 100).toFixed(0) + "%"
      : "0%";

  const totalEffective = stats.by_soul.reduce((acc, s) => acc + s.effective_count, 0);
  const totalAll = stats.by_soul.reduce((acc, s) => acc + s.call_count, 0);
  const effectiveRate = totalAll > 0 ? ((totalEffective / totalAll) * 100).toFixed(0) + "%" : "0%";

  const alertCount = unsummoned.length + lowEff.length;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">仪表盘</h1>
        <p className="text-sm text-muted-foreground mt-1">万民幡运行概览</p>
      </div>
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
          title="有效率"
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
