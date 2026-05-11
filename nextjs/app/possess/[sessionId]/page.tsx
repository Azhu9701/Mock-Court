"use client";

import { useState, use } from "react";
import { SessionRunner } from "@/components/session-runner";

export default function SessionPage({
  params,
  searchParams: searchParamsPromise,
}: {
  params: Promise<{ sessionId: string }>;
  searchParams: Promise<{ mode?: string }>;
}) {
  const { sessionId } = use(params);
  const { mode = "single" } = use(searchParamsPromise);
  const [sessionDone, setSessionDone] = useState(false);

  return (
    <div className="max-w-5xl mx-auto space-y-4" data-testid="session-page">
      <SessionRunner
        sessionId={sessionId}
        mode={mode}
        onDone={() => setSessionDone(true)}
        sessionDone={sessionDone}
      />
    </div>
  );
}
