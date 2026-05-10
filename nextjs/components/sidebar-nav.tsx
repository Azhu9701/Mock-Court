import { navConfig } from "@/config/nav";
import { NavItem } from "@/components/nav-item";

interface SidebarNavProps {
  currentPath: string;
}

export function SidebarNav({ currentPath }: SidebarNavProps) {
  return (
    <nav className="flex-1 overflow-y-auto p-2" aria-label="主导航" data-testid="sidebar-nav">
      {navConfig.map((group) => (
        <div key={group.label} className="mb-4">
          <h3 className="mb-1 px-3 text-xs font-semibold text-muted-foreground">
            {group.label}
          </h3>
          {group.items.map((item) => (
            <NavItem
              key={item.key}
              item={item}
              active={currentPath.startsWith(item.href)}
            />
          ))}
        </div>
      ))}
    </nav>
  );
}
