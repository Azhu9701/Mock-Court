const STORAGE_KEY = "aionui-pending-session";

export interface PendingSessionMeta {
  sessionId: string;
  task: string;
  mode: string | null;
  matchedSouls: { name: string; field: string; ismism_code: string; rationale: string }[];
  review: { verdict: string; checks: string[]; notes: string; reviewer: string } | null;
  phases: string[];        // already completed phases
  searchTopic: boolean;
  judgment: string;
  worry: string;
  unknown: string;
  needsAnalysis: boolean;  // true → session page should call analyzeTask
}

export function storePendingSession(meta: PendingSessionMeta) {
  sessionStorage.setItem(STORAGE_KEY, JSON.stringify(meta));
}

export function readPendingSession(sessionId: string): PendingSessionMeta | null {
  const raw = sessionStorage.getItem(STORAGE_KEY);
  if (!raw) return null;
  try {
    const meta = JSON.parse(raw) as PendingSessionMeta;
    if (meta.sessionId === sessionId) return meta;
  } catch {}
  return null;
}

export function clearPendingSession(sessionId: string) {
  const raw = sessionStorage.getItem(STORAGE_KEY);
  if (!raw) return;
  try {
    const meta = JSON.parse(raw);
    if (meta.sessionId === sessionId) sessionStorage.removeItem(STORAGE_KEY);
  } catch {
    sessionStorage.removeItem(STORAGE_KEY);
  }
}
