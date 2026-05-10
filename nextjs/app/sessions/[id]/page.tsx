import { notFound } from "next/navigation";
import Link from "next/link";
import { fetchSessionDetail, type SessionDetail } from "@/lib/api";
import { ArrowLeft, User } from "lucide-react";
import { Button } from "@/components/ui/button";
import SessionActions from "@/components/session-actions";
import FollowUpInput from "@/components/follow-up-input";
import { SoulResponseCard } from "@/components/soul-response-card";
import { SynthesisSection } from "@/components/synthesis-section";
import { MODE_LABELS_LONG, modeColorBg } from "@/config/possession-modes";

export const dynamic = "force-dynamic";

export default async function SessionDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;

  let detail: SessionDetail;
  try { detail = await fetchSessionDetail(id); } catch { notFound(); }

  const { session, messages } = detail;

  // 分离用户消息和魂的响应
  const userMessages = messages.filter(m => m.role === "user");
  const soulMessages = messages.filter(m => (m.role === "assistant" || m.soul_name) && m.soul_name !== "知识卡片");
  const synthesisMessages = messages.filter(m => m.role === "synthesis");
  const systemMessages = messages.filter(m => m.role === "system" && !m.content.startsWith("📇"));

  // 按魂分组
  const soulResponses: Record<string, string> = {};
  soulMessages.forEach(msg => {
    const name = msg.soul_name || msg.role;
    if (!soulResponses[name]) {
      soulResponses[name] = "";
    }
    soulResponses[name] += (soulResponses[name] ? "\n\n" : "") + msg.content;
  });

  return (
    <div className="max-w-5xl mx-auto space-y-6">
      {/* 头部 */}
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

      {/* 用户消息 */}
      {userMessages.map((msg) => (
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
              <p className="text-sm leading-relaxed">{msg.content}</p>
            </div>
          </div>
        </div>
      ))}

      {/* 魂响应 - 并排显示 */}
      {Object.keys(soulResponses).length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {Object.entries(soulResponses).map(([name, content]) => (
            <SoulResponseCard
              key={name}
              name={name}
              content={content}
            />
          ))}
        </div>
      )}

      {/* 系统消息 */}
      {systemMessages.map((msg) => (
        <div key={msg.id} className="text-center py-2">
          <span className="text-xs text-muted-foreground">{msg.content}</span>
        </div>
      ))}

      {/* 辩证综合 */}
      {synthesisMessages.length > 0 && (
        <SynthesisSection messages={synthesisMessages} />
      )}

      {/* 追问输入 */}
      <FollowUpInput sessionId={id} />
    </div>
  );
}
