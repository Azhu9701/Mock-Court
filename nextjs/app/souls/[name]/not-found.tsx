import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function SoulNotFound() {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-4">
      <p className="text-4xl font-bold text-muted-foreground">404</p>
      <p className="text-lg">角色不存在</p>
      <p className="text-sm text-muted-foreground">
        该角色可能已被删除，或尚未添加
      </p>
      <Link href="/souls">
        <Button variant="outline" data-testid="back-to-list-btn">
          返回角色列表
        </Button>
      </Link>
    </div>
  );
}
