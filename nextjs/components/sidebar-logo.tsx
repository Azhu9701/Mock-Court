import Link from "next/link";
import { Flag } from "lucide-react";

export function SidebarLogo() {
  return (
    <Link
      href="/"
      className="flex h-14 items-center gap-2 border-b px-4 shrink-0"
      data-testid="sidebar-logo"
    >
      <Flag className="h-6 w-6 text-primary" />
      <span className="text-lg font-bold whitespace-nowrap">Soul Banner</span>
    </Link>
  );
}
