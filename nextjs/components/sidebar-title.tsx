"use client";

import { useDomain } from "@/contexts/domain-context";

/**
 * Sidebar 标题——根据当前领域动态显示。
 * ready=false 时显示默认值 "模拟仲裁庭" 以防 hydration mismatch。
 */
export function SidebarTitle() {
  const { systemName, ready } = useDomain();
  return (
    <span className="text-lg font-bold whitespace-nowrap">
      {ready ? systemName : "模拟仲裁庭"}
    </span>
  );
}
