"use client";

import { useState } from "react";
import { Zap, ChevronDown, ChevronUp, MessageSquare, ArrowRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CollisionEvent } from "@/hooks/use-websocket";

interface CollisionNotificationProps {
  collisions: CollisionEvent[];
}

export function CollisionNotification({ collisions }: CollisionNotificationProps) {
  const [expanded, setExpanded] = useState(true);
  const latestCollision = collisions[collisions.length - 1];

  return (
    <div className="border-t border-amber-200 dark:border-amber-800 bg-amber-50/50 dark:bg-amber-950/30">
      <div className="px-4 py-3">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 flex-1">
            <div className="p-1.5 bg-amber-100 dark:bg-amber-900 rounded-full">
              <Zap className="h-4 w-4 text-amber-600" />
            </div>
            <div className="flex flex-col">
              <span className="font-semibold text-amber-800 dark:text-amber-200 text-sm">
                ⚡ 碰撞！{collisions.length} 个交叉提问
              </span>
              {latestCollision && (
                <span className="text-xs text-amber-700 dark:text-amber-300">
                  {latestCollision.from} → {latestCollision.to}
                </span>
              )}
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setExpanded(!expanded)}
            className="text-amber-700 dark:text-amber-300 hover:bg-amber-100 dark:hover:bg-amber-900"
          >
            {expanded ? <ChevronUp className="h-4 w-4 mr-1" /> : <ChevronDown className="h-4 w-4 mr-1" />}
            {expanded ? "收起" : "查看详情"}
          </Button>
        </div>

        {expanded && (
          <div className="mt-3 space-y-2">
            {collisions.map((collision, index) => (
              <div
                key={index}
                className="bg-white dark:bg-gray-900 rounded-lg p-3 border border-amber-200 dark:border-amber-800 shadow-sm"
              >
                <div className="flex items-center gap-2 mb-2">
                  <MessageSquare className="h-4 w-4 text-amber-500" />
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-sm text-amber-700 dark:text-amber-300">
                      {collision.from}
                    </span>
                    <ArrowRight className="h-3 w-3 text-muted-foreground" />
                    <span className="font-medium text-sm text-amber-700 dark:text-amber-300">
                      {collision.to}
                    </span>
                  </div>
                </div>
                <p className="text-sm text-muted-foreground pl-6 italic">
                  "{collision.content}"
                </p>
                {collision.injected && (
                  <p className="text-xs text-green-600 dark:text-green-400 mt-2 pl-6">
                    ✓ 追问已注入 {collision.to} 面板
                  </p>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
