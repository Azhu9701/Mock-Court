"use client";

import { useEffect, useState } from "react";
import { useParams, notFound } from "next/navigation";
import Link from "next/link";
import { fetchSessionDetail, type SessionDetail } from "@/lib/api";
import { ArrowLeft, User } from "lucide-react";
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
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    fetchSessionDetail(id).then(setDetail).catch(() => setError(true));
  }, [id]);

  if (error) return notFound();
  if (!detail) return <Skeleton className="h-96" />;

  const { session, messages } = detail;

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
        <SynthesisSection
          key={msg.id}
          messages={[{
            id: msg.id,
            content: msg.content,
            created_at: msg.created_at,
          }]}
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
                  messages={[{
                    id: answer.id,
                    content: answer.content,
                    created_at: answer.created_at,
                  }]}
                />
              )}
            </div>
          ))}
        </div>
      )}

      <FollowUpInput sessionId={id} />
    </div>
  );
}
