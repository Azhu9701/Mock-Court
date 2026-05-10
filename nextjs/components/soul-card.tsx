import Link from "next/link";
import type { SoulListEntry } from "@/lib/api";
import { Users } from "lucide-react";

export function SoulCard({ soul }: { soul: SoulListEntry }) {
  return (
    <Link
      href={`/souls/${encodeURIComponent(soul.name)}`}
      data-testid={`soul-card-${soul.name}`}
      className="group flex flex-col gap-3 rounded-lg border bg-background p-4 transition-all hover:-translate-y-0.5 hover:shadow-md hover:border-primary/30"
    >
      <div className="flex items-start justify-between">
        <span className="text-xs text-muted-foreground font-mono">
          {soul.ismism_code}
        </span>
      </div>
      <h3 className="text-lg font-semibold truncate">{soul.name}</h3>
      <p className="text-sm text-muted-foreground">{soul.field}</p>
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span className="flex items-center gap-1">
          <Users className="h-3 w-3" />
          {soul.summon_count}
        </span>
        {soul.tags.length > 0 && (
          <span className="truncate max-w-[60%] text-right">
            {soul.tags.slice(0, 3).join(", ")}
          </span>
        )}
      </div>
    </Link>
  );
}
