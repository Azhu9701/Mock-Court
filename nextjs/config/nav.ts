import type { ComponentType } from "react";
import { BarChart3, Brain, Cpu, Globe, History, Search, Users } from "lucide-react";

export interface NavItem {
  key: string;
  label: string;
  href: string;
  icon: ComponentType<{ className?: string }>;
}

export interface NavGroup {
  label: string;
  items: NavItem[];
}

/** 基础导航配置（哲学/通用领域） */
const BASE_NAV: NavGroup[] = [
  {
    label: "核心",
    items: [
      { key: "souls", label: "魂", href: "/souls", icon: Users },
      { key: "possess", label: "附体", href: "/possess", icon: Brain },
    ],
  },
  {
    label: "回顾",
    items: [
      { key: "sessions", label: "合议记录", href: "/sessions", icon: History },
      { key: "analytics", label: "统计", href: "/analytics", icon: BarChart3 },
      { key: "knowledge", label: "知识库", href: "/knowledge", icon: Search },
    ],
  },
  {
    label: "工具",
    items: [
      { key: "models", label: "模型配置", href: "/models", icon: Cpu },
      { key: "searxng", label: "SearXNG 搜索", href: "/searxng", icon: Globe },
    ],
  },
];

/** 法庭领域导航配置 */
const COURT_NAV: NavGroup[] = [
  {
    label: "核心",
    items: [
      { key: "souls", label: "角色", href: "/souls", icon: Users },
      { key: "possess", label: "开庭", href: "/possess", icon: Brain },
    ],
  },
  {
    label: "回顾",
    items: [
      { key: "sessions", label: "庭审记录", href: "/sessions", icon: History },
      { key: "analytics", label: "庭审统计", href: "/analytics", icon: BarChart3 },
      { key: "knowledge", label: "知识库", href: "/knowledge", icon: Search },
    ],
  },
  {
    label: "工具",
    items: [
      { key: "models", label: "模型配置", href: "/models", icon: Cpu },
      { key: "searxng", label: "SearXNG 搜索", href: "/searxng", icon: Globe },
    ],
  },
];

/** 导出给非 React 上下文使用的静态 nav（向后兼容） */
export const navConfig: NavGroup[] = COURT_NAV;

/** 根据领域 profile 返回对应的导航配置 */
export function getNavConfig(profile: string): NavGroup[] {
  return profile === "court" ? COURT_NAV : BASE_NAV;
}
