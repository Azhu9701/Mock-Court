import type { ComponentType } from "react";
import { BarChart3, Brain, Globe, History, Search, Users } from "lucide-react";

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

export const navConfig: NavGroup[] = [
  {
    label: "核心",
    items: [
      { key: "souls", label: "魂览", href: "/souls", icon: Users },
      { key: "possess", label: "讨论", href: "/possess", icon: Brain },
    ],
  },
  {
    label: "回顾",
    items: [
      { key: "sessions", label: "会话历史", href: "/sessions", icon: History },
      { key: "analytics", label: "仪表盘", href: "/analytics", icon: BarChart3 },
      { key: "knowledge", label: "知识库", href: "/knowledge", icon: Search },
    ],
  },
  {
    label: "工具",
    items: [
      { key: "searxng", label: "SearXNG 搜索", href: "/searxng", icon: Globe },
    ],
  },
];
