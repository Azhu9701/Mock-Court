"use client";

import { Suspense } from "react";
import { PossessionEntry } from "@/components/possession-entry";

export default function PossessPage() {
  return (
    <div className="space-y-6">
      <Suspense>
        <PossessionEntry />
      </Suspense>
    </div>
  );
}
