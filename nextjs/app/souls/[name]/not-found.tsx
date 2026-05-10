import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function SoulNotFound() {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-4">
      <p className="text-4xl font-bold text-muted-foreground">404</p>
      <p className="text-lg">魂不存在</p>
      <p className="text-sm text-muted-foreground">
        该魂可能已被散离，或从未入幡
      </p>
      <Link href="/souls">
        <Button variant="outline" data-testid="back-to-list-btn">
          返回魂览
        </Button>
      </Link>
    </div>
  );
}
