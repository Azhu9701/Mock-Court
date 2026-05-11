"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { SessionRunner } from "@/components/session-runner";
import { SessionContextHeader, type MatchedSoulInfo } from "@/components/session-context-header";
import { SoulResponseCard } from "@/components/soul-response-card";
import { SynthesisSection } from "@/components/synthesis-section";
import { BreadcrumbSetter } from "@/components/breadcrumb-setter";
import SessionActions from "@/components/session-actions";
import FollowUpInput from "@/components/follow-up-input";
import { fetchSessionDetail, fetchSoul } from "@/lib/api";
import { popPendingSession } from "@/lib/pending-session";
import { ArrowLeft, User, Brain, Sparkles, ShieldCheck, Zap, Play, Loader2, CheckCircle2, ChevronDown, ChevronUp } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { MODE_LABELS_LONG, modeColorBg } from "@/config/possession-modes";

const FLOW_PHASES: { key: string; icon: React.ComponentType<{ className?: string }>; label: string; desc: string }[] = [
  { key: "classifying", icon: Brain, label: "入口分流", desc: "分析任务类型" },
  { key: "matching", icon: Sparkles, label: "匹配魂", desc: "智能匹配思想者" },
  { key: "reviewing", icon: ShieldCheck, label: "审查", desc: "审查魂组合" },
  { key: "adjusting", icon: Zap, label: "调整", desc: "优化魂搭配" },
  { key: "starting", icon: Play, label: "启动", desc: "启动讨论会话" },
];

export default function SessionDetailView({ id }: { id: string }) {
  const [detail, setDetail] = useState<Awaited<ReturnType<typeof fetchSessionDetail>> | null>(null);
  const [mode, setMode] = useState("single");
  const [matchedSouls, setMatchedSouls] = useState<MatchedSoulInfo[]>([]);
  const [review, setReview] = useState<{ verdict: string; checks: string[]; notes: string; reviewer: string } | null>(null);
  const [phases, setPhases] = useState<string[]>([]);
  const [flowExpanded, setFlowExpanded] = useState(true);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      const pending = popPendingSession();
      if (pending && pending.sessionId === id) {
        if (!cancelled) {
          setMode(pending.mode);
          setMatchedSouls(pending.matchedSouls.map((s) => ({
            name: s.name,
            field: s.field || "",
            ismism_code: s.ismism_code || "",
            rationale: s.rationale || "",
          })));
          if (pending.review) setReview(pending.review);
          if (pending.phases?.length) setPhases(pending.phases);
        }
      }

      try {
        const d = await fetchSessionDetail(id);
        if (cancelled) return;
        setDetail(d);
        setMode(d.session.mode);

        const soulNames = Array.from(
          new Set(d.messages.filter((m) => m.soul_name && m.role !== "system").map((m) => m.soul_name!))
        );
        const souls: MatchedSoulInfo[] = [];
        for (const name of soulNames) {
          try {
            const profile = await fetchSoul(name);
            souls.push({ name, field: profile.field || "", ismism_code: profile.ismism_code || "", rationale: profile.self_declare || "" });
          } catch {
            souls.push({ name, field: "", ismism_code: "", rationale: "" });
          }
        }
        if (!cancelled) setMatchedSouls(souls);
      } catch {} finally {
        if (!cancelled) setLoading(false);
      }
    }
    load();
    return () => { cancelled = true; };
  }, [id]);

  if (loading || !detail) {
    return (
      <div className="max-w-5xl mx-auto space-y-4 animate-pulse">
        <div className="h-20 bg-muted rounded-xl" />
        <div className="h-96 bg-muted rounded-xl" />
      </div>
    );
  }

  const { session, messages } = detail;
  const isActive = session.status === "active" || session.status === "running";

  if (isActive) {
    const completedPhases = new Set(phases);
    const totalFlowPhases = FLOW_PHASES.length;

    return (
      <div className="max-w-5xl mx-auto space-y-4">
        <BreadcrumbSetter label={session.title} />
        <SessionContextHeader task={session.title} mode={mode} matchedSouls={matchedSouls} review={review} />

        {/* 5 步流程进度条 — 仅在从 /possess 刚跳转过来时显示 */}
        {phases.length > 0 && (
          <div className="rounded-xl border bg-background p-6 shadow-sm space-y-4 animate-in fade-in duration-300">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold flex items-center gap-2">
                <CheckCircle2 className="h-5 w-5 text-emerald-500" />
                讨论流程
              </h2>
              <Button variant="ghost" size="sm" onClick={() => setFlowExpanded(!flowExpanded)}>
                {flowExpanded ? <ChevronUp className="h-4 w-4 mr-1" /> : <ChevronDown className="h-4 w-4 mr-1" />}
                {flowExpanded ? "收起" : "展开"}
              </Button>
            </div>

            {flowExpanded && (
              <>
                <div className="mb-1">
                  <div className="flex items-center justify-between text-xs text-muted-foreground mb-1.5">
                    <span>{completedPhases.size}/{totalFlowPhases} 步骤</span>
                    <span className="text-emerald-600 font-medium">全部完成</span>
                  </div>
                  <div className="h-2 bg-muted rounded-full overflow-hidden">
                    <div className="h-full bg-gradient-to-r from-primary to-emerald-500 rounded-full w-full" />
                  </div>
                </div>

                <div className="flex items-center justify-center gap-0 overflow-x-auto">
                  {FLOW_PHASES.map((p, i) => {
                    const Icon = p.icon;
                    const isDone = completedPhases.has(p.key);
                    return (
                      <div key={p.key} className="flex items-center">
                        <div className="flex flex-col items-center gap-1.5">
                          <div className={cn(
                            "flex items-center justify-center w-10 h-10 rounded-full transition-all duration-300",
                            isDone ? "bg-emerald-500 text-white shadow-sm" : "bg-muted text-muted-foreground/40"
                          )}>
                            {isDone ? <CheckCircle2 className="h-5 w-5" /> : <Icon className="h-4 w-4" />}
                          </div>
                          <span className={cn(
                            "text-[10px] leading-tight text-center max-w-[60px]",
                            isDone ? "text-emerald-600 font-medium" : "text-muted-foreground/40"
                          )}>
                            {p.label}
                          </span>
                        </div>
                        {i < FLOW_PHASES.length - 1 && (
                          <div className={cn(
                            "h-0.5 w-8 sm:w-12 mx-0.5 rounded-full transition-colors duration-500",
                            isDone ? "bg-emerald-400" : "bg-muted"
                          )} />
                        )}
                      </div>
                    );
                  })}
                </div>

                {/* Matched souls summary */}
                {matchedSouls.length > 0 && (
                  <div className="mt-4 space-y-4 text-sm border-t pt-4">
                    <div>
                      <h4 className="font-medium text-muted-foreground mb-3 flex items-center gap-2">
                        <Sparkles className="h-4 w-4" />匹配魂
                      </h4>
                      <div className="grid gap-3">
                        {matchedSouls.map((s) => (
                          <div key={s.name} className="rounded-lg border p-3 bg-background transition-all hover:shadow-sm">
                            <div className="flex items-center gap-2 flex-wrap">
                              <span className="font-semibold text-base">{s.name}</span>
                              <span className="text-xs bg-muted px-2 py-0.5 rounded">{s.field}</span>
                              {s.ismism_code && <span className="text-xs text-muted-foreground font-mono">{s.ismism_code}</span>}
                            </div>
                            <p className="text-muted-foreground mt-2 text-sm leading-relaxed">{s.rationale}</p>
                          </div>
                        ))}
                      </div>
                    </div>

                    {review?.reviewer && (
                      <div>
                        <h4 className="font-medium text-muted-foreground mb-3 flex items-center gap-2">
                          <ShieldCheck className="h-4 w-4" />审查 · {review.reviewer}
                        </h4>
                        <div className={cn("rounded-lg border p-3",
                          review.verdict === "pass" ? "border-green-200 bg-green-50 dark:bg-green-950/20" :
                          review.verdict === "conditional" ? "border-yellow-200 bg-yellow-50 dark:bg-yellow-950/20" :
                          "border-red-200 bg-red-50 dark:bg-red-950/20"
                        )}>
                          <div className="font-medium mb-2">裁决: {
                            review.verdict === "pass" ? "✅ 通过" : review.verdict === "conditional" ? "⚠️ 条件通过" : "❌ 拒绝"
                          }</div>
                          <ul className="space-y-1">
                            {review.checks.map((c, i) => (
                              <li key={i} className="text-sm flex items-start gap-2"><span>→</span><span>{c}</span></li>
                            ))}
                          </ul>
                          {review.notes && <p className="text-sm mt-2 italic text-muted-foreground border-t pt-2">📝 {review.notes}</p>}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </>
            )}
          </div>
        )}

        <SessionRunner sessionId={id} mode={mode} matchedSouls={matchedSouls} />
      </div>
    );
  }

  // ── History view (completed/archived) ──
  const sorted = [...messages].sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime());
  const userMsgs = sorted.filter((m) => m.role === "user");
  const soulMsgs = sorted.filter((m) => (m.role === "assistant" || m.role === "soul") && m.soul_name && m.soul_name !== "知识卡片");
  const synthMsgs = sorted.filter((m) => m.role === "synthesis");
  const sysMsgs = sorted.filter((m) => m.role === "system");

  const soulResponses: Record<string, string> = {};
  for (const m of soulMsgs) {
    const name = m.soul_name!;
    soulResponses[name] = (soulResponses[name] ? soulResponses[name] + "\n\n" : "") + m.content;
  }

  const firstSynth = synthMsgs[0];
  const initUserMsgs = firstSynth ? userMsgs.filter((m) => new Date(m.created_at).getTime() < new Date(firstSynth.created_at).getTime()) : userMsgs;
  const followUserMsgs = firstSynth ? userMsgs.filter((m) => new Date(m.created_at).getTime() > new Date(firstSynth.created_at).getTime()) : [];
  const followPairs: { question: typeof userMsgs[number]; answer: typeof synthMsgs[number] | null }[] = [];
  for (const q of followUserMsgs) {
    const qTime = new Date(q.created_at).getTime();
    const answer = synthMsgs.find((s) => new Date(s.created_at).getTime() > qTime);
    followPairs.push({ question: q, answer: answer || null });
  }
  const followSynthIds = new Set(followPairs.filter((p) => p.answer).map((p) => p.answer!.id));
  const initSynths = synthMsgs.filter((s) => !followSynthIds.has(s.id));

  return (
    <div className="max-w-5xl mx-auto space-y-6">
      <BreadcrumbSetter label={session.title} />
      <div className="flex items-center gap-3">
        <Link href="/sessions">
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <ArrowLeft className="h-4 w-4" />
          </Button>
        </Link>
        <div className="flex-1 min-w-0">
          <h1 className="text-xl font-bold truncate flex items-center gap-2">
            {session.title}
            <span className={`w-2 h-2 rounded-full ${modeColorBg(session.mode)}`} />
          </h1>
          <p className="text-sm text-muted-foreground flex items-center gap-2">
            <span>{(MODE_LABELS_LONG as Record<string, string>)[session.mode] || session.mode}</span>
            <span>·</span>
            <span>{new Date(session.created_at).toLocaleString("zh-CN")}</span>
          </p>
        </div>
        <SessionActions sessionId={id} title={session.title} />
      </div>

      {initUserMsgs.map((msg) => (
        <div key={msg.id} className="flex gap-3 flex-row-reverse">
          <div className="shrink-0">
            <div className="w-9 h-9 rounded-full bg-primary/10 flex items-center justify-center">
              <User className="h-4 w-4 text-primary" />
            </div>
          </div>
          <div className="max-w-[70%]">
            <div className="flex items-center gap-2 mb-1 flex-row-reverse">
              <span className="text-xs font-semibold text-muted-foreground">用户</span>
              <span className="text-xs text-muted-foreground">
                {new Date(msg.created_at).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}
              </span>
            </div>
            <div className="rounded-xl p-4 bg-primary text-primary-foreground">
              <p className="text-sm leading-relaxed whitespace-pre-wrap">{msg.content}</p>
            </div>
          </div>
        </div>
      ))}

      {Object.keys(soulResponses).length > 0 && (
        <div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {Object.entries(soulResponses).map(([name, content]) => (
              <SoulResponseCard key={name} name={name} content={content} />
            ))}
          </div>
        </div>
      )}

      {sysMsgs.map((msg) => (
        <div key={msg.id} className="text-center py-2">
          <span className="text-xs text-muted-foreground">{msg.content}</span>
        </div>
      ))}

      {initSynths.map((msg) => (
        <SynthesisSection key={msg.id} messages={[{ id: msg.id, content: msg.content, created_at: msg.created_at }]} />
      ))}

      {followPairs.length > 0 && (
        <div className="space-y-6 border-t pt-6">
          <h3 className="text-sm font-semibold text-muted-foreground">追问记录</h3>
          {followPairs.map(({ question, answer }) => (
            <div key={question.id} className="space-y-4">
              <div className="flex gap-3 flex-row-reverse">
                <div className="shrink-0">
                  <div className="w-9 h-9 rounded-full bg-primary/10 flex items-center justify-center">
                    <User className="h-4 w-4 text-primary" />
                  </div>
                </div>
                <div className="max-w-[70%]">
                  <div className="flex items-center gap-2 mb-1 flex-row-reverse">
                    <span className="text-xs font-semibold text-muted-foreground">追问</span>
                    <span className="text-xs text-muted-foreground">
                      {new Date(question.created_at).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })}
                    </span>
                  </div>
                  <div className="rounded-xl p-4 bg-primary/5 border border-primary/10">
                    <p className="text-sm leading-relaxed whitespace-pre-wrap">{question.content}</p>
                  </div>
                </div>
              </div>
              {answer && <SynthesisSection messages={[{ id: answer.id, content: answer.content, created_at: answer.created_at }]} />}
            </div>
          ))}
        </div>
      )}

      <FollowUpInput sessionId={id} />
    </div>
  );
}
