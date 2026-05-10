const ACCENT_PALETTE = [
  "bg-red-500",
  "bg-blue-500",
  "bg-purple-500",
  "bg-green-500",
  "bg-orange-500",
  "bg-pink-500",
  "bg-indigo-500",
  "bg-teal-500",
] as const;

const AVATAR_BG_PALETTE = [
  "bg-red-50 text-red-600 dark:bg-red-950 dark:text-red-400",
  "bg-blue-50 text-blue-600 dark:bg-blue-950 dark:text-blue-400",
  "bg-purple-50 text-purple-600 dark:bg-purple-950 dark:text-purple-400",
  "bg-green-50 text-green-600 dark:bg-green-950 dark:text-green-400",
  "bg-orange-50 text-orange-600 dark:bg-orange-950 dark:text-orange-400",
  "bg-pink-50 text-pink-600 dark:bg-pink-950 dark:text-pink-400",
  "bg-indigo-50 text-indigo-600 dark:bg-indigo-950 dark:text-indigo-400",
  "bg-teal-50 text-teal-600 dark:bg-teal-950 dark:text-teal-400",
] as const;

function hashString(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) - hash + str.charCodeAt(i)) | 0;
  }
  return Math.abs(hash);
}

export function getSoulAccent(ismismCode: string): string {
  if (!ismismCode) return "bg-primary";
  return ACCENT_PALETTE[hashString(ismismCode) % ACCENT_PALETTE.length];
}

export function getSoulAvatarBg(ismismCode: string): string {
  if (!ismismCode) return "bg-muted text-muted-foreground";
  return AVATAR_BG_PALETTE[hashString(ismismCode) % AVATAR_BG_PALETTE.length];
}
