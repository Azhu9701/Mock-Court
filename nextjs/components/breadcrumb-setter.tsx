"use client";

import { useEffect } from "react";
import { useBreadcrumb } from "@/contexts/breadcrumb-context";

export function BreadcrumbSetter({ label }: { label: string | null }) {
  const { setLastLabel } = useBreadcrumb();

  useEffect(() => {
    setLastLabel(label);
    return () => setLastLabel(null);
  }, [label, setLastLabel]);

  return null;
}
