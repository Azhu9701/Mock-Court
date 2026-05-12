"use client";

import { useEffect, useState, useCallback } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { Trash2, Pencil, Check, X, RefreshCw } from "lucide-react";
import { cn } from "@/lib/utils";
import { fetchSessions, deleteSession, renameSession, type SessionSummary } from "@/lib/api";
import { modeLabel, MODE_COLORS_BG } from "@/config/possession-modes";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ConfirmButton } from "@/components/ui/confirm-button";

export const SESSIONS_UPDATED_EVENT = "aionui-sessions-updated";

export function triggerSessionsUpdate() {
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent(SESSIONS_UPDATED_EVENT));
  }
}

export function SidebarSessions() {
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState("");
  const [refreshing, setRefreshing] = useState(false);
  const [clientReady, setClientReady] = useState(false);
  const pathname = usePathname();
  const router = useRouter();

  const refreshSessions = useCallback(() => {
    setRefreshing(true);
    fetchSessions(10)
      .then(setSessions)
      .catch(() => {})
      .finally(() => {
        setRefreshing(false);
        setClientReady(true);
      });
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => {
      refreshSessions();
    }, 0);

    const handleSessionsUpdated = () => {
      refreshSessions();
    };
    window.addEventListener(SESSIONS_UPDATED_EVENT, handleSessionsUpdated);

    return () => {
      clearTimeout(timer);
      window.removeEventListener(SESSIONS_UPDATED_EVENT, handleSessionsUpdated);
    };
  }, [refreshSessions]);

  const handleDelete = async (sessionId: string) => {
    try {
      await deleteSession(sessionId);
      refreshSessions();
      triggerSessionsUpdate();
      if (pathname === `/sessions/${sessionId}`) {
        router.push("/sessions");
      }
    } catch (e) {
      console.error("Failed to delete session:", e);
    }
  };

  const handleRename = async (sessionId: string) => {
    if (!editingTitle.trim()) {
      setEditingId(null);
      return;
    }
    try {
      await renameSession(sessionId, editingTitle.trim());
      refreshSessions();
      triggerSessionsUpdate();
      setEditingId(null);
    } catch (e) {
      console.error("Failed to rename session:", e);
    }
  };

  if (!clientReady) {
    return (
      <div className="border-t pt-2 pb-4">
        <div className="flex items-center justify-between px-3 mb-1">
          <h3 className="text-xs font-semibold text-muted-foreground">
            最近对话
          </h3>
        </div>
        <div className="space-y-0.5 px-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-6 bg-muted/50 rounded-md animate-pulse" />
          ))}
        </div>
      </div>
    );
  }

  if (sessions.length === 0) return null;

  return (
    <div className="border-t pt-2 pb-4">
      <div className="flex items-center justify-between px-3 mb-1">
        <h3 className="text-xs font-semibold text-muted-foreground">
          最近对话
        </h3>
        <Button
          variant="ghost"
          size="icon"
          className="h-5 w-5"
          onClick={refreshSessions}
          disabled={refreshing}
          title="刷新会话列表"
        >
          <RefreshCw className={cn("h-3 w-3", refreshing && "animate-spin")} />
        </Button>
      </div>
      <div className="space-y-0.5">
        {sessions.slice(0, 8).map((s) => {
          const href = `/sessions/${s.id}`;
          const active = pathname === href || pathname === `/possess/${s.id}`;
          const isEditing = editingId === s.id;

          return (
            <div key={s.id} className="group relative">
              {isEditing ? (
                <div className="flex items-center gap-1 px-2 py-1">
                  <Input
                    value={editingTitle}
                    onChange={(e) => setEditingTitle(e.target.value)}
                    className="h-5 text-xs px-1.5"
                    autoFocus
                    onKeyDown={(e) => {
                      if (e.key === "Enter") handleRename(s.id);
                      if (e.key === "Escape") setEditingId(null);
                    }}
                  />
                  <Button size="icon" variant="ghost" className="h-5 w-5" onClick={() => handleRename(s.id)}>
                    <Check className="h-3 w-3" />
                  </Button>
                  <Button size="icon" variant="ghost" className="h-5 w-5" onClick={() => setEditingId(null)}>
                    <X className="h-3 w-3" />
                  </Button>
                </div>
              ) : (
                <div className={cn(
                  "flex items-center gap-1.5 rounded-md pl-2 pr-1 py-1 text-xs transition-colors",
                  active ? "bg-primary/10" : "hover:bg-muted"
                )}>
                  {/* 模式色点 */}
                  <div className={cn(
                    "w-1.5 h-1.5 rounded-full shrink-0",
                    (MODE_COLORS_BG as Record<string, string>)[s.mode] || "bg-gray-400"
                  )} />
                  <Link
                    href={href}
                    data-testid={`sidebar-session-${s.id}`}
                    className={cn(
                      "flex-1 min-w-0 truncate",
                      active
                        ? "text-primary font-medium"
                        : "text-muted-foreground hover:text-foreground"
                    )}
                  >
                    {s.title}
                  </Link>
                  <span className="text-[10px] text-muted-foreground/50 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                    {modeLabel(s.mode)}
                  </span>
                  <div className="flex items-center shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-5 w-5"
                      title="重命名"
                      onClick={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        setEditingTitle(s.title);
                        setEditingId(s.id);
                      }}
                    >
                      <Pencil className="h-3 w-3" />
                    </Button>
                    <ConfirmButton
                      icon={<Trash2 className="h-3 w-3 text-red-500" />}
                      confirmText="确认删除"
                      title="删除会话"
                      size="icon"
                      className="h-5 w-5"
                      onConfirm={async () => {
                        await handleDelete(s.id);
                      }}
                    />
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
