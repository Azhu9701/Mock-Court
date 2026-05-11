"use client";

import { useEffect, useState, useRef } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Trash2, Brain, Users, ChevronRight, Download } from "lucide-react";
import { ConfirmButton } from "@/components/ui/confirm-button";
import { deleteSession, exportSessionMarkdown, fetchSessions } from "@/lib/api";
import { triggerSessionsUpdate, SESSIONS_UPDATED_EVENT } from "@/components/sidebar-sessions";
import type { SessionSummary } from "@/lib/api";
import { modeLabel, MODE_COLORS_BG, MODE_COLORS_TEXT } from "@/config/possession-modes";
import type { PossessionMode } from "@/config/possession-modes";

interface SessionTimelineProps {
  sessions: SessionSummary[];
}

const modeBgColors: Record<string, string> = {
  single: "bg-blue-50 dark:bg-blue-950/30 border-blue-200 dark:border-blue-800",
  conference: "bg-purple-50 dark:bg-purple-950/30 border-purple-200 dark:border-purple-800",
  debate: "bg-orange-50 dark:bg-orange-950/30 border-orange-200 dark:border-orange-800",
  relay: "bg-green-50 dark:bg-green-950/30 border-green-200 dark:border-green-800",
  learn: "bg-teal-50 dark:bg-teal-950/30 border-teal-200 dark:border-teal-800",
  practice_opening: "bg-red-50 dark:bg-red-950/30 border-red-200 dark:border-red-800",
};

function groupByDate(sessions: SessionSummary[]): Map<string, SessionSummary[]> {
  const groups = new Map<string, SessionSummary[]>();
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today.getTime() - 86400000);

  for (const s of sessions) {
    const d = new Date(s.created_at);
    const dateOnly = new Date(d.getFullYear(), d.getMonth(), d.getDate());
    let label: string;
    if (dateOnly.getTime() === today.getTime()) {
      label = "今天";
    } else if (dateOnly.getTime() === yesterday.getTime()) {
      label = "昨天";
    } else {
      label = d.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
    }
    const arr = groups.get(label) || [];
    arr.push(s);
    groups.set(label, arr);
  }
  return groups;
}

function SessionRow({ s, onDelete }: { s: SessionSummary; onDelete: (id: string) => void }) {
  const router = useRouter();

  const handleExport = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    try { await exportSessionMarkdown(s.id, s.title); }
    catch { }
  };

  const handleDelete = async () => {
    try {
      await deleteSession(s.id);
      onDelete(s.id);
      triggerSessionsUpdate();
    } catch {}
  };

  return (
    <Link
      href={`/sessions/${s.id}`}
      className={`flex items-center gap-3 p-3 rounded-lg border transition-all hover:shadow-md group ${modeBgColors[s.mode] || "bg-background border-muted"}`}
      data-testid={`session-item-${s.id}`}
    >
      {/* 模式标识 */}
      <div className="flex items-center gap-2 shrink-0">
        <div className={`w-2 h-2 rounded-full ${MODE_COLORS_BG[s.mode as PossessionMode] || "bg-gray-400"}`} />
        <span className={`text-xs font-medium px-2 py-0.5 rounded-full bg-white/50 dark:bg-gray-800 ${MODE_COLORS_TEXT[s.mode as PossessionMode] ? '' : 'text-muted-foreground'}`}>
          {modeLabel(s.mode)}
        </span>
      </div>

      {/* 标题和内容预览 */}
      <div className="flex-1 min-w-0">
        <h4 className="text-sm font-medium truncate group-hover:text-primary transition-colors">
          {s.title}
        </h4>
        {s.message_count > 0 && (
          <p className="text-xs text-muted-foreground mt-0.5 truncate">
            {s.message_count} 条消息
          </p>
        )}
      </div>

      {/* 参与魂数量 */}
      <div className="flex items-center gap-1 text-xs text-muted-foreground shrink-0">
        <Users className="h-3 w-3" />
        <span>{s.soul_count || 1}</span>
      </div>

      {/* 时间 */}
      <div className="text-xs text-muted-foreground shrink-0 text-right w-16">
        {new Date(s.created_at).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}
      </div>

      {/* 导出按钮 */}
      <button
        onClick={handleExport}
        className="opacity-0 group-hover:opacity-100 transition-opacity shrink-0 p-1 hover:bg-muted rounded absolute right-10"
        title="导出 Markdown"
      >
        <Download className="h-4 w-4" />
      </button>

      {/* 箭头 */}
      <ChevronRight className="h-4 w-4 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity shrink-0" />

      {/* 删除按钮 */}
      <ConfirmButton
        icon={<Trash2 className="h-4 w-4" />}
        confirmText="确认"
        title="删除会话"
        className="opacity-0 group-hover:opacity-100 transition-opacity shrink-0 absolute right-3"
        onConfirm={handleDelete}
      />
    </Link>
  );
}

export function SessionTimeline({ sessions: initialSessions }: SessionTimelineProps) {
  const [sessions, setSessions] = useState<SessionSummary[]>(initialSessions);
  const skipEventRef = useRef(false);

  useEffect(() => {
    const handle = () => {
      if (skipEventRef.current) {
        skipEventRef.current = false;
        return;
      }
      fetchSessions(200).then(setSessions).catch(() => {});
    };
    window.addEventListener(SESSIONS_UPDATED_EVENT, handle);
    return () => window.removeEventListener(SESSIONS_UPDATED_EVENT, handle);
  }, []);

  const handleDelete = (id: string) => {
    skipEventRef.current = true;
    setSessions((prev) => prev.filter((s) => s.id !== id));
  };
  if (sessions.length === 0) {
    return (
      <div data-testid="session-timeline" className="flex flex-col items-center justify-center py-12">
        <Brain className="h-12 w-12 text-muted-foreground mb-4" />
        <h3 className="text-sm font-semibold text-muted-foreground">暂无会话记录</h3>
        <p className="text-xs text-muted-foreground mt-1">开始你的第一次附体之旅</p>
        <Link href="/possess" className="mt-4 text-xs text-primary hover:underline">
          前往附体
        </Link>
      </div>
    );
  }

  const groups = groupByDate(sessions);

  return (
    <div data-testid="session-timeline" className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Brain className="h-4 w-4 text-primary" />
          <h3 className="text-sm font-semibold">会话历史</h3>
        </div>
        <span className="text-xs text-muted-foreground">{sessions.length} 个会话</span>
      </div>

      <div className="space-y-4">
        {Array.from(groups.entries()).map(([label, items]) => (
          <div key={label}>
            <h4 className="text-xs font-semibold text-muted-foreground mb-2 px-1">
              {label}
            </h4>
            <div className="space-y-2">
              {items.map((s) => (
                <div key={s.id} className="relative">
                  <SessionRow s={s} onDelete={handleDelete} />
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
