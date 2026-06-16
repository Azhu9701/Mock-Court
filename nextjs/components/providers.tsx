"use client";

import { ThemeProvider } from "next-themes";
import { SidebarProvider } from "@/contexts/sidebar-context";
import { DomainProvider } from "@/contexts/domain-context";
import { ToastProvider } from "@/components/ui/toast";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <ToastProvider>
        <DomainProvider>
          <SidebarProvider>{children}</SidebarProvider>
        </DomainProvider>
      </ToastProvider>
    </ThemeProvider>
  );
}
