import Link from "next/link";
import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";

export function QuickActions() {
  return (
    <Link href="/possess" data-testid="new-possession-btn">
      <Button size="sm">
        <Plus className="h-4 w-4" />
      </Button>
    </Link>
  );
}
