import { ThemeToggle } from "@/components/theme-toggle";
import { Button } from "@/components/ui/button";
import { ChevronLeft } from "lucide-react";
import { useSidebar } from "@/contexts/sidebar-context";

export function SidebarFooter() {
  const { toggle } = useSidebar();
  return (
    <div
      className="flex items-center justify-between border-t p-2"
      data-testid="sidebar-footer"
    >
      <ThemeToggle />
      <Button
        variant="ghost"
        size="icon"
        onClick={toggle}
        data-testid="sidebar-collapse-btn"
        aria-label="折叠侧边栏"
      >
        <ChevronLeft className="h-4 w-4" />
      </Button>
    </div>
  );
}
