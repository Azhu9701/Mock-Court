"use client";

import { useEffect, useState } from "react";
import { useTheme } from "next-themes";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  Sun, Moon, Monitor,
  Database, Search, HardDrive, Trash, Server, Terminal,
  RefreshCw, Download, Loader2, AlertTriangle, CheckCircle2,
} from "lucide-react";
import { API_BASE, rebuildFts } from "@/lib/api";

const themeOptions = [
  { value: "system", label: "跟随系统", icon: Monitor },
  { value: "light", label: "亮色模式", icon: Sun },
  { value: "dark", label: "暗色模式", icon: Moon },
] as const;

interface SettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function SettingsDialog({ open, onOpenChange }: SettingsDialogProps) {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  const [saved, setSaved] = useState<Record<string, boolean>>({});

  const [healthStatus, setHealthStatus] = useState<boolean | null>(null);
  const [healthChecking, setHealthChecking] = useState(false);
  const [rebuilding, setRebuilding] = useState(false);
  const [rebuildMsg, setRebuildMsg] = useState<{ ok: boolean; text: string } | null>(null);
  const [exporting, setExporting] = useState(false);
  const [clearingSessions, setClearingSessions] = useState(false);
  const [clearingSouls, setClearingSouls] = useState(false);
  const [clearingArchive, setClearingArchive] = useState(false);
  const [clearMsg, setClearMsg] = useState<{ ok: boolean; text: string } | null>(null);

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (!open) return;

    checkHealth();
    setRebuildMsg(null);
    setClearMsg(null);
  }, [open]);

  const checkHealth = async () => {
    setHealthChecking(true);
    try {
      const res = await fetch(`${API_BASE}/health`, { signal: AbortSignal.timeout(5000) });
      setHealthStatus(res.ok);
    } catch {
      setHealthStatus(false);
    } finally {
      setHealthChecking(false);
    }
  };

  const handleRebuild = async () => {
    setRebuilding(true);
    setRebuildMsg(null);
    try {
      const res = await rebuildFts();
      setRebuildMsg({ ok: true, text: `索引重建完成，共索引 ${res.indexed} 条记录` });
    } catch (e) {
      setRebuildMsg({ ok: false, text: e instanceof Error ? e.message : "重建失败" });
    } finally {
      setRebuilding(false);
    }
  };

  const handleExportArchive = async () => {
    setExporting(true);
    try {
      const res = await fetch(`${API_BASE}/archive/export`, { method: "POST" });
      if (!res.ok) throw new Error(res.statusText);
      const data = await res.json();
      alert(`归档导出已启动\n任务 ID：${data.task_id}\n请等待后台处理完成后在数据目录查看`);
    } catch (e) {
      alert(`导出失败：${e instanceof Error ? e.message : "未知错误"}`);
    } finally {
      setExporting(false);
    }
  };

  const handleClearAll = async (type: "sessions" | "souls" | "archive") => {
    const labels = { sessions: "所有会话", souls: "所有角色", archive: "所有归档" };
    const setters = { sessions: setClearingSessions, souls: setClearingSouls, archive: setClearingArchive };
    if (!confirm(`确定要删除${labels[type]}吗？此操作不可恢复。`)) return;
    setters[type](true);
    setClearMsg(null);
    try {
      const endpoint = type === "sessions" ? "sessions" : type === "souls" ? "souls" : "archive";
      const allRes = await fetch(`${API_BASE}/${endpoint}`);
      if (!allRes.ok) throw new Error(allRes.statusText);
      const items: { id?: string; name?: string }[] = await allRes.json();
      for (const item of items) {
        const id = item.id || item.name;
        if (id) {
          await fetch(`${API_BASE}/${endpoint}/${encodeURIComponent(id)}`, { method: "DELETE" });
        }
      }
      setClearMsg({ ok: true, text: `${labels[type]}已清除（${items.length} 条）` });
    } catch (e) {
      setClearMsg({ ok: false, text: e instanceof Error ? e.message : "清除失败" });
    } finally {
      setters[type](false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg flex flex-col max-h-[90vh]" data-testid="settings-dialog">
        <DialogHeader className="shrink-0">
          <DialogTitle>设置</DialogTitle>
        </DialogHeader>

        <div className="overflow-y-auto flex-1 min-h-0 space-y-6 py-2">
          {/* ── 外观 ── */}
          <div>
            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
              {mounted && theme === "dark" ? (
                <Moon className="h-4 w-4" />
              ) : (
                <Sun className="h-4 w-4" />
              )}
              外观
            </h3>
            <p className="text-xs text-muted-foreground mb-3">
              选择你喜欢的配色方案
            </p>
            <div className="flex gap-2">
              {themeOptions.map((opt) => {
                const Icon = opt.icon;
                const active = mounted && theme === opt.value;
                return (
                  <Button
                    key={opt.value}
                    variant={active ? "default" : "outline"}
                    size="sm"
                    onClick={() => setTheme(opt.value)}
                    className="flex-1"
                  >
                    <Icon className="h-4 w-4" />
                    {opt.label}
                  </Button>
                );
              })}
            </div>
          </div>

          <hr className="border-border" />

          {/* ── 数据管理 ── */}
          <div>
            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
              <Database className="h-4 w-4" />
              数据管理
            </h3>
            <p className="text-xs text-muted-foreground mb-3">
              管理系统数据存储，包括索引重建、数据导出与清理
            </p>

            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-medium">重建知识索引</p>
                  <p className="text-[10px] text-muted-foreground">
                    重新索引所有角色输出和会话记录，用于全文搜索
                  </p>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleRebuild}
                  disabled={rebuilding}
                >
                  {rebuilding ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <RefreshCw className="h-3.5 w-3.5" />
                  )}
                  重建
                </Button>
              </div>
              {rebuildMsg && (
                <div className={`text-xs rounded-md p-2 ${
                  rebuildMsg.ok
                    ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
                    : "bg-destructive/10 text-destructive"
                }`}>
                  {rebuildMsg.text}
                </div>
              )}

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-medium">导出归档</p>
                  <p className="text-[10px] text-muted-foreground">
                    将所有会话数据导出为 .md 归档文件
                  </p>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleExportArchive}
                  disabled={exporting}
                >
                  {exporting ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <Download className="h-3.5 w-3.5" />
                  )}
                  导出
                </Button>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-medium flex items-center gap-1">
                    <AlertTriangle className="h-3 w-3 text-destructive" />
                    清除所有会话
                  </p>
                  <p className="text-[10px] text-muted-foreground">
                    删除所有庭审会话记录，不可恢复
                  </p>
                </div>
                <Button
                  size="sm"
                  variant="destructive"
                  onClick={() => handleClearAll("sessions")}
                  disabled={clearingSessions}
                >
                  {clearingSessions ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <Trash className="h-3.5 w-3.5" />
                  )}
                  清除
                </Button>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-medium flex items-center gap-1">
                    <AlertTriangle className="h-3 w-3 text-destructive" />
                    清除所有角色
                  </p>
                  <p className="text-[10px] text-muted-foreground">
                    删除所有角色及其 prompt 和有效性数据
                  </p>
                </div>
                <Button
                  size="sm"
                  variant="destructive"
                  onClick={() => handleClearAll("souls")}
                  disabled={clearingSouls}
                >
                  {clearingSouls ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <Trash className="h-3.5 w-3.5" />
                  )}
                  清除
                </Button>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-medium flex items-center gap-1">
                    <AlertTriangle className="h-3 w-3 text-destructive" />
                    清除所有归档
                  </p>
                  <p className="text-[10px] text-muted-foreground">
                    删除所有综合报告、知识卡片等归档
                  </p>
                </div>
                <Button
                  size="sm"
                  variant="destructive"
                  onClick={() => handleClearAll("archive")}
                  disabled={clearingArchive}
                >
                  {clearingArchive ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <Trash className="h-3.5 w-3.5" />
                  )}
                  清除
                </Button>
              </div>

              {clearMsg && (
                <div className={`text-xs rounded-md p-2 ${
                  clearMsg.ok
                    ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
                    : "bg-destructive/10 text-destructive"
                }`}>
                  {clearMsg.text}
                </div>
              )}
            </div>
          </div>

          <hr className="border-border" />

          {/* ── 系统状态 ── */}
          <div>
            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
              <Server className="h-4 w-4" />
              系统状态
            </h3>

            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs">
                <span className="text-muted-foreground flex items-center gap-1.5">
                  <Terminal className="h-3.5 w-3.5" />
                  后端服务 ({API_BASE.replace("/api/v1", "").replace("http://", "").replace("https://", "")})
                </span>
                <div className="flex items-center gap-2">
                  {healthChecking ? (
                    <span className="flex items-center gap-1 text-muted-foreground">
                      <Loader2 className="h-3 w-3 animate-spin" />
                      检测中
                    </span>
                  ) : healthStatus === true ? (
                    <span className="flex items-center gap-1 text-emerald-600 dark:text-emerald-400">
                      <CheckCircle2 className="h-3.5 w-3.5" />
                      正常
                    </span>
                  ) : healthStatus === false ? (
                    <span className="flex items-center gap-1 text-destructive">
                      <AlertTriangle className="h-3.5 w-3.5" />
                      离线
                    </span>
                  ) : null}
                  <Button size="xs" variant="ghost" onClick={checkHealth} disabled={healthChecking}>
                    <RefreshCw className="h-3 w-3" />
                  </Button>
                </div>
              </div>

              <div className="flex items-center justify-between text-xs">
                <span className="text-muted-foreground flex items-center gap-1.5">
                  <HardDrive className="h-3.5 w-3.5" />
                  数据存储
                </span>
                <span className="text-muted-foreground">
                  ~/data/
                </span>
              </div>

              <div className="flex items-center justify-between text-xs">
                <span className="text-muted-foreground flex items-center gap-1.5">
                  <Search className="h-3.5 w-3.5" />
                  WebSocket (庭审)
                </span>
                <span className="text-muted-foreground">
                  {API_BASE.replace("http://", "ws://").replace("https://", "wss://").replace("/api/v1", "")}
                </span>
              </div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
