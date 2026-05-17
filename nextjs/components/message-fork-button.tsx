"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { GitFork, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { forkSession } from "@/lib/api";

export function MessageForkButton({
  sessionId,
  messageSeq,
}: {
  sessionId: string;
  messageSeq: number;
}) {
  const router = useRouter();
  const [forking, setForking] = useState(false);

  const onFork = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setForking(true);
    try {
      const result = await forkSession(sessionId, messageSeq);
      router.push(`/sessions/${result.session_id}?fork=true`);
    } catch (e: unknown) {
      console.error("Fork failed:", e);
      setForking(false);
    }
  };

  return (
    <Button
      variant="ghost"
      size="sm"
      className="h-7 px-2 text-xs gap-1"
      onClick={onFork}
      disabled={forking}
      title="从这条消息重新出发"
    >
      {forking ? (
        <Loader2 className="h-3.5 w-3.5 animate-spin" />
      ) : (
        <GitFork className="h-3.5 w-3.5" />
      )}
      分叉
    </Button>
  );
}
