"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight } from "lucide-react";

const labels: Record<string, string> = {
  souls: "魂览",
  possess: "讨论",
  sessions: "会话历史",
  analytics: "仪表盘",
};

export function Breadcrumb() {
  const pathname = usePathname();
  const segments = pathname.split("/").filter(Boolean);

  if (segments.length === 0) return null;

  return (
    <nav data-testid="breadcrumb" aria-label="面包屑">
      <ol className="flex items-center gap-1 text-sm text-muted-foreground">
        <li>
          <Link href="/" className="hover:text-foreground transition-colors">
            首页
          </Link>
        </li>
        {segments.map((seg, i) => (
          <li key={i} className="flex items-center gap-1">
            <ChevronRight className="h-3 w-3" />
            <Link
              href={`/${segments.slice(0, i + 1).join("/")}`}
              className="hover:text-foreground transition-colors capitalize"
            >
              {labels[seg] || decodeURIComponent(seg)}
            </Link>
          </li>
        ))}
      </ol>
    </nav>
  );
}
