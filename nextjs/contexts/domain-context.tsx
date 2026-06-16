"use client";

import { createContext, useCallback, useContext, useEffect, useState } from "react";
import { getDomainInfo, setDomain as apiSetDomain, type DomainInfo } from "@/lib/api";

// 内置领域预设——用于初始显示（在 API 响应前）
const DOMAIN_PRESETS: Record<string, { icon: string; label: string }> = {
  philosophy: { icon: "🧭", label: "万民幡" },
  legal: { icon: "⚖️", label: "法律智囊团" },
  labor: { icon: "🛡️", label: "工友智囊团" },
};

interface DomainContextType {
  profile: string;
  systemName: string;
  agentNoun: string;
  synthesisVerb: string;
  dimensions: string[];
  enabledModes: string[];
  icon: string;
  ready: boolean;
  switchDomain: (profile: string) => Promise<void>;
}

const DomainContext = createContext<DomainContextType | undefined>(undefined);

const DEFAULT_STATE: Omit<DomainContextType, "switchDomain"> = {
  profile: "philosophy",
  systemName: "万民幡",
  agentNoun: "魂",
  synthesisVerb: "辩证综合",
  dimensions: ["场域", "本体论", "认识论", "目的论"],
  enabledModes: ["single", "conference", "debate", "relay", "learn", "practice_opening"],
  icon: "🧭",
  ready: false,
};

export function DomainProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState(DEFAULT_STATE);

  // 启动后从后端同步当前领域（以服务端为准，而非 localStorage）
  useEffect(() => {
    let cancelled = false;
    getDomainInfo()
      .then((info: DomainInfo) => {
        if (cancelled) return;
        const preset = DOMAIN_PRESETS[info.profile];
        setState({
          profile: info.profile,
          systemName: info.system_name,
          agentNoun: info.agent_noun,
          synthesisVerb: info.synthesis_verb,
          dimensions: info.dimensions,
          enabledModes: info.enabled_modes,
          icon: preset?.icon ?? "🌐",
          ready: true,
        });
      })
      .catch(() => {
        if (!cancelled) setState((s) => ({ ...s, ready: true }));
      });
    return () => { cancelled = true; };
  }, []);

  const switchDomain = useCallback(async (profile: string) => {
    const info = await apiSetDomain(profile);
    const preset = DOMAIN_PRESETS[info.profile] ?? DOMAIN_PRESETS[profile];
    setState({
      profile: info.profile,
      systemName: info.system_name,
      agentNoun: info.agent_noun,
      synthesisVerb: info.synthesis_verb,
      dimensions: info.dimensions,
      enabledModes: info.enabled_modes,
      icon: preset?.icon ?? "🌐",
      ready: true,
    });
  }, []);

  return (
    <DomainContext.Provider value={{ ...state, switchDomain }}>
      {children}
    </DomainContext.Provider>
  );
}

export function useDomain() {
  const ctx = useContext(DomainContext);
  if (!ctx) throw new Error("useDomain must be used within DomainProvider");
  return ctx;
}
