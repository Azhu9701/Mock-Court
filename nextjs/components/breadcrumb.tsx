"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight } from "lucide-react";
import { useBreadcrumb } from "@/contexts/breadcrumb-context";

const labels: Record<string, string> = {
  souls: "魂览",
  possess: "讨论",
  sessions: "会话历史",
  analytics: "蛇皮统计",
};

export function Breadcrumb() {
  const pathname = usePathname();
  const { lastLabel } = useBreadcrumb();
  const segments = pathname.split("/").filter(Boolean);

  if (segments.length === 0) return null;

  const isLast = (i: number) => i === segments.length - 1;

  return (
    <nav data-testid="breadcrumb" aria-label="面包屑">
      <ol className="flex items-center gap-1 text-sm text-muted-foreground">
        <li>
          <Link href="/" className="hover:text-foreground transition-colors">
            首页
          </Link>
        </li>
        {segments.map((seg, i) => {
          const rawLabel = isLast(i) && lastLabel
            ? lastLabel
            : labels[seg] || decodeURIComponent(seg);
          const displayLabel = rawLabel.length > 30
            ? rawLabel.slice(0, 30) + "…"
            : rawLabel;

          return (
            <li key={i} className="flex items-center gap-1">
              <ChevronRight className="h-3 w-3" />
              <Link
                href={`/${segments.slice(0, i + 1).join("/")}`}
                className="hover:text-foreground transition-colors capitalize"
              >
                {displayLabel}
              </Link>
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
