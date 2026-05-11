const STORAGE_KEY = "aionui-pending-session";

export interface PendingSessionMeta {
  sessionId: string;
  task: string;
  mode: string;
  matchedSouls: { name: string; field: string; ismism_code: string; rationale: string }[];
  review: { verdict: string; checks: string[]; notes: string; reviewer: string } | null;
}

export function storePendingSession(meta: PendingSessionMeta) {
  sessionStorage.setItem(STORAGE_KEY, JSON.stringify(meta));
}

export function popPendingSession(): PendingSessionMeta | null {
  const raw = sessionStorage.getItem(STORAGE_KEY);
  if (!raw) return null;
  sessionStorage.removeItem(STORAGE_KEY);
  try {
    return JSON.parse(raw) as PendingSessionMeta;
  } catch {
    return null;
  }
}

export function keepPendingSession() {
  // Called when the user refreshes the page; don't consume the metadata yet
}
