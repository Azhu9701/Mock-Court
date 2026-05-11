"use client";

import { createContext, useContext, useState, ReactNode } from "react";

interface BreadcrumbContextType {
  lastLabel: string | null;
  setLastLabel: (label: string | null) => void;
}

const BreadcrumbContext = createContext<BreadcrumbContextType>({
  lastLabel: null,
  setLastLabel: () => {},
});

export function BreadcrumbProvider({ children }: { children: ReactNode }) {
  const [lastLabel, setLastLabel] = useState<string | null>(null);

  return (
    <BreadcrumbContext.Provider value={{ lastLabel, setLastLabel }}>
      {children}
    </BreadcrumbContext.Provider>
  );
}

export function useBreadcrumb() {
  return useContext(BreadcrumbContext);
}
