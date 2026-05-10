import { Suspense } from "react";
import { fetchSessions } from "@/lib/api";
import { SessionTimeline } from "@/components/session-timeline";
import { Skeleton } from "@/components/ui/skeleton";

export const dynamic = "force-dynamic";

export default function SessionsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">会话历史</h1>
        <p className="text-sm text-muted-foreground mt-1">浏览所有附体会话记录</p>
      </div>
      <Suspense fallback={<Skeleton className="h-96 rounded-xl" />}>
        <SessionsAsync />
      </Suspense>
    </div>
  );
}

async function SessionsAsync() {
  const sessions = await fetchSessions(200);
  return <SessionTimeline sessions={sessions} />;
}
