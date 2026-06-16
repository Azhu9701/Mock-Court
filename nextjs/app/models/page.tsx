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
  models?: string[];
}

const PROVIDER_META: Record<
  string,
  { desc: string; envKey: string; lmstudio?: boolean; endpointConfig?: boolean }
> = {
  deepseek: { desc: "DeepSeek V4，性价比极高", envKey: "DEEPSEEK_API_KEY" },
  claude: { desc: "Anthropic Claude，推理能力强", envKey: "ANTHROPIC_API_KEY", endpointConfig: true },
  openai: { desc: "OpenAI GPT-4o，通用性广", envKey: "OPENAI_API_KEY", endpointConfig: true },
  lmstudio: {
    desc: "本地模型，隐私无限制",
    envKey: "LMSTUDIO_MODEL",
    lmstudio: true,
  },
};

// Claude endpoint 兼容预设——点击后自动填充 URL/Model
const CLAUDE_PRESETS = [
  {
    id: "anthropic",
    label: "Anthropic 官方",
    url: "https://api.anthropic.com",
    model: "claude-sonnet-4-6",
  },
  {
    id: "kimi",
    label: "Kimi Code",
    url: "https://api.kimi.com/coding/v1",
    model: "claude-sonnet-4-6",
  },
] as const;

// OpenAI endpoint 兼容预设——点击后自动填充 URL/Model
const OPENAI_PRESETS = [
  {
    id: "openai",
    label: "OpenAI 官方",
    url: "https://api.openai.com/v1",
    model: "gpt-4o",
  },
  {
    id: "kimi",
    label: "Kimi Code (OpenAI 兼容)",
    url: "https://api.kimi.com/coding/v1",
    model: "kimi-for-coding",
  },
] as const;

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

  // OpenAI config
  const [openaiUrl, setOpenaiUrl] = useState("https://api.openai.com/v1");
  const [openaiUrlDraft, setOpenaiUrlDraft] = useState("https://api.openai.com/v1");
  const [openaiKey, setOpenaiKey] = useState("");
  const [openaiKeyDraft, setOpenaiKeyDraft] = useState("");
  const [openaiModel, setOpenaiModel] = useState("gpt-4o");
  const [openaiModelDraft, setOpenaiModelDraft] = useState("gpt-4o");
  const [openaiFetchedModels, setOpenaiFetchedModels] = useState<string[]>([]);
  const [savingOpenaiUrl, setSavingOpenaiUrl] = useState(false);
  const [openaiUrlSaved, setOpenaiUrlSaved] = useState(false);
  const [savingOpenaiKey, setSavingOpenaiKey] = useState(false);
  const [openaiKeySaved, setOpenaiKeySaved] = useState(false);
  const [savingOpenaiModel, setSavingOpenaiModel] = useState(false);
  const [openaiModelSaved, setOpenaiModelSaved] = useState(false);
  const openaiUrlDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const openaiKeyDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const openaiModelDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [testingOpenaiEndpoint, setTestingOpenaiEndpoint] = useState(false);
  const [openaiEndpointTestResult, setOpenaiEndpointTestResult] = useState<{
    ok: boolean; message?: string; latency_ms?: number; models?: string[];
  } | null>(null);

  // Claude config
  const [claudeUrl, setClaudeUrl] = useState("https://api.anthropic.com/v1");
  const [claudeUrlDraft, setClaudeUrlDraft] = useState("https://api.anthropic.com/v1");
  const [claudeKey, setClaudeKey] = useState("");
  const [claudeKeyDraft, setClaudeKeyDraft] = useState("");
  const [claudeModel, setClaudeModel] = useState("claude-sonnet-4-6");
  const [claudeModelDraft, setClaudeModelDraft] = useState("claude-sonnet-4-6");
  const [savingClaudeUrl, setSavingClaudeUrl] = useState(false);
  const [claudeUrlSaved, setClaudeUrlSaved] = useState(false);
  const [savingClaudeKey, setSavingClaudeKey] = useState(false);
  const [claudeKeySaved, setClaudeKeySaved] = useState(false);
  const [savingClaudeModel, setSavingClaudeModel] = useState(false);
  const [claudeModelSaved, setClaudeModelSaved] = useState(false);
  const claudeUrlDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const claudeKeyDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const claudeModelDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [testingClaudeEndpoint, setTestingClaudeEndpoint] = useState(false);
  const [claudeEndpointTestResult, setClaudeEndpointTestResult] = useState<{
    ok: boolean; message?: string; latency_ms?: number;
  } | null>(null);

  const fetchOpenaiConfig = useCallback(async () => {
    try {
      const [urlRes, keyRes, modelRes] = await Promise.all([
        fetch(`${API_BASE}/config/openai-url`),
        fetch(`${API_BASE}/config/openai-key`),
        fetch(`${API_BASE}/config/openai-model`),
      ]);
      if (urlRes.ok) {
        const data = await urlRes.json();
        const url = data.url || "https://api.openai.com/v1";
        setOpenaiUrl(url);
        setOpenaiUrlDraft(url);
      }
      if (keyRes.ok) {
        const data = await keyRes.json();
        const masked = data.has_key ? "••••••••" : "";
        setOpenaiKey(masked);
        setOpenaiKeyDraft(masked);
      }
      if (modelRes.ok) {
        const data = await modelRes.json();
        const model = data.model || "gpt-4o";
        setOpenaiModel(model);
        setOpenaiModelDraft(model);
      }
    } catch {}
  }, []);

  const fetchClaudeConfig = useCallback(async () => {
    try {
      const [urlRes, keyRes, modelRes] = await Promise.all([
        fetch(`${API_BASE}/config/claude-url`),
        fetch(`${API_BASE}/config/claude-key`),
        fetch(`${API_BASE}/config/claude-model`),
      ]);
      if (urlRes.ok) {
        const data = await urlRes.json();
        const url = data.url || "https://api.anthropic.com/v1";
        setClaudeUrl(url);
        setClaudeUrlDraft(url);
      }
      if (keyRes.ok) {
        const data = await keyRes.json();
        const masked = data.has_key ? "••••••••" : "";
        setClaudeKey(masked);
        setClaudeKeyDraft(masked);
      }
      if (modelRes.ok) {
        const data = await modelRes.json();
        const model = data.model || "claude-sonnet-4-6";
        setClaudeModel(model);
        setClaudeModelDraft(model);
      }
    } catch {}
  }, []);

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

  // Cleanup all debounce timers on unmount
  useEffect(() => {
    return () => {
      [
        modelDebounceRef, debounceRef, relayDebounceRef, urlDebounceRef,
        openaiUrlDebounceRef, openaiKeyDebounceRef, openaiModelDebounceRef,
        claudeUrlDebounceRef, claudeKeyDebounceRef, claudeModelDebounceRef,
      ].forEach((ref) => { if (ref.current) clearTimeout(ref.current); });
    };
  }, []);

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
    fetchOpenaiConfig();
    fetchClaudeConfig();
    // Load keys from localStorage
    const stored: Record<string, string> = {};
    ["deepseek", "claude", "openai", "lmstudio"].forEach((p) => {
      stored[p] = localStorage.getItem(`apikey_${p}`) || "";
    });
    setKeys(stored);
  }, [fetchProviders, fetchDefaults, fetchLmstudioUrl, fetchLmstudioKey, fetchLmstudioModel, fetchRelayConfig, fetchOpenaiConfig, fetchClaudeConfig]);

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

      if (id === "openai") {
        const key = openaiKeyDraft === "••••••••" ? "" : openaiKeyDraft;
        const res = await fetch(`${API_BASE}/config/openai/test`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url: openaiUrlDraft, api_key: key || undefined }),
          signal: controller.signal,
        });
        clearTimeout(timer);
        const data = res.ok ? await res.json() : { ok: false, message: `HTTP ${res.status}` };
        setTestResults((prev) => ({
          ...prev,
          [id]: { ok: data.ok, message: data.message || "", latency_ms: data.latency_ms ?? null, model: data.model, models: data.models },
        }));
        if (Array.isArray(data.models)) {
          setOpenaiFetchedModels(data.models);
          if (!data.models.includes(openaiModelDraft) && data.models.length > 0) {
            setOpenaiModelDraft(data.models[0]);
            saveOpenaiModel(data.models[0]);
          }
        }
      } else if (id === "claude") {
        const key = claudeKeyDraft === "••••••••" ? "" : claudeKeyDraft;
        const res = await fetch(`${API_BASE}/config/claude/test`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url: claudeUrlDraft, api_key: key || undefined }),
          signal: controller.signal,
        });
        clearTimeout(timer);
        const data = res.ok ? await res.json() : { ok: false, message: `HTTP ${res.status}` };
        setTestResults((prev) => ({
          ...prev,
          [id]: { ok: data.ok, message: data.message || "", latency_ms: data.latency_ms ?? null, model: data.model },
        }));
      } else {
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

  const testOpenaiEndpoint = async () => {
    setTestingOpenaiEndpoint(true);
    setOpenaiEndpointTestResult(null);
    try {
      const key = openaiKeyDraft === "••••••••" ? "" : openaiKeyDraft;
      const ctrl = new AbortController();
      const t = setTimeout(() => ctrl.abort(), 15000);
      const res = await fetch(`${API_BASE}/config/openai/test`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: openaiUrlDraft, api_key: key || undefined }),
        signal: ctrl.signal,
      });
      clearTimeout(t);
      const data = res.ok ? await res.json() : { ok: false, message: `HTTP ${res.status}` };
      setOpenaiEndpointTestResult({ ok: data.ok, message: data.message, latency_ms: data.latency_ms, models: data.models });
      if (Array.isArray(data.models)) {
        setOpenaiFetchedModels(data.models);
        if (!data.models.includes(openaiModelDraft) && data.models.length > 0) {
          setOpenaiModelDraft(data.models[0]);
          saveOpenaiModel(data.models[0]);
        }
      }
    } catch {
      setOpenaiEndpointTestResult({ ok: false, message: "网络错误" });
    }
    setTestingOpenaiEndpoint(false);
  };

  const testClaudeEndpoint = async () => {
    setTestingClaudeEndpoint(true);
    setClaudeEndpointTestResult(null);
    try {
      const key = claudeKeyDraft === "••••••••" ? "" : claudeKeyDraft;
      const ctrl = new AbortController();
      const t = setTimeout(() => ctrl.abort(), 15000);
      const res = await fetch(`${API_BASE}/config/claude/test`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: claudeUrlDraft, api_key: key || undefined }),
        signal: ctrl.signal,
      });
      clearTimeout(t);
      const data = res.ok ? await res.json() : { ok: false, message: `HTTP ${res.status}` };
      setClaudeEndpointTestResult({ ok: data.ok, message: data.message, latency_ms: data.latency_ms });
    } catch {
      setClaudeEndpointTestResult({ ok: false, message: "网络错误" });
    }
    setTestingClaudeEndpoint(false);
  };

  const saveOpenaiUrl = useCallback(async (url: string) => {
    if (openaiUrlDebounceRef.current) clearTimeout(openaiUrlDebounceRef.current);
    openaiUrlDebounceRef.current = setTimeout(async () => {
      setSavingOpenaiUrl(true);
      try {
        const res = await fetch(`${API_BASE}/config/openai-url`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url }),
        });
        if (res.ok) {
          setOpenaiUrl(url);
          setOpenaiUrlSaved(true);
          setTimeout(() => setOpenaiUrlSaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingOpenaiUrl(false);
    }, 500);
  }, [fetchProviders]);

  const saveOpenaiKey = useCallback(async (keyDraft: string) => {
    if (openaiKeyDebounceRef.current) clearTimeout(openaiKeyDebounceRef.current);
    openaiKeyDebounceRef.current = setTimeout(async () => {
      setSavingOpenaiKey(true);
      try {
        const realKey = keyDraft === "••••••••" ? "" : keyDraft;
        const res = await fetch(`${API_BASE}/config/openai-key`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ key: realKey || null }),
        });
        if (res.ok) {
          setOpenaiKey(realKey ? "••••••••" : "");
          setOpenaiKeySaved(true);
          setTimeout(() => setOpenaiKeySaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingOpenaiKey(false);
    }, 500);
  }, [fetchProviders]);

  const saveOpenaiModel = useCallback(async (model: string) => {
    if (openaiModelDebounceRef.current) clearTimeout(openaiModelDebounceRef.current);
    openaiModelDebounceRef.current = setTimeout(async () => {
      setSavingOpenaiModel(true);
      try {
        const res = await fetch(`${API_BASE}/config/openai-model`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ model }),
        });
        if (res.ok) {
          setOpenaiModel(model);
          setOpenaiModelSaved(true);
          setTimeout(() => setOpenaiModelSaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingOpenaiModel(false);
    }, 300);
  }, [fetchProviders]);

  const saveClaudeUrl = useCallback(async (url: string) => {
    if (claudeUrlDebounceRef.current) clearTimeout(claudeUrlDebounceRef.current);
    claudeUrlDebounceRef.current = setTimeout(async () => {
      setSavingClaudeUrl(true);
      try {
        const res = await fetch(`${API_BASE}/config/claude-url`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url }),
        });
        if (res.ok) {
          setClaudeUrl(url);
          setClaudeUrlSaved(true);
          setTimeout(() => setClaudeUrlSaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingClaudeUrl(false);
    }, 500);
  }, [fetchProviders]);

  const saveClaudeKey = useCallback(async (keyDraft: string) => {
    if (claudeKeyDebounceRef.current) clearTimeout(claudeKeyDebounceRef.current);
    claudeKeyDebounceRef.current = setTimeout(async () => {
      setSavingClaudeKey(true);
      try {
        const realKey = keyDraft === "••••••••" ? "" : keyDraft;
        const res = await fetch(`${API_BASE}/config/claude-key`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ key: realKey || null }),
        });
        if (res.ok) {
          setClaudeKey(realKey ? "••••••••" : "");
          setClaudeKeySaved(true);
          setTimeout(() => setClaudeKeySaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingClaudeKey(false);
    }, 500);
  }, [fetchProviders]);

  const saveClaudeModel = useCallback(async (model: string) => {
    if (claudeModelDebounceRef.current) clearTimeout(claudeModelDebounceRef.current);
    claudeModelDebounceRef.current = setTimeout(async () => {
      setSavingClaudeModel(true);
      try {
        const res = await fetch(`${API_BASE}/config/claude-model`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ model }),
        });
        if (res.ok) {
          setClaudeModel(model);
          setClaudeModelSaved(true);
          setTimeout(() => setClaudeModelSaved(false), 2000);
          fetchProviders();
        }
      } catch {}
      setSavingClaudeModel(false);
    }, 300);
  }, [fetchProviders]);

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
      if (provider === "openai") {
        await fetch(`${API_BASE}/config/openai-key`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ key: val || null }),
        });
        setOpenaiKey(val ? "••••••••" : "");
      }
      if (provider === "claude") {
        await fetch(`${API_BASE}/config/claude-key`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ key: val || null }),
        });
        setClaudeKey(val ? "••••••••" : "");
      }
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

          {/* Provider config */}
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
          ) : meta.endpointConfig ? (
            <div className="space-y-4">
              <div>
                <label className="text-xs font-medium block mb-1.5">
                  Base URL
                </label>

                {/* endpoint 兼容预设：OpenAI / Claude 各自的快捷配置 */}
                {selected === "openai" && (
                  <div className="flex flex-wrap gap-1.5 mb-2">
                    {OPENAI_PRESETS.map((preset) => (
                      <button
                        key={preset.id}
                        type="button"
                        onClick={() => {
                          setOpenaiUrlDraft(preset.url);
                          setOpenaiModelDraft(preset.model);
                          saveOpenaiUrl(preset.url);
                          saveOpenaiModel(preset.model);
                        }}
                        className={`text-[10px] px-2 py-1 rounded-md border transition-colors ${
                          openaiUrlDraft === preset.url
                            ? "border-primary bg-primary/10 text-primary"
                            : "border-border text-muted-foreground hover:bg-muted/50"
                        }`}
                      >
                        {preset.label}
                      </button>
                    ))}
                  </div>
                )}
                {selected === "claude" && (
                  <div className="flex flex-wrap gap-1.5 mb-2">
                    {CLAUDE_PRESETS.map((preset) => (
                      <button
                        key={preset.id}
                        type="button"
                        onClick={() => {
                          setClaudeUrlDraft(preset.url);
                          setClaudeModelDraft(preset.model);
                          saveClaudeUrl(preset.url);
                          saveClaudeModel(preset.model);
                        }}
                        className={`text-[10px] px-2 py-1 rounded-md border transition-colors ${
                          claudeUrlDraft === preset.url
                            ? "border-primary bg-primary/10 text-primary"
                            : "border-border text-muted-foreground hover:bg-muted/50"
                        }`}
                      >
                        {preset.label}
                      </button>
                    ))}
                  </div>
                )}
                <div className="flex gap-2 items-center">
                  <Input
                    type="text"
                    placeholder={selected === "openai" ? "https://api.openai.com/v1" : "https://api.anthropic.com/v1"}
                    value={selected === "openai" ? openaiUrlDraft : claudeUrlDraft}
                    onChange={(e) => {
                      const val = e.target.value;
                      if (selected === "openai") {
                        setOpenaiUrlDraft(val);
                        saveOpenaiUrl(val);
                      } else {
                        setClaudeUrlDraft(val);
                        saveClaudeUrl(val);
                      }
                    }}
                    className="text-sm"
                  />
                  {(selected === "openai" ? savingOpenaiUrl : savingClaudeUrl) && (
                    <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground shrink-0" />
                  )}
                  {(selected === "openai" ? openaiUrlSaved && !savingOpenaiUrl : claudeUrlSaved && !savingClaudeUrl) && (
                    <span className="text-[10px] text-emerald-600 dark:text-emerald-400 shrink-0">
                      已保存
                    </span>
                  )}
                </div>
                <p className="text-[10px] text-muted-foreground mt-1">
                  支持官方地址或 OpenAI-compatible / Anthropic-compatible 中转地址
                </p>
              </div>

              <div>
                <label className="text-xs font-medium block mb-1.5">
                  API Key
                </label>
                <div className="flex gap-2 items-center">
                  <Input
                    type="password"
                    placeholder={selected === "openai" ? "sk-..." : "sk-ant-..."}
                    value={selected === "openai" ? openaiKeyDraft : claudeKeyDraft}
                    onChange={(e) => {
                      const val = e.target.value;
                      if (selected === "openai") {
                        setOpenaiKeyDraft(val);
                        saveOpenaiKey(val);
                      } else {
                        setClaudeKeyDraft(val);
                        saveClaudeKey(val);
                      }
                    }}
                    className="text-sm"
                  />
                  {(selected === "openai" ? savingOpenaiKey : savingClaudeKey) && (
                    <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground shrink-0" />
                  )}
                  {(selected === "openai" ? openaiKeySaved && !savingOpenaiKey : claudeKeySaved && !savingClaudeKey) && (
                    <span className="text-[10px] text-emerald-600 dark:text-emerald-400 shrink-0">
                      已保存
                    </span>
                  )}
                </div>
                <p className="text-[10px] text-muted-foreground mt-1">
                  或设置环境变量 {meta.envKey}
                </p>
              </div>

              <div>
                <label className="text-xs font-medium block mb-1.5">
                  模型
                </label>
                {selected === "openai" && openaiFetchedModels.length > 0 ? (
                  <div className="space-y-2">
                    <Select
                      value={openaiModelDraft || undefined}
                      onValueChange={(v) => {
                        if (!v) return;
                        setOpenaiModelDraft(v);
                        saveOpenaiModel(v);
                      }}
                    >
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="选择 OpenAI 模型" />
                      </SelectTrigger>
                      <SelectContent>
                        {openaiFetchedModels.map((m) => (
                          <SelectItem key={m} value={m}>
                            {m}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    {openaiModel && (
                      <p className="text-[10px] text-muted-foreground">
                        当前激活：{openaiModel}
                      </p>
                    )}
                  </div>
                ) : (
                  <div className="flex gap-2 items-center">
                    <Input
                      type="text"
                      placeholder={selected === "openai" ? "gpt-4o" : "claude-sonnet-4-6"}
                      value={selected === "openai" ? openaiModelDraft : claudeModelDraft}
                      onChange={(e) => {
                        const val = e.target.value;
                        if (selected === "openai") {
                          setOpenaiModelDraft(val);
                          saveOpenaiModel(val);
                        } else {
                          setClaudeModelDraft(val);
                          saveClaudeModel(val);
                        }
                      }}
                      className="text-sm"
                    />
                    {(selected === "openai" ? savingOpenaiModel : savingClaudeModel) && (
                      <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground shrink-0" />
                    )}
                    {(selected === "openai" ? openaiModelSaved && !savingOpenaiModel : claudeModelSaved && !savingClaudeModel) && (
                      <span className="text-[10px] text-emerald-600 dark:text-emerald-400 shrink-0">
                        已保存
                      </span>
                    )}
                  </div>
                )}
                {selected === "openai" && (
                  <p className="text-[10px] text-muted-foreground mt-1">
                    点击测试后会自动拉取该 Base URL 下的模型列表
                  </p>
                )}
                {selected === "claude" && (
                  <p className="text-[10px] text-muted-foreground mt-1">
                    Anthropic 官方无标准 /models 列表，模型名需手动填写
                  </p>
                )}
              </div>

              <div className="flex items-center gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={selected === "openai" ? testOpenaiEndpoint : testClaudeEndpoint}
                  disabled={selected === "openai" ? testingOpenaiEndpoint : testingClaudeEndpoint}
                >
                  {(selected === "openai" ? testingOpenaiEndpoint : testingClaudeEndpoint) ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin mr-1" />
                  ) : (
                    <Wifi className="h-3.5 w-3.5 mr-1" />
                  )}
                  测试连通
                </Button>
              </div>

              {(selected === "openai" ? openaiEndpointTestResult : claudeEndpointTestResult) && (
                <div
                  className={`text-xs rounded-md p-3 space-y-1 ${
                    (selected === "openai" ? openaiEndpointTestResult?.ok : claudeEndpointTestResult?.ok)
                      ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
                      : "bg-destructive/10 text-destructive"
                  }`}
                >
                  <div className="font-medium">
                    {(selected === "openai" ? openaiEndpointTestResult?.ok : claudeEndpointTestResult?.ok) ? "连接成功" : "连接失败"}
                    {(selected === "openai" ? openaiEndpointTestResult?.latency_ms : claudeEndpointTestResult?.latency_ms) != null && (
                      <span className="ml-2 font-normal opacity-70">
                        {selected === "openai" ? openaiEndpointTestResult?.latency_ms : claudeEndpointTestResult?.latency_ms}ms
                      </span>
                    )}
                  </div>
                  {(selected === "openai" ? openaiEndpointTestResult?.message : claudeEndpointTestResult?.message) && (
                    <div>
                      {selected === "openai" ? openaiEndpointTestResult?.message : claudeEndpointTestResult?.message}
                    </div>
                  )}
                  {selected === "openai" && Array.isArray(openaiEndpointTestResult?.models) && openaiEndpointTestResult.models.length > 0 && (
                    <div>
                      拉取到 {openaiEndpointTestResult.models.length} 个模型
                      <span className="opacity-70">
                        {" "}（{openaiEndpointTestResult.models.slice(0, 8).join("、")}
                        {openaiEndpointTestResult.models.length > 8 ? "…" : ""}）
                      </span>
                    </div>
                  )}
                </div>
              )}
            </div>
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
              {Array.isArray(testResults[selected].models) && testResults[selected].models!.length > 0 && (
                <div className="mt-1 opacity-80">
                  可用模型：{testResults[selected].models!.length} 个
                  {" "}（{testResults[selected].models!.slice(0, 8).join("、")}
                  {testResults[selected].models!.length > 8 ? "…" : ""}）
                </div>
              )}
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
