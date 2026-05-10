"use client";

import { useEffect, useState } from "react";
import { useTheme } from "next-themes";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  Eye, EyeOff, Trash2, Settings2, Sun, Moon, Monitor,
  Database, Search, HardDrive, Trash, Server, Terminal,
  RefreshCw, Download, Loader2, AlertTriangle, CheckCircle2,
} from "lucide-react";
import { DEEPSEEK_MODELS_NO_DEFAULT, REASONING_OPTIONS } from "@/config/models";
import { rebuildFts } from "@/lib/api";

type Provider = "claude" | "openai" | "deepseek";

interface ApiKeyEntry {
  provider: Provider;
  label: string;
  key: string;
  envVar: string;
}

const API_KEYS: ApiKeyEntry[] = [
  { provider: "claude", label: "Claude (Anthropic)", key: "", envVar: "ANTHROPIC_API_KEY" },
  { provider: "openai", label: "OpenAI", key: "", envVar: "OPENAI_API_KEY" },
  { provider: "deepseek", label: "DeepSeek", key: "", envVar: "DEEPSEEK_API_KEY" },
];

const REASONING_NO_DEFAULT = REASONING_OPTIONS.filter(r => r.value !== "");

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

  const [keys, setKeys] = useState<Record<string, string>>({});
  const [visible, setVisible] = useState<Record<string, boolean>>({});
  const [saved, setSaved] = useState<Record<string, boolean>>({});
  const [defaultModel, setDefaultModel] = useState<string>("deepseek-v4-pro");
  const [defaultReasoning, setDefaultReasoning] = useState<string>("think");

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

    const stored: Record<string, string> = {};
    for (const k of API_KEYS) {
      const val = localStorage.getItem(`apikey_${k.provider}`) || "";
      stored[k.provider] = val;
    }
    setKeys(stored);
    setDefaultModel(localStorage.getItem("default_model") || "deepseek-v4-pro");
    setDefaultReasoning(localStorage.getItem("default_reasoning") || "think");

    checkHealth();
    setRebuildMsg(null);
    setClearMsg(null);
  }, [open]);

  const checkHealth = async () => {
    setHealthChecking(true);
    try {
      const res = await fetch("http://127.0.0.1:3096/api/v1/health", { signal: AbortSignal.timeout(5000) });
      setHealthStatus(res.ok);
    } catch {
      setHealthStatus(false);
    } finally {
      setHealthChecking(false);
    }
  };

  const saveKey = async (provider: string) => {
    const val = keys[provider] || "";
    localStorage.setItem(`apikey_${provider}`, val);
    try {
      const map: Record<string, string> = { claude: "anthropic", openai: "openai", deepseek: "deepseek" };
      await fetch("http://127.0.0.1:3096/api/v1/apikey/set", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ provider: map[provider] || provider, key: val }),
      });
    } catch { /* backend may not be running */ }
    setSaved((prev) => ({ ...prev, [provider]: true }));
    setTimeout(() => setSaved((prev) => ({ ...prev, [provider]: false })), 2000);
  };

  const clearKey = (provider: string) => {
    setKeys((prev) => ({ ...prev, [provider]: "" }));
    localStorage.removeItem(`apikey_${provider}`);
  };

  const saveModelSettings = async () => {
    localStorage.setItem("default_model", defaultModel);
    localStorage.setItem("default_reasoning", defaultReasoning);
    try {
      await fetch("http://127.0.0.1:3096/api/v1/config/model", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ model: defaultModel, reasoning: defaultReasoning }),
      });
    } catch { /* backend may not be running */ }
    setSaved((prev) => ({ ...prev, model: true }));
    setTimeout(() => setSaved((prev) => ({ ...prev, model: false })), 2000);
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
      const res = await fetch("http://127.0.0.1:3096/api/v1/archive/export", { method: "POST" });
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
    const labels = { sessions: "所有会话", souls: "所有魂", archive: "所有归档" };
    const setters = { sessions: setClearingSessions, souls: setClearingSouls, archive: setClearingArchive };
    if (!confirm(`确定要删除${labels[type]}吗？此操作不可恢复。`)) return;
    setters[type](true);
    setClearMsg(null);
    try {
      const endpoint = type === "sessions" ? "sessions" : type === "souls" ? "souls" : "archive";
      const allRes = await fetch(`http://127.0.0.1:3096/api/v1/${endpoint}`);
      if (!allRes.ok) throw new Error(allRes.statusText);
      const items: { id?: string; name?: string }[] = await allRes.json();
      for (const item of items) {
        const id = item.id || item.name;
        if (id) {
          await fetch(`http://127.0.0.1:3096/api/v1/${endpoint}/${encodeURIComponent(id)}`, { method: "DELETE" });
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
          {/* ── API Keys ── */}
          <div>
            <h3 className="text-sm font-semibold mb-3">API Key</h3>
            <p className="text-xs text-muted-foreground mb-3">
              Key 存储在浏览器 localStorage 中，仅供本地使用
            </p>
            {API_KEYS.map((entry) => (
              <div key={entry.provider} className="mb-3">
                <label className="text-xs font-medium">{entry.label}</label>
                <div className="flex gap-1 mt-1">
                  <div className="relative flex-1">
                    <Input
                      type={visible[entry.provider] ? "text" : "password"}
                      placeholder={`输入 ${entry.label} API Key...`}
                      value={keys[entry.provider] || ""}
                      onChange={(e) =>
                        setKeys((prev) => ({
                          ...prev,
                          [entry.provider]: e.target.value,
                        }))
                      }
                      className="pr-8 text-sm"
                      data-testid={`apikey-${entry.provider}`}
                    />
                    <button
                      type="button"
                      className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                      onClick={() =>
                        setVisible((prev) => ({
                          ...prev,
                          [entry.provider]: !prev[entry.provider],
                        }))
                      }
                    >
                      {visible[entry.provider] ? (
                        <EyeOff className="h-3.5 w-3.5" />
                      ) : (
                        <Eye className="h-3.5 w-3.5" />
                      )}
                    </button>
                  </div>
                  <Button
                    size="sm"
                    variant={saved[entry.provider] ? "default" : "outline"}
                    onClick={() => saveKey(entry.provider)}
                    data-testid={`save-key-${entry.provider}`}
                  >
                    {saved[entry.provider] ? "已保存" : "保存"}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => clearKey(entry.provider)}
                    data-testid={`clear-key-${entry.provider}`}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
                <p className="text-[10px] text-muted-foreground mt-0.5">
                  或设置环境变量 {entry.envVar}
                </p>
              </div>
            ))}
          </div>

          <hr className="border-border" />

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

          {/* ── 默认模型配置 ── */}
          <div>
            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
              <Settings2 className="h-4 w-4" />
              默认模型配置
            </h3>
            <p className="text-xs text-muted-foreground mb-3">
              设置召唤魂时默认使用的模型。单个魂可以在魂详情页单独配置
            </p>

            <div className="space-y-4">
              <div>
                <label className="text-xs font-medium block mb-1.5">默认模型</label>
                <Select value={defaultModel} onValueChange={(value) => setDefaultModel(value ?? "")}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="选择默认模型" />
                  </SelectTrigger>
                  <SelectContent>
                    {DEEPSEEK_MODELS_NO_DEFAULT.map((model) => (
                      <SelectItem key={model.value} value={model.value}>
                        {model.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div>
                <label className="text-xs font-medium block mb-1.5">默认推理强度</label>
                <Select value={defaultReasoning} onValueChange={(value) => setDefaultReasoning(value ?? "")}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="选择推理强度" />
                  </SelectTrigger>
                  <SelectContent>
                    {REASONING_NO_DEFAULT.map((opt) => (
                      <SelectItem key={opt.value} value={opt.value}>
                        {opt.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <Button
                size="sm"
                variant={saved.model ? "default" : "outline"}
                onClick={saveModelSettings}
                className="w-full"
              >
                {saved.model ? "已保存默认模型配置" : "保存默认模型配置"}
              </Button>
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
                    重新索引所有魂输出和会话记录，用于全文搜索
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
                    删除所有附体会话记录，不可恢复
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
                    清除所有魂
                  </p>
                  <p className="text-[10px] text-muted-foreground">
                    删除所有魂及其 prompt 和有效性数据
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
                  后端服务 (127.0.0.1:3096)
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
                  <Database className="h-3.5 w-3.5" />
                  API Key 状态
                </span>
                <div className="flex gap-2">
                  {API_KEYS.map((entry) => {
                    const hasKey = !!(keys[entry.provider]);
                    return (
                      <span
                        key={entry.provider}
                        className={`rounded-full px-2 py-0.5 text-[10px] ${
                          hasKey
                            ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
                            : "bg-muted text-muted-foreground"
                        }`}
                      >
                        {entry.provider}
                      </span>
                    );
                  })}
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
                  WebSocket (附体)
                </span>
                <span className="text-muted-foreground">
                  ws://127.0.0.1:3096
                </span>
              </div>
            </div>
          </div>

          <hr className="border-border" />

          {/* ── 幡主 ── */}
          <div>
            <h3 className="text-sm font-semibold mb-2">幡主</h3>
            <p className="text-xs text-muted-foreground">
              幡主也是被召唤的魂。附体前 spawn 幡主子 agent 独立审查魂匹配结果。
            </p>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
