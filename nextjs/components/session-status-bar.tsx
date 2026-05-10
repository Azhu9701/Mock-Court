"use client";

import { Button } from "@/components/ui/button";
import { Wifi, WifiOff, CheckCircle, AlertCircle } from "lucide-react";

interface SessionStatusBarProps {
  status: "connecting" | "streaming" | "done" | "error";
  error: string | null;
  onReconnect: () => void;
}

export function SessionStatusBar({
  status,
  error,
  onReconnect,
}: SessionStatusBarProps) {
  return (
    <div
      className="flex items-center justify-between h-10 px-4 border-b bg-muted/50 text-sm"
      data-testid="session-status-bar"
    >
      <div className="flex items-center gap-2">
        {status === "connecting" && (
          <>
            <Wifi className="h-4 w-4 text-yellow-500 animate-pulse" />
            <span className="text-muted-foreground">连接中...</span>
          </>
        )}
        {status === "streaming" && (
          <>
            <Wifi className="h-4 w-4 text-green-500" />
            <span className="text-muted-foreground">接收中</span>
          </>
        )}
        {status === "done" && (
          <>
            <CheckCircle className="h-4 w-4 text-green-500" />
            <span className="text-muted-foreground">完成</span>
          </>
        )}
        {status === "error" && (
          <>
            <WifiOff className="h-4 w-4 text-red-500" />
            <span className="text-red-500">{error || "连接失败"}</span>
          </>
        )}
      </div>
      {status === "error" && (
        <Button
          variant="outline"
          size="sm"
          onClick={onReconnect}
          data-testid="reconnect-btn"
        >
          <AlertCircle className="mr-1 h-3 w-3" />
          重试
        </Button>
      )}
    </div>
  );
}
