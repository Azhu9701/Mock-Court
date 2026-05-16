"use client";

import { useEffect, useState } from "react";
import { useParams, notFound } from "next/navigation";
import Link from "next/link";
import {
  fetchSessionDetail,
  fetchSessionDigest,
  triggerDistill,
  obsEmoji,
  type SessionDetail,
  type SessionDigest,
} from "@/lib/api";
import { SESSIONS_UPDATED_EVENT } from "@/components/sidebar-sessions";
import {
  ArrowLeft,
  User,
  ChevronDown,
  ChevronUp,
  Sparkles,
  RefreshCw,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import SessionActions from "@/components/session-actions";
import FollowUpInput from "@/components/follow-up-input";
import { SoulResponseCard } from "@/components/soul-response-card";
import { SynthesisSection } from "@/components/synthesis-section";
import { BreadcrumbSetter } from "@/components/breadcrumb-setter";
import { MODE_LABELS_LONG, modeColorBg } from "@/config/possession-modes";
import { Skeleton } from "@/components/ui/skeleton";

export default function SessionDetailPage() {
  const params = useParams<{ id: string }>();
  const id = params.id;

  // Layer 1: lightweight digest (5-10 observations, ~5-10KB)
  const [digest, setDigest] = useState<SessionDigest | null>(null);
  const [digestError, setDigestError] = useState(false);

  // Layer 2: full conversation (loaded on user demand)
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [expanded, setExpanded] = useState(false);

  const [distilling, setDistilling] = useState(false);

  // Fetch digest on mount
  useEffect(() => {
    fetchSessionDigest(id).then(setDigest).catch(() => setDigestError(true));
  }, [id]);

  // Refresh digest when WS observations_ready dispatches SESSIONS_UPDATED_EVENT
  useEffect(() => {
    const handle = () => {
      fetchSessionDigest(id).then(setDigest).catch(() => {});
    };
    window.addEventListener(SESSIONS_UPDATED_EVENT, handle);
    return () => window.removeEventListener(SESSIONS_UPDATED_EVENT, handle);
  }, [id]);

  // Fetch full detail when user expands
  useEffect(() => {
    if (expanded && !detail) {
      fetchSessionDetail(id).then(setDetail).catch(() => {});
    }
  }, [expanded, id, detail]);

  const handleDistill = async () => {
    setDistilling(true);
    try {
      await triggerDistill(id);
      // distill is async; poll digest a few times
      setTimeout(() => {
        fetchSessionDigest(id).then(setDigest).catch(() => {});
        setDistilling(false);
      }, 3000);
    } catch {
      setDistilling(false);
    }
  };

  if (digestError) return notFound();
  if (!digest) return <Skeleton className="h-96" />;

  return (
    <div className="max-w-5xl mx-auto space-y-6">
      <BreadcrumbSetter label={digest.title} />

      {/* Header */}
      <div className="flex items-center gap-3">
        <Link href="/sessions">
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <ArrowLeft className="h-4 w-4" />
          </Button>
        </Link>
        <div className="flex-1 min-w-0">
          <h1 className="text-xl font-bold truncate flex items-center gap-2">
            {digest.title}
            <span className={`w-2 h-2 rounded-full ${modeColorBg(digest.mode)}`} />
          </h1>
          <p className="text-sm text-muted-foreground flex items-center gap-2">
            <span>{(MODE_LABELS_LONG as Record<string, string>)[digest.mode] || digest.mode}</span>
            <span>·</span>
            <span>{new Date(digest.created_at).toLocaleString("zh-CN")}</span>
          </p>
        </div>
        <SessionActions sessionId={id} title={digest.title} />
      </div>

      {/* Layer 1: digest section */}
      {digest.observations.length > 0 ? (
        <div className="rounded-xl border bg-gradient-to-br from-blue-50/50 to-purple-50/30 dark:from-blue-950/20 dark:to-purple-950/10 p-5 space-y-4">
          <div className="flex items-center justify-between flex-wrap gap-2">
            <h3 className="text-sm font-semibold flex items-center gap-2">
              <Sparkles className="h-4 w-4 text-blue-500" />
              本次会话精华 · {digest.observations.length} 条 observation
            </h3>
            <span className="text-xs text-muted-foreground font-mono">
              省 {(digest.savings_ratio * 100).toFixed(0)}% tokens ·{" "}
              {digest.total_read_tokens.toLocaleString()}/{digest.total_work_tokens.toLocaleString()}
            </span>
          </div>

          {digest.summary && (
            <div className="text-sm text-muted-foreground italic border-l-2 border-blue-300 dark:border-blue-700 pl-3">
              “{digest.summary}”
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {digest.observations.map((o) => (
              <div
                key={o.id}
                className="rounded-lg border bg-background/80 p-3 space-y-1.5 hover:shadow-sm transition-shadow"
              >
                <div className="flex items-start gap-2">
                  <span className="text-base shrink-0 leading-none mt-0.5">{obsEmoji(o.obs_type)}</span>
                  <div className="flex-1 min-w-0">
                    <h4 className="text-sm font-semibold leading-tight">{o.title}</h4>
                    {o.soul_name && (
                      <span className="text-[10px] text-muted-foreground">— {o.soul_name}</span>
                    )}
                  </div>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed pl-7 whitespace-pre-wrap">
                  {o.content}
                </p>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <div className="rounded-xl border border-dashed bg-muted/30 p-6 text-center space-y-3">
          <p className="text-sm text-muted-foreground">该会话尚未压缩为 observation</p>
          <Button onClick={handleDistill} disabled={distilling} variant="outline" size="sm">
            <RefreshCw className={`h-4 w-4 mr-2 ${distilling ? "animate-spin" : ""}`} />
            {distilling ? "压缩中..." : "立即压缩"}
          </Button>
        </div>
      )}

      {/* Layer 2 toggle */}
      <div className="flex justify-center">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setExpanded(!expanded)}
          className="text-muted-foreground hover:text-foreground"
        >
          {expanded ? (
            <ChevronUp className="h-4 w-4 mr-1" />
          ) : (
            <ChevronDown className="h-4 w-4 mr-1" />
          )}
          {expanded ? "收起完整对话" : "展开完整对话"}
        </Button>
      </div>

      {/* Layer 2: full conversation, lazily loaded */}
      {expanded && (detail ? <FullConversation detail={detail} /> : <Skeleton className="h-96" />)}

      <FollowUpInput sessionId={id} />
    </div>
  );
}

function FullConversation({ detail }: { detail: SessionDetail }) {
  const { messages } = detail;
  const sorted = [...messages].sort(
    (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
  );

  const userMsgs = sorted.filter((m) => m.role === "user");
  const soulMsgs = sorted.filter(
    (m) => (m.role === "assistant" || m.role === "soul") && m.soul_name && m.soul_name !== "知识卡片"
  );
  const synthMsgs = sorted.filter((m) => m.role === "synthesis");
  const sysMsgs = sorted.filter((m) => m.role === "system" && !m.content.startsWith("[REVIEW]"));

  const soulResponses: Record<string, string> = {};
  for (const m of soulMsgs) {
    const name = m.soul_name!;
    soulResponses[name] = (soulResponses[name] ? soulResponses[name] + "\n\n" : "") + m.content;
  }

  const firstSynth = synthMsgs[0];
  const initUserMsgs = firstSynth
    ? userMsgs.filter((m) => new Date(m.created_at).getTime() < new Date(firstSynth.created_at).getTime())
    : userMsgs;
  const followUserMsgs = firstSynth
    ? userMsgs.filter((m) => new Date(m.created_at).getTime() > new Date(firstSynth.created_at).getTime())
    : [];

  const followPairs: { question: typeof userMsgs[number]; answer: typeof synthMsgs[number] | null }[] = [];
  for (const q of followUserMsgs) {
    const qTime = new Date(q.created_at).getTime();
    const answer = synthMsgs.find((s) => new Date(s.created_at).getTime() > qTime);
    followPairs.push({ question: q, answer: answer || null });
  }

  const followSynthIds = new Set(followPairs.filter((p) => p.answer).map((p) => p.answer!.id));
  const initSynths = synthMsgs.filter((s) => !followSynthIds.has(s.id));

  return (
    <div className="space-y-6 border-t pt-6">
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
        <SynthesisSection
          key={msg.id}
          messages={[{ id: msg.id, content: msg.content, created_at: msg.created_at }]}
        />
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
              {answer && (
                <SynthesisSection
                  messages={[{ id: answer.id, content: answer.content, created_at: answer.created_at }]}
                />
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
