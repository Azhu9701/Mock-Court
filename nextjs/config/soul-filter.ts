export interface IsmismCode {
  field: number;
  ontology: number;
  epistemology: number;
  teleology: number;
}

export interface IsmismSliderValues {
  field: [number, number];
  ontology: [number, number];
  epistemology: [number, number];
  teleology: [number, number];
}

export function parseIsmismCode(s: string): IsmismCode | null {
  const parts = s.split("-");
  if (parts.length !== 4) return null;
  const nums = parts.map(Number);
  if (nums.some(isNaN)) return null;
  return {
    field: nums[0],
    ontology: nums[1],
    epistemology: nums[2],
    teleology: nums[3],
  };
}

export function ismismDistance(a: IsmismCode, b: IsmismCode): number {
  return Math.sqrt(
    (a.field - b.field) ** 2 +
    (a.ontology - b.ontology) ** 2 +
    (a.epistemology - b.epistemology) ** 2 +
    (a.teleology - b.teleology) ** 2
  );
}

export function isWithinRange(
  code: IsmismCode,
  ranges: IsmismSliderValues
): boolean {
  return (
    code.field >= ranges.field[0] && code.field <= ranges.field[1] &&
    code.ontology >= ranges.ontology[0] && code.ontology <= ranges.ontology[1] &&
    code.epistemology >= ranges.epistemology[0] && code.epistemology <= ranges.epistemology[1] &&
    code.teleology >= ranges.teleology[0] && code.teleology <= ranges.teleology[1]
  );
}

export const ISMISM_LABELS = {
  field: "领域",
  ontology: "本体论",
  epistemology: "认识论",
  teleology: "目的论",
} as const;
