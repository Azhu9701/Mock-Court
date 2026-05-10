"use client";

import { useState } from "react";
import { Breadcrumb } from "@/components/breadcrumb";
import { MobileMenuButton } from "@/components/mobile-menu-button";
import { QuickActions } from "@/components/quick-actions";
import { SettingsDialog } from "@/components/settings-dialog";
import { Button } from "@/components/ui/button";
import { Settings } from "lucide-react";

export function Header() {
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <>
      <header className="flex h-14 items-center gap-4 border-b bg-background px-4 lg:px-8" data-testid="header">
        <MobileMenuButton />
        <Breadcrumb />
        <div className="flex-1" />
        <QuickActions />
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setSettingsOpen(true)}
          data-testid="settings-btn"
          aria-label="设置"
        >
          <Settings className="h-4 w-4" />
        </Button>
      </header>
      <SettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} />
    </>
  );
}
