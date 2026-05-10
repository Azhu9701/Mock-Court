import { Brain, MessageCircle, Swords, GitBranch, GraduationCap, Hammer } from "lucide-react";

export type PossessionMode =
  | "single"
  | "conference"
  | "debate"
  | "relay"
  | "learn"
  | "practice_opening";

export interface ModeConfig {
  key: PossessionMode;
  label: string;
  description: string;
  icon: string;
  minSouls: number;
  maxSouls: number;
}

export const MODES: ModeConfig[] = [
  {
    key: "single",
    label: "单魂附体",
    description: "召唤一位思想家，进行一对一深度对话",
    icon: "message-circle",
    minSouls: 1,
    maxSouls: 1,
  },
  {
    key: "conference",
    label: "合议",
    description: "多魂并行思考，辩证综合得出更全面的答案",
    icon: "brain",
    minSouls: 2,
    maxSouls: 10,
  },
  {
    key: "debate",
    label: "辩论",
    description: "两位思想家围绕议题展开辩论，最终给出裁决",
    icon: "swords",
    minSouls: 2,
    maxSouls: 2,
  },
  {
    key: "relay",
    label: "接力",
    description: "魂链接力思考，每位基于前一位的输出继续深入",
    icon: "git-branch",
    minSouls: 2,
    maxSouls: 10,
  },
  {
    key: "learn",
    label: "学习",
    description: "请思想家作为学习伙伴，解释他的思考过程",
    icon: "graduation-cap",
    minSouls: 1,
    maxSouls: 1,
  },
  {
    key: "practice_opening",
    label: "实践开口",
    description: "四步实践循环：收集→消化→修订→行动",
    icon: "hammer",
    minSouls: 1,
    maxSouls: 10,
  },
];

export const MODE_LABELS: Record<PossessionMode, string> = {
  single: "单魂",
  conference: "合议",
  debate: "辩论",
  relay: "接力",
  learn: "学习",
  practice_opening: "实践开口",
};

export const MODE_LABELS_LONG: Record<PossessionMode, string> = {
  single: "单魂模式",
  conference: "合议模式",
  debate: "辩论模式",
  relay: "接力模式",
  learn: "学习模式",
  practice_opening: "实践开口模式",
};

export const MODE_COLORS_BG: Record<PossessionMode, string> = {
  single: "bg-blue-500",
  conference: "bg-purple-500",
  debate: "bg-orange-500",
  relay: "bg-green-500",
  learn: "bg-teal-500",
  practice_opening: "bg-red-500",
};

export const MODE_COLORS_TEXT: Record<PossessionMode, string> = {
  single: "text-blue-500",
  conference: "text-purple-500",
  debate: "text-orange-500",
  relay: "text-green-500",
  learn: "text-teal-500",
  practice_opening: "text-red-500",
};

export const MODE_COLORS_HEX: Record<PossessionMode, string> = {
  single: "#3b82f6",
  conference: "#8b5cf6",
  debate: "#f97316",
  relay: "#22c55e",
  learn: "#14b8a6",
  practice_opening: "#ef4444",
};

export function modeLabel(mode: string): string {
  return MODE_LABELS[mode as PossessionMode] || mode;
}

export function modeColorBg(mode: string): string {
  return MODE_COLORS_BG[mode as PossessionMode] || "bg-gray-400";
}

export const iconMap: Record<string, React.ComponentType<{ className?: string }>> = {
  "message-circle": MessageCircle,
  brain: Brain,
  swords: Swords,
  "git-branch": GitBranch,
  "graduation-cap": GraduationCap,
  hammer: Hammer,
};
