import { notFound } from "next/navigation";
import { fetchSoul } from "@/lib/api";
import { IsmismRadar } from "@/components/ismism-radar";
import { SoulPrompt } from "@/components/soul-prompt";
import { PracticeObservations } from "@/components/practice-observations";
import { SummonButton } from "@/components/summon-button";
import { SoulModelConfig } from "@/components/soul-model-config";
import { Calendar } from "lucide-react";

export const dynamic = "force-dynamic";

interface SoulDetailPageProps {
  params: Promise<{ name: string }>;
}

export default async function SoulDetailPage({ params }: SoulDetailPageProps) {
  const { name } = await params;
  const decodedName = decodeURIComponent(name);

  let profile;
  try {
    profile = await fetchSoul(decodedName);
  } catch {
    notFound();
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <h1 className="text-2xl font-bold truncate">{profile.name}</h1>
        </div>
        <SummonButton soulName={profile.name} />
      </div>

      <div className="flex items-center gap-1 text-xs text-muted-foreground">
        <Calendar className="h-3 w-3 shrink-0" />
        创建于 {new Date(profile.created_at).toLocaleDateString("zh-CN")}
      </div>

      {profile.domains.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {profile.domains.map((d) => (
            <span
              key={d}
              className="rounded-md bg-muted px-2.5 py-1 text-xs"
            >
              {d}
            </span>
          ))}
        </div>
      )}

      {profile.tags.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {profile.tags.map((t) => (
            <span
              key={t}
              className="rounded-full border px-2.5 py-1 text-xs text-muted-foreground"
            >
              {t}
            </span>
          ))}
        </div>
      )}

      <IsmismRadar ismismCode={profile.ismism_code} />

      <SoulPrompt prompt={profile.summon_prompt} soulName={profile.name} />

      <SoulModelConfig soulName={profile.name} currentModel={profile.model} />

      <PracticeObservations observations={profile.practice_observations || []} />
    </div>
  );
}
