"use client";

import { Button } from "@/components/ui/button";

export default function SoulListError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-4">
      <p className="text-lg font-semibold">加载失败</p>
      <p className="text-sm text-muted-foreground">{error.message}</p>
      <Button onClick={reset} data-testid="retry-btn">
        重试
      </Button>
    </div>
  );
}
