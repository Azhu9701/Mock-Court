"use client";

import { Menu } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSidebar } from "@/contexts/sidebar-context";

export function MobileMenuButton() {
  const { collapsed, toggle } = useSidebar();

  return (
    <Button
      variant="ghost"
      size="icon"
      className="lg:hidden"
      onClick={toggle}
      data-testid="mobile-menu-btn"
      aria-label="切换菜单"
    >
      <Menu className="h-5 w-5" />
    </Button>
  );
}
