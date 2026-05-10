"use client";

import { Sidebar } from "@/components/sidebar";
import { Header } from "@/components/header";

export function ShellLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen overflow-hidden" data-testid="shell-layout">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header />
        <main
          className="flex-1 overflow-y-auto p-3 sm:p-4 md:p-6 lg:p-8"
          data-testid="main-content"
        >
          {children}
        </main>
      </div>
    </div>
  );
}
