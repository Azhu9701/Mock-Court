"use client";

import { useEffect, useState, useCallback, useRef } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Eye,
  EyeOff,
  Loader2,
  RefreshCw,
  Check,
  X,
  Wifi,
  WifiOff,
  Server,
} from "lucide-react";
import { DEEPSEEK_MODELS_NO_DEFAULT, REASONING_OPTIONS } from "@/config/models";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "/api/v1";

interface ProviderStatus {
  id: string;
  name: string;
  model: string;
  available: boolean;
  has_key: boolean;
  tier: string;
  active: boolean;
}

interface TestResult {
  ok: boolean;
  message: string;
  latency_ms: number | null;
  model?: string;
}

const PROVIDER_META: Record<
  string,
  { desc: string; envKey: string; lmstudio?: boolean }
> = {
  deepseek: { desc: "DeepSeek V4，性价比极高", envKey: "DEEPSEEK_API_KEY" },
  claude: { desc: "Anthropic Claude，推理能力强", envKey: "ANTHROPIC_API_KEY" },
  openai: { desc: "OpenAI GPT-4o，通用性广", envKey: "OPENAI_API_KEY" },
  lmstudio: {
    desc: "本地模型，隐私无限制",
    envKey: "LMSTUDIO_MODEL",
    lmstudio: true,
  },
};

const STATIC_PROVIDERS: ProviderStatus[] = [
  { id: "deepseek", name: "DeepSeek", model: "deepseek-v4-pro", available: false, has_key: false, tier: "Pro", active: false },
  { id: "claude", name: "Claude", model: "claude-sonnet-4-6", available: false, has_key: false, tier: "Pro", active: false },
  { id: "openai", name: "OpenAI", model: "gpt-4o", available: false, has_key: false, tier: "Pro", active: false },
  { id: "lmstudio", name: "LM Studio", model: "local-model", available: true, has_key: true, tier: "Pro", active: false },
];

export default function ModelsPage() {
  const [providers, setProviders] = useState<ProviderStatus[]>(STATIC_PROVIDERS);
  const [selected, setSelected] = useState<string>("deepseek");
  const [testing, setTesting] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, TestResult>>(
    {}
  );
  const [switching, setSwitching] = useState<string | null>(null);

  // API Key form state
  const [keys, setKeys] = useState<Record<string, string>>({});
  const [keyVisible, setKeyVisible] = useState<Record<string, boolean>>({});
  const [savingKey, setSavingKey] = useState<string | null>(null);
  const [editingKey, setEditingKey] = useState<Record<string, boolean>>({});

  // LM Studio model
  const [lmstudioModel, setLmstudioModel] = useState("");
  const [lmstudioModelDraft, setLmstudioModelDraft] = useState("");
  const [savingLmstudioModel, setSavingLmstudioModel] = useState(false);
  const [lmstudioModelSaved, setLmstudioModelSaved] = useState(false);
  const modelDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // LM Studio config
  const [lmstudioUrl, setLmstudioUrl] = useState("http://localhost:1234/v1");
  const [lmstudioKey, setLmstudioKey] = useState("");
  const [lmstudioKeyDraft, setLmstudioKeyDraft] = useState("");
  const [savingLmstudioUrl, setSavingLmstudioUrl] = useState(false);
  const [savingLmstudioKey, setSavingLmstudioKey] = useState(false);
  const [lmstudioSaved, setLmstudioSaved] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Default model config
  const [defaultModel, setDefaultModel] = useState("deepseek-v4-pro");
  const [defaultReasoning, setDefaultReasoning] = useState("think");
  const [savingDefaults, setSavingDefaults] = useState(false);
  const [defaultsSaved, setDefaultsSaved] = useState(false);
  const [defaultsError, setDefaultsError] = useState("");

  const fetchLmstudioModel = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/config/lmstudio-model`);
      if (res.ok) {
        const data = await res.json();
        const model = data.model || "";
        setLmstudioModel(model);
        setLmstudioModelDraft(model);
      }
    } catch {}
  }, []);

  // ── 中转站 (Agent Proxy) ──
  const [relayUrl, setRelayUrl] = useState("");
  const [relayUrlDraft, setRelayUrlDraft] = useState("");
  const [relayKey, setRelayKey] = useState("");
  const [relayKeyDraft, setRelayKeyDraft] = useState("");
  const [savingRelay, setSavingRelay] = useState(false);
  const [relaySaved, setRelaySaved] = useState(false);
  const [testingRelay, setTestingRelay] = useState(false);
  const [relayTestResult, setRelayTestResult] = useState<{
    ok: boolean; models_count?: number; models?: string[];
    latency_ms?: number; chat_ok?: boolean;
  } | null>(null);
  const relayDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const fetchRelayConfig = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/config/relay`);
      if (res.ok) {
        const data = await res.json();
        setRelayUrl(data.url || "");
        setRelayUrlDraft(data.url || "");
        setRelayKey(data.has_key ? "••••••••" : "");
        setRelayKeyDraft("");
      }
    } catch {}
  }, []);

  const saveRelay = useCallback(async (url: string, key: string) => {
    if (relayDebounceRef.current) clearTimeout(relayDebounceRef.current);
    relayDebounceRef.current = setTimeout(async () => {
      setSavingRelay(true);
      try {
        const realKey = key === "••••••••" ? "" : key;
        const res = await fetch(`${API_BASE}/config/relay`, {
          method: "POST", headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url, api_key: realKey }),
        });
        if (res.ok) {
          setRelayUrl(url);
          if (key && key !== "••••••••") setRelayKey("••••••••");
          setRelaySaved(true); setTimeout(() => setRelaySaved(false), 2000);
        }
      } catch {}
      setSavingRelay(false);
    }, 800);
  }, []);

  const testRelay = async () => {
    setTestingRelay(true); setRelayTestResult(null);
    try {
      const ctrl = new AbortController();
      const t = setTimeout(() => ctrl.abort(), 15000);
      const k = relayKeyDraft === "••••••••" ? "" : relayKeyDraft;
      const res = await fetch(`${API_BASE}/config/relay/test`, {
        method: "POST", headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: relayUrlDraft, api_key: k }),
        signal: ctrl.signal,
      });
      clearTimeout(t);
      setRelayTestResult(res.ok ? await res.json() : { ok: false });
    } catch { setRelayTestResult({ ok: false }); }
    setTestingRelay(false);
  };

  const saveLmstudioModel = useCallback(async (model: string) => {
    if (modelDebounceRef.current) clearTimeout(modelDebounceRef.current);
    modelDebounceRef.current = setTimeout(async () => {
      setSavingLmstudioModel(true);
      try {
        const res = await fetch(`${API_BASE}/config/lmstudio-model`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ model }),
        });
        if (res.ok) {
          setLmstudioModel(model);
          setLmstudioModelSaved(true);
          setTimeout(() => setLmstudioModelSaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingLmstudioModel(false);
    }, 500);
  }, []);

  const fetchLmstudioUrl = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/config/lmstudio-url`);
      if (res.ok) {
        const data = await res.json();
        const url = data.url || "http://localhost:1234/v1";
        setLmstudioUrl(url);
        setLmstudioUrlDraft(url);
      }
    } catch {}
  }, []);

  const fetchLmstudioKey = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/config/lmstudio-key`);
      if (res.ok) {
        const data = await res.json();
        const key = data.key || "";
        setLmstudioKey(key);
        setLmstudioKeyDraft(key);
      }
    } catch {}
  }, []);

  const [lmstudioUrlDraft, setLmstudioUrlDraft] = useState("http://localhost:1234/v1");
  const [lmstudioUrlSaved, setLmstudioUrlSaved] = useState(false);
  const urlDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const saveLmstudioUrl = useCallback(async (url: string) => {
    if (urlDebounceRef.current) clearTimeout(urlDebounceRef.current);
    urlDebounceRef.current = setTimeout(async () => {
      setSavingLmstudioUrl(true);
      try {
        const res = await fetch(`${API_BASE}/config/lmstudio-url`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url }),
        });
        if (res.ok) {
          setLmstudioUrl(url);
          setLmstudioUrlSaved(true);
          setTimeout(() => setLmstudioUrlSaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingLmstudioUrl(false);
    }, 500);
  }, []);

  const saveLmstudioKey = useCallback(async (key: string) => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(async () => {
      setSavingLmstudioKey(true);
      try {
        const res = await fetch(`${API_BASE}/config/lmstudio-key`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ key: key || null }),
        });
        if (res.ok) {
          setLmstudioKey(key);
          setLmstudioSaved(true);
          setTimeout(() => setLmstudioSaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingLmstudioKey(false);
    }, 500);
  }, []);

  const fetchProviders = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/config/providers`);
      if (res.ok) {
        const data: ProviderStatus[] = await res.json();
        // Merge API status into static list, preserving order
        setProviders((prev) =>
          prev.map((p) => {
            const remote = data.find((d) => d.id === p.id);
            return remote ? { ...p, ...remote } : p;
          })
        );
        const active = data.find((p) => p.active);
        if (active) setSelected(active.id);
      }
    } catch {}
  }, []);

  const fetchDefaults = useCallback(async () => {
    try {
      const res = await fetch(`${API_BASE}/config/model`);
      if (res.ok) {
        const data = await res.json();
        setDefaultModel(data.model || "deepseek-v4-pro");
        setDefaultReasoning(data.reasoning || "think");
      }
    } catch {}
  }, []);

  useEffect(() => {
    fetchProviders();
    fetchDefaults();
    fetchLmstudioUrl();
    fetchLmstudioKey();
    fetchLmstudioModel();
    fetchRelayConfig();
    // Load keys from localStorage
    const stored: Record<string, string> = {};
    ["deepseek", "claude", "openai", "lmstudio"].forEach((p) => {
      stored[p] = localStorage.getItem(`apikey_${p}`) || "";
    });
    setKeys(stored);
  }, [fetchProviders, fetchDefaults, fetchLmstudioUrl, fetchLmstudioKey, fetchLmstudioModel, fetchRelayConfig]);

  const switchProvider = async (id: string) => {
    setSwitching(id);
    try {
      await fetch(`${API_BASE}/config/provider`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ provider: id }),
      });
      await fetchProviders();
    } catch {}
    setSwitching(null);
  };

  const testProvider = async (id: string, apiKey?: string) => {
    setTesting(id);
    setTestResults((prev) => {
      const next = { ...prev };
      delete next[id];
      return next;
    });
    try {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), 20000);
      const body: Record<string, unknown> = { provider: id };
      if (apiKey) body.api_key = apiKey;
      const res = await fetch(`${API_BASE}/config/provider/test`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
        signal: controller.signal,
      });
      clearTimeout(timer);
      if (res.ok) {
        const data: TestResult = await res.json();
        setTestResults((prev) => ({ ...prev, [id]: data }));
        // LM Studio 测试成功时自动回填检测到的模型名
        if (id === "lmstudio" && data.model) {
          setLmstudioModelDraft(data.model);
          saveLmstudioModel(data.model);
        }
      } else {
        setTestResults((prev) => ({
          ...prev,
          [id]: { ok: false, message: `HTTP ${res.status}`, latency_ms: null },
        }));
      }
    } catch (e) {
      setTestResults((prev) => ({
        ...prev,
        [id]: {
          ok: false,
          message: e instanceof DOMException && e.name === "AbortError"
            ? "连接超时（20秒）"
            : (e instanceof Error ? e.message : "网络错误"),
          latency_ms: null,
        },
      }));
    }
    setTesting(null);
  };

  const saveApiKey = async (provider: string) => {
    const val = keys[provider] || "";
    localStorage.setItem(`apikey_${provider}`, val);
    setSavingKey(provider);
    try {
      const map: Record<string, string> = {
        claude: "anthropic",
        openai: "openai",
        deepseek: "deepseek",
        lmstudio: "lmstudio",
      };
      await fetch(`${API_BASE}/apikey/set`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ provider: map[provider] || provider, key: val }),
      });
    } catch {}
    setSavingKey(null);
    await fetchProviders();
  };

  const saveDefaults = async () => {
    setSavingDefaults(true);
    setDefaultsError("");
    localStorage.setItem("default_model", defaultModel);
    localStorage.setItem("default_reasoning", defaultReasoning);
    try {
      const res = await fetch(`${API_BASE}/config/model`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          model: defaultModel,
          reasoning: defaultReasoning,
        }),
      });
      if (res.ok) {
        setDefaultsSaved(true);
        setTimeout(() => setDefaultsSaved(false), 2000);
      } else {
        const err = await res.text();
        setDefaultsError(`保存失败: ${err.slice(0, 100)}`);
      }
    } catch (e) {
      setDefaultsError(`网络错误: ${e instanceof Error ? e.message : "未知错误"}`);
    }
    setSavingDefaults(false);
  };

  const selectedProvider = providers.find((p) => p.id === selected);
  const meta = selected ? PROVIDER_META[selected] : null;

  return (
    <div className="flex flex-col gap-6 max-w-3xl mx-auto">
      <div>
        <h1 className="text-2xl font-bold tracking-tight flex items-center gap-2">
          <Server className="h-6 w-6" />
          模型配置
        </h1>
        <p className="text-sm text-muted-foreground mt-1">
          管理和切换 AI 模型提供商。点击卡片设为活跃。
        </p>
      </div>

      {/* ── Provider 卡片 ── */}
      <div className="grid grid-cols-2 gap-3">
        {providers.map((p) => {
          const isActive = p.active;
          const testResult = testResults[p.id];
          return (
            <button
              key={p.id}
              onClick={() => setSelected(p.id)}
              className={`relative text-left rounded-lg border p-4 transition-all ${
                isActive
                  ? "border-primary bg-primary/5 ring-1 ring-primary/20"
                  : selected === p.id
                    ? "border-primary/50 bg-card"
                    : "border-border bg-card hover:bg-muted/50"
              } ${p.id === "lmstudio" ? "col-span-2" : ""}`}
            >
              {/* Active badge */}
              {isActive && (
                <span className="absolute top-2 right-2 text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-primary text-primary-foreground">
                  活跃
                </span>
              )}

              <div className="flex items-center gap-2 mb-1">
                <span
                  className={`inline-block h-2 w-2 rounded-full shrink-0 ${
                    p.available
                      ? "bg-emerald-500"
                      : p.id === "lmstudio"
                        ? "bg-yellow-500"
                        : "bg-red-400"
                  }`}
                />
                <span className="font-semibold text-sm">{p.name}</span>
              </div>
              <p className="text-xs text-muted-foreground">{p.model}</p>
              {!p.available && p.id !== "lmstudio" && (
                <p className="text-[10px] text-red-400 mt-1">无 API Key</p>
              )}

              {testResult && (
                <div
                  className={`mt-2 text-[10px] flex items-center gap-1 ${testResult.ok ? "text-emerald-600" : "text-red-500"}`}
                >
                  {testResult.ok ? (
                    <Wifi className="h-3 w-3" />
                  ) : (
                    <WifiOff className="h-3 w-3" />
                  )}
                  {testResult.ok
                    ? `${testResult.latency_ms}ms`
                    : testResult.message.slice(0, 30)}
                </div>
              )}
            </button>
          );
        })}
      </div>

      {/* ── 选中 Provider 详情 ── */}
      {selected && selectedProvider && meta && (
        <div className="rounded-lg border bg-card p-4 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-semibold">
                {selectedProvider.name} 配置
              </h3>
              <p className="text-xs text-muted-foreground">{meta.desc}</p>
            </div>
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                onClick={() => testProvider(selected, selected === "lmstudio" ? lmstudioKey : undefined)}
                disabled={testing === selected}
              >
                {testing === selected ? (
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                ) : (
                  <RefreshCw className="h-3.5 w-3.5" />
                )}
                测试
              </Button>
              {!selectedProvider.active && selectedProvider.available && (
                <Button
                  size="sm"
                  onClick={() => switchProvider(selected)}
                  disabled={switching === selected}
                >
                  {switching === selected ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin mr-1" />
                  ) : null}
                  设为活跃
                </Button>
              )}
            </div>
          </div>

          <hr className="border-border" />

          {/* API Key input */}
          {meta.lmstudio ? (
            <>
              <div>
                <label className="text-xs font-medium block mb-1.5">
                  模型名
                </label>
                <div className="flex gap-2 items-center">
                  <Input
                    type="text"
                    placeholder="如 qwen2.5-7b-instruct"
                    value={lmstudioModelDraft}
                    onChange={(e) => {
                      const val = e.target.value;
                      setLmstudioModelDraft(val);
                      saveLmstudioModel(val);
                    }}
                    className="text-sm"
                  />
                  {savingLmstudioModel && (
                    <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground shrink-0" />
                  )}
                  {lmstudioModelSaved && !savingLmstudioModel && (
                    <span className="text-[10px] text-emerald-600 dark:text-emerald-400 shrink-0">
                      已保存
                    </span>
                  )}
                </div>
                <p className="text-[10px] text-muted-foreground mt-1">
                  对应 LMSTUDIO_MODEL 环境变量，停止输入 0.5s 后自动保存
                </p>
              </div>
              <div>
                <label className="text-xs font-medium block mb-1.5">
                  API Key
                </label>
                <div className="flex gap-2 items-center">
                  <Input
                    type="password"
                    placeholder="留空表示无认证"
                    value={lmstudioKeyDraft}
                    onChange={(e) => {
                      const val = e.target.value;
                      setLmstudioKeyDraft(val);
                      saveLmstudioKey(val);
                    }}
                    className="text-sm"
                  />
                  {savingLmstudioKey && (
                    <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground shrink-0" />
                  )}
                  {lmstudioSaved && !savingLmstudioKey && (
                    <span className="text-[10px] text-emerald-600 dark:text-emerald-400 shrink-0">
                      已保存
                    </span>
                  )}
                </div>
                <p className="text-[10px] text-muted-foreground mt-1">
                  LM Studio 服务端设置的认证 token，停止输入 0.5s 后自动保存
                </p>
              </div>
              <div>
                <label className="text-xs font-medium block mb-1.5">
                  端点地址
                </label>
                <div className="flex gap-2 items-center">
                  <Input
                    type="text"
                    placeholder="http://localhost:1234/v1"
                    value={lmstudioUrlDraft}
                    onChange={(e) => {
                      const val = e.target.value;
                      setLmstudioUrlDraft(val);
                      saveLmstudioUrl(val);
                    }}
                    className="text-sm"
                  />
                  {savingLmstudioUrl && (
                    <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground shrink-0" />
                  )}
                  {lmstudioUrlSaved && !savingLmstudioUrl && (
                    <span className="text-[10px] text-emerald-600 dark:text-emerald-400 shrink-0">
                      已保存
                    </span>
                  )}
                </div>
                <p className="text-[10px] text-muted-foreground mt-1">
                  或通过 LMSTUDIO_BASE_URL 环境变量设置
                </p>
              </div>
            </>
          ) : (
            <div>
              <label className="text-xs font-medium block mb-1.5">
                API Key
              </label>

              {/* 已配置：显示遮罩状态 */}
              {selectedProvider.has_key && !editingKey[selected] ? (
                <div className="flex items-center gap-2">
                  <div className="flex-1 rounded-md border bg-muted/50 px-3 py-2 text-sm text-muted-foreground">
                    ••••••••••••••••
                  </div>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => setEditingKey((prev) => ({ ...prev, [selected]: true }))}
                  >
                    更改
                  </Button>
                </div>
              ) : (
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <Input
                      type={keyVisible[selected] ? "text" : "password"}
                      placeholder={`输入 ${selectedProvider.name} API Key...`}
                      value={keys[selected] || ""}
                      onChange={(e) =>
                        setKeys((prev) => ({
                          ...prev,
                          [selected]: e.target.value,
                        }))
                      }
                      className="pr-8 text-sm"
                    />
                    <button
                      type="button"
                      className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                      onClick={() =>
                        setKeyVisible((prev) => ({
                          ...prev,
                          [selected]: !prev[selected],
                        }))
                      }
                    >
                      {keyVisible[selected] ? (
                        <EyeOff className="h-3.5 w-3.5" />
                      ) : (
                        <Eye className="h-3.5 w-3.5" />
                      )}
                    </button>
                  </div>
                  <Button
                    size="sm"
                    variant={savingKey === selected ? "default" : "outline"}
                    onClick={() => {
                      saveApiKey(selected);
                      setEditingKey((prev) => ({ ...prev, [selected]: false }));
                    }}
                    disabled={savingKey === selected}
                  >
                    {savingKey === selected ? (
                      <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    ) : (
                      <Check className="h-3.5 w-3.5" />
                    )}
                  </Button>
                  {selectedProvider.has_key && (
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => {
                        setKeys((prev) => ({ ...prev, [selected]: "" }));
                        setEditingKey((prev) => ({ ...prev, [selected]: false }));
                      }}
                    >
                      <X className="h-3.5 w-3.5" />
                    </Button>
                  )}
                </div>
              )}
              <p className="text-[10px] text-muted-foreground mt-1">
                或设置环境变量 {meta.envKey}
              </p>
            </div>
          )}

          {/* Test result detail */}
          {testResults[selected] && (
            <div
              className={`text-xs rounded-md p-2 ${
                testResults[selected].ok
                  ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
                  : "bg-destructive/10 text-destructive"
              }`}
            >
              {testResults[selected].ok
                ? `连通成功，延迟 ${testResults[selected].latency_ms}ms`
                : `连通失败：${testResults[selected].message}`}
            </div>
          )}
        </div>
      )}

      {/* ── 中转站 (Agent Proxy) ── */}
      <div className="rounded-lg border bg-card p-4 space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-sm font-semibold flex items-center gap-2">
              <Server className="h-4 w-4" />
              中转站配置
            </h3>
            <p className="text-xs text-muted-foreground mt-0.5">
              统一管理所有 AI 调用——设置 Agent Proxy 地址后，DeepSeek/Claude/OpenAI 全部走中转站
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={testRelay}
              disabled={testingRelay}
            >
              {testingRelay ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin mr-1" />
              ) : (
                <Wifi className="h-3.5 w-3.5 mr-1" />
              )}
              测试连接
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs font-medium block mb-1.5">
              Base URL
            </label>
            <div className="flex gap-2 items-center">
              <Input
                type="text"
                placeholder="https://your-relay-server/v1"
                value={relayUrlDraft}
                onChange={(e) => {
                  const val = e.target.value;
                  setRelayUrlDraft(val);
                  saveRelay(val, relayKeyDraft);
                }}
                className="text-sm"
              />
              {savingRelay && (
                <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground shrink-0" />
              )}
              {relaySaved && !savingRelay && (
                <span className="text-[10px] text-emerald-600 shrink-0">已保存</span>
              )}
            </div>
            <p className="text-[10px] text-muted-foreground mt-1">
              对应环境变量 AI_RELAY_URL
            </p>
          </div>
          <div>
            <label className="text-xs font-medium block mb-1.5">
              API Key
            </label>
            <Input
              type="password"
              placeholder="在 Agent Proxy 后台创建"
              value={relayKeyDraft}
              onChange={(e) => {
                const val = e.target.value;
                setRelayKeyDraft(val);
                saveRelay(relayUrlDraft, val);
              }}
              className="text-sm"
            />
            <p className="text-[10px] text-muted-foreground mt-1">
              对应环境变量 AGENT_PROXY_KEY
            </p>
          </div>
        </div>

        {/* 测试结果 */}
        {relayTestResult && (
          <div
            className={`text-xs rounded-md p-3 space-y-1 ${
              relayTestResult.ok
                ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
                : "bg-destructive/10 text-destructive"
            }`}
          >
            <div className="font-medium">
              {relayTestResult.ok ? "连接成功" : "连接失败"}
              {relayTestResult.latency_ms != null && (
                <span className="ml-2 font-normal opacity-70">{relayTestResult.latency_ms}ms</span>
              )}
            </div>
            {relayTestResult.models_count != null && (
              <div>
                可用模型：{relayTestResult.models_count} 个
                {relayTestResult.models && relayTestResult.models.length > 0 && (
                  <span className="opacity-70">
                    {" "}（{relayTestResult.models.slice(0, 8).join("、")}
                    {relayTestResult.models.length > 8 ? "…" : ""}）
                  </span>
                )}
              </div>
            )}
            {relayTestResult.chat_ok !== undefined && (
              <div>Chat API：{relayTestResult.chat_ok ? "正常" : "异常"}</div>
            )}
          </div>
        )}
      </div>

      {/* ── 默认模型配置 ── */}
      <div className="rounded-lg border bg-card p-4 space-y-4">
        <h3 className="text-sm font-semibold">默认模型配置</h3>
        <p className="text-xs text-muted-foreground">
          未指定模型时使用的默认值
        </p>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-xs font-medium block mb-1.5">
              默认模型
            </label>
            <Select
              value={defaultModel}
              onValueChange={(v) => setDefaultModel(v ?? "")}
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="选择模型" />
              </SelectTrigger>
              <SelectContent>
                {DEEPSEEK_MODELS_NO_DEFAULT.map((m) => (
                  <SelectItem key={m.value} value={m.value}>
                    {m.label}
                  </SelectItem>
                ))}
                {lmstudioModel && (
                  <SelectItem key="lmstudio" value={lmstudioModel}>
                    LM Studio ({lmstudioModel}) - 本地模型
                  </SelectItem>
                )}
              </SelectContent>
            </Select>
          </div>
          <div>
            <label className="text-xs font-medium block mb-1.5">
              推理强度
            </label>
            <Select
              value={defaultReasoning}
              onValueChange={(v) => setDefaultReasoning(v ?? "")}
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="选择强度" />
              </SelectTrigger>
              <SelectContent>
                {REASONING_OPTIONS.filter((r) => r.value).map((r) => (
                  <SelectItem key={r.value} value={r.value}>
                    {r.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <Button
          size="sm"
          variant={savingDefaults ? "default" : "outline"}
          onClick={saveDefaults}
          disabled={savingDefaults}
          className="w-full"
        >
          {savingDefaults ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin mr-1" />
          ) : null}
          保存默认配置
        </Button>
        {defaultsSaved && (
          <p className="text-[10px] text-emerald-600 text-center">已保存到服务端</p>
        )}
        {defaultsError && (
          <p className="text-[10px] text-red-500 text-center">{defaultsError}</p>
        )}
      </div>
    </div>
  );
}
