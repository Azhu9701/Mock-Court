"use client";

import { useState, useMemo } from "react";
import type { SoulMessage } from "@/hooks/use-websocket";
import { parseIsmismCode } from "@/config/soul-filter";
import { cn } from "@/lib/utils";
import { AlertCircle, CheckCircle2, Lightbulb, Target, Eye, ChevronDown, ChevronUp, GraduationCap } from "lucide-react";

const FIELD_LEVELS = ["未标定", "秩序主义", "表征主义", "反思主义", "批判实践"];
const ONTOLOGY_LEVELS = ["未标定", "物质实在", "道/结构", "主体/生命", "符号/虚无"];
const EPISTEMOLOGY_LEVELS = ["未标定", "经验实证", "理性演绎", "直觉体验", "辩证批判"];
const TELEOLOGY_LEVELS = ["未标定", "维持回归", "建构改良", "完成解放", "否定消解"];

interface LearnViewProps {
  messages: Record<string, SoulMessage>;
}

interface FeedbackSection {
  title: string;
  icon: React.ComponentType<{ className?: string }>;
  color: string;
  bgColor: string;
  borderColor: string;
  items: string[];
}

function IdeologyBadge({ label, value, levels }: { label: string; value: number; levels: string[] }) {
  const colorByValue: Record<number, string> = {
    1: "bg-slate-100 text-slate-700 dark:bg-slate-900 dark:text-slate-300",
    2: "bg-cyan-100 text-cyan-700 dark:bg-cyan-900 dark:text-cyan-300",
    3: "bg-violet-100 text-violet-700 dark:bg-violet-900 dark:text-violet-300",
    4: "bg-rose-100 text-rose-700 dark:bg-rose-900 dark:text-rose-300",
  };
  const colorClass = colorByValue[value] || "bg-gray-100 text-gray-700 dark:bg-gray-900 dark:text-gray-300";

  return (
    <div className="flex items-center gap-1.5">
      <span className="text-xs font-medium text-muted-foreground w-12 shrink-0">{label}</span>
      <span className={cn("text-xs px-2 py-0.5 rounded-full font-medium", colorClass)}>
        {levels[value] || "未标定"}
      </span>
    </div>
  );
}

function SoulIdeologyPanel({ name, ismismCode }: { name: string; ismismCode: string }) {
  const [expanded, setExpanded] = useState(false);
  const code = parseIsmismCode(ismismCode);

  return (
    <div className="rounded-xl border bg-background shadow-sm">
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between p-3 hover:bg-muted/30 transition-colors"
      >
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-full bg-gradient-to-br from-teal-400 to-cyan-500 flex items-center justify-center text-white text-sm font-bold shadow-sm">
            {name.charAt(0)}
          </div>
          <div className="text-left">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-sm">{name}</span>
              <span className="text-xs text-muted-foreground font-mono">{ismismCode}</span>
            </div>
            <div className="text-xs text-muted-foreground">学习伙伴 · 意识形态坐标</div>
          </div>
        </div>
        {expanded ? (
          <ChevronUp className="h-4 w-4 text-muted-foreground" />
        ) : (
          <ChevronDown className="h-4 w-4 text-muted-foreground" />
        )}
      </button>

      {expanded && code && (
        <div className="border-t px-3 py-3 space-y-1.5 bg-muted/10">
          <IdeologyBadge label="领域" value={code.field} levels={FIELD_LEVELS} />
          <IdeologyBadge label="本体论" value={code.ontology} levels={ONTOLOGY_LEVELS} />
          <IdeologyBadge label="认识论" value={code.epistemology} levels={EPISTEMOLOGY_LEVELS} />
          <IdeologyBadge label="目的论" value={code.teleology} levels={TELEOLOGY_LEVELS} />
        </div>
      )}
    </div>
  );
}

function FeedbackCard({ section }: { section: FeedbackSection }) {
  const Icon = section.icon;
  return (
    <div className={cn("rounded-xl border p-4", section.bgColor, section.borderColor)}>
      <div className="flex items-center gap-2 mb-3">
        <Icon className={cn("h-4 w-4 shrink-0", section.color)} />
        <h3 className={cn("text-sm font-semibold", section.color)}>{section.title}</h3>
      </div>
      <ul className="space-y-2">
        {section.items.map((item, i) => (
          <li key={i} className="text-sm text-foreground/90 leading-relaxed flex items-start gap-2">
            <span className="text-muted-foreground mt-1">•</span>
            <span>{item}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function parseFeedback(content: string): FeedbackSection[] {
  const sections: FeedbackSection[] = [];
  const remaining: string[] = [];

  const sectionPatterns = [
    {
      keywords: ["抓住", "抓准", "准确", "正确"],
      section: {
        title: "抓准的核心",
        icon: CheckCircle2,
        color: "text-emerald-600 dark:text-emerald-400",
        bgColor: "bg-emerald-50 dark:bg-emerald-950/20",
        borderColor: "border-emerald-200 dark:border-emerald-800",
        items: [] as string[],
      },
    },
    {
      keywords: ["逻辑", "断裂", "漏洞", "不足"],
      section: {
        title: "逻辑漏洞",
        icon: AlertCircle,
        color: "text-amber-600 dark:text-amber-400",
        bgColor: "bg-amber-50 dark:bg-amber-950/20",
        borderColor: "border-amber-200 dark:border-amber-800",
        items: [] as string[],
      },
    },
    {
      keywords: ["维度", "视角", "忽略", "盲区", "盲点"],
      section: {
        title: "缺失维度",
        icon: Eye,
        color: "text-red-600 dark:text-red-400",
        bgColor: "bg-red-50 dark:bg-red-950/20",
        borderColor: "border-red-200 dark:border-red-800",
        items: [] as string[],
      },
    },
    {
      keywords: ["建议", "改进", "加强", "补充"],
      section: {
        title: "改进建议",
        icon: Lightbulb,
        color: "text-blue-600 dark:text-blue-400",
        bgColor: "bg-blue-50 dark:bg-blue-950/20",
        borderColor: "border-blue-200 dark:border-blue-800",
        items: [] as string[],
      },
    },
  ];

  const lines = content.split("\n").map((l) => l.trim()).filter((l) => l.length > 0);

  for (const line of lines) {
    const cleanLine = line.replace(/^[\d\-\*\•\.\)\(]+\s*/, "");
    if (!cleanLine) continue;

    let matched = false;
    for (const pattern of sectionPatterns) {
      if (pattern.keywords.some((kw) => cleanLine.includes(kw))) {
        pattern.section.items.push(cleanLine);
        matched = true;
        break;
      }
    }

    if (!matched && cleanLine.length > 10) {
      remaining.push(cleanLine);
    }
  }

  for (const pattern of sectionPatterns) {
    if (pattern.section.items.length > 0) {
      sections.push(pattern.section);
    }
  }

  if (remaining.length > 0) {
    sections.unshift({
      title: "总体评价",
      icon: Target,
      color: "text-purple-600 dark:text-purple-400",
      bgColor: "bg-purple-50 dark:bg-purple-950/20",
      borderColor: "border-purple-200 dark:border-purple-800",
      items: remaining,
    });
  }

  return sections;
}

export function LearnView({ messages }: LearnViewProps) {
  const entries = Object.values(messages);
  const soulMessage = entries[0];

  const feedbackSections = useMemo(() => {
    if (!soulMessage?.content) return [];
    return parseFeedback(soulMessage.content);
  }, [soulMessage]);

  const ismismCode = soulMessage?.ismismCode || "0-0-0-0";
  const soulName = soulMessage?.soulName || "思想家";

  if (!soulMessage) {
    return (
      <div className="max-w-3xl mx-auto py-10 text-center text-muted-foreground">
        <GraduationCap className="h-12 w-12 mx-auto mb-4 text-teal-400" />
        <p>等待学习伙伴的回应...</p>
      </div>
    );
  }

  return (
    <div data-testid="learn-view" className="max-w-4xl mx-auto space-y-6">
      <div className="text-center space-y-1 py-2">
        <h2 className="text-lg font-semibold bg-gradient-to-r from-teal-500 to-cyan-500 bg-clip-text text-transparent">
          学习模式 · 论证训练
        </h2>
        <p className="text-xs text-muted-foreground">
          你的学习伙伴会从他的意识形态立场出发，对你的论证进行结构性反馈
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-[1fr,1.2fr]">
        <div className="space-y-4">
          <SoulIdeologyPanel name={soulName} ismismCode={ismismCode} />

          {feedbackSections.length === 0 && !soulMessage.isStreaming && (
            <div className="rounded-xl border border-dashed border-muted-foreground/30 p-6 text-center space-y-2">
              <div className="text-4xl">📝</div>
              <p className="text-sm text-muted-foreground">思想家正在评估你的论证...</p>
            </div>
          )}
        </div>

        <div className="space-y-3">
          {feedbackSections.length > 0 ? (
            feedbackSections.map((section, i) => (
              <FeedbackCard key={i} section={section} />
            ))
          ) : (
            <div className="rounded-xl border bg-background p-4 shadow-sm">
              <div className="flex items-center gap-2 mb-3">
                <GraduationCap className="h-4 w-4 text-teal-500" />
                <h3 className="text-sm font-semibold text-teal-600 dark:text-teal-400">
                  学习伙伴回应
                </h3>
                {soulMessage.isStreaming && (
                  <span className="text-xs text-muted-foreground animate-pulse">生成中...</span>
                )}
              </div>
              <div className="text-sm text-foreground/90 leading-relaxed whitespace-pre-wrap">
                {soulMessage.content}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
