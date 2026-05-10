"use client";

import { createContext, useCallback, useContext, useEffect, useState } from "react";

const DEFAULT_WIDTH = 240;
const MIN_WIDTH = 180;
const MAX_WIDTH = 400;

interface SidebarContextType {
  collapsed: boolean;
  width: number;
  toggle: () => void;
  setWidth: (width: number) => void;
  ready: boolean;
}

const SidebarContext = createContext<SidebarContextType | undefined>(undefined);

export function SidebarProvider({ children }: { children: React.ReactNode }) {
  const [collapsed, setCollapsed] = useState(false);
  const [width, setWidthState] = useState(DEFAULT_WIDTH);
  const [ready, setReady] = useState(false);

  // Sync from localStorage after mount (avoids hydration mismatch)
  useEffect(() => {
    const storedCollapsed = localStorage.getItem("sidebar-collapsed") === "true";
    const storedWidth = parseInt(localStorage.getItem("sidebar-width") || String(DEFAULT_WIDTH), 10);
    setCollapsed(storedCollapsed);
    setWidthState(isNaN(storedWidth) ? DEFAULT_WIDTH : Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, storedWidth)));
    setReady(true);
  }, []);

  const toggle = useCallback(() => {
    setCollapsed((prev) => {
      localStorage.setItem("sidebar-collapsed", String(!prev));
      return !prev;
    });
  }, []);

  const setWidth = useCallback((newWidth: number) => {
    const clampedWidth = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, newWidth));
    setWidthState(clampedWidth);
    localStorage.setItem("sidebar-width", String(clampedWidth));
  }, []);

  return (
    <SidebarContext.Provider value={{ collapsed, width, toggle, setWidth, ready }}>
      {children}
    </SidebarContext.Provider>
  );
}

export function useSidebar() {
  const ctx = useContext(SidebarContext);
  if (!ctx) throw new Error("useSidebar must be used within SidebarProvider");
  return ctx;
}
