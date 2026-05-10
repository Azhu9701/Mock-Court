"use client";

import { useState, use } from "react";
import { useRouter } from "next/navigation";
import { SessionRunner } from "@/components/session-runner";
import { PostSessionReview } from "@/components/post-session-review";

export default function SessionPage({
  params,
  searchParams: searchParamsPromise,
}: {
  params: Promise<{ sessionId: string }>;
  searchParams: Promise<{ mode?: string }>;
}) {
  const { sessionId } = use(params);
  const { mode = "single" } = use(searchParamsPromise);
  const router = useRouter();
  const [showReview, setShowReview] = useState(false);
  const [sessionDone, setSessionDone] = useState(false);

  if (showReview) {
    return (
      <div className="flex flex-col flex-1 -m-4 lg:-m-8">
        <PostSessionReview
          sessionId={sessionId}
          onComplete={() => router.push("/possess")}
        />
      </div>
    );
  }

  return (
    <div className="flex flex-col flex-1 -m-4 lg:-m-8" data-testid="session-page">
      <SessionRunner
        sessionId={sessionId}
        mode={mode}
        onDone={() => setSessionDone(true)}
        onReview={() => setShowReview(true)}
        sessionDone={sessionDone}
      />
    </div>
  );
}
