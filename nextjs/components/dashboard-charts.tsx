"use client";

import dynamic from "next/dynamic";
import { SoulEffectivenessTable } from "@/components/soul-effectiveness-table";
import { Skeleton } from "@/components/ui/skeleton";

const ModeBarChart = dynamic(
  () => import("@/components/mode-bar-chart").then((mod) => mod.ModeBarChart),
  {
    ssr: false,
    loading: () => <Skeleton className="h-64 rounded-xl" />,
  }
);

export { ModeBarChart, SoulEffectivenessTable };
