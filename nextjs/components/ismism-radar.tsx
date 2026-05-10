"use client";

import { parseIsmismCode, ISMISM_LABELS } from "@/config/soul-filter";

const FIELD_LEVELS = ["未标定", "秩序主义", "表征主义", "反思主义", "批判实践"];
const ONTOLOGY_LEVELS = ["未标定", "物质实在", "道/结构", "主体/生命", "符号/虚无"];
const EPISTEMOLOGY_LEVELS = ["未标定", "经验实证", "理性演绎", "直觉体验", "辩证批判"];
const TELEOLOGY_LEVELS = ["未标定", "维持回归", "建构改良", "完成解放", "否定消解"];

interface IsmismRadarProps {
  ismismCode: string;
}

function levelBadge(value: number, levels: string[]) {
  const label = levels[value] || levels[0];
  return (
    <span className="text-xs rounded-full bg-muted px-1.5 py-0.5 text-muted-foreground whitespace-nowrap">
      {value} · {label}
    </span>
  );
}

export function IsmismRadar({ ismismCode }: IsmismRadarProps) {
  const code = parseIsmismCode(ismismCode);
  if (!code) return null;

  return (
    <div data-testid="ismism-radar" className="space-y-3">
      <h3 className="text-sm font-semibold">主义主义坐标</h3>

      <div className="space-y-2">
        <div className="grid grid-cols-2 gap-1.5 text-xs">
          <div className="flex items-center justify-between gap-2 rounded-md border px-2.5 py-1.5">
            <span className="font-medium shrink-0">{ISMISM_LABELS.field}</span>
            {levelBadge(code.field, FIELD_LEVELS)}
          </div>
          <div className="flex items-center justify-between gap-2 rounded-md border px-2.5 py-1.5">
            <span className="font-medium shrink-0">{ISMISM_LABELS.ontology}</span>
            {levelBadge(code.ontology, ONTOLOGY_LEVELS)}
          </div>
          <div className="flex items-center justify-between gap-2 rounded-md border px-2.5 py-1.5">
            <span className="font-medium shrink-0">{ISMISM_LABELS.epistemology}</span>
            {levelBadge(code.epistemology, EPISTEMOLOGY_LEVELS)}
          </div>
          <div className="flex items-center justify-between gap-2 rounded-md border px-2.5 py-1.5">
            <span className="font-medium shrink-0">{ISMISM_LABELS.teleology}</span>
            {levelBadge(code.teleology, TELEOLOGY_LEVELS)}
          </div>
        </div>
        <p className="text-xs text-muted-foreground font-mono text-center sm:text-left">
          编码：{ismismCode}
        </p>
      </div>
    </div>
  );
}
