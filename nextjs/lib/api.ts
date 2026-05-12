export const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://127.0.0.1:3096/api/v1";

interface ApiError extends Error {
  status?: number;
  url?: string;
  operation?: string;
}

interface ApiRequestOptions extends RequestInit {
  operation?: string;
}

async function apiRequest<T>(
  endpoint: string,
  options: ApiRequestOptions = {}
): Promise<T> {
  const { operation, ...fetchOptions } = options;
  const url = `${API_BASE}${endpoint}`;
  const opName = operation || endpoint;

  const res = await fetch(url, fetchOptions);

  if (!res.ok) {
    let errorDetail = res.statusText;
    try {
      const errorData = await res.json();
      errorDetail = errorData.message || errorData.error || errorDetail;
    } catch { }

    const error = new Error(`API request failed: ${opName} - ${errorDetail}`) as ApiError;
    error.status = res.status;
    error.url = url;
    error.operation = opName;
    throw error;
  }

  if (res.status === 204) {
    return undefined as T;
  }

  return res.json();
}

export interface SoulListEntry {
  name: string;
  ismism_code: string;
  field: string;
  domains: string[];
  tags: string[];
  summon_count: number;
  trigger_keywords: string[];
  self_declare: string;
  model: string;
  compat: string[];
  incompat: string[];
}

export interface SoulMatch {
  entry: SoulListEntry;
  relevance: number;
  matched_fields: string[];
}

export interface SoulProfile {
  name: string;
  ismism_code: string;
  field: string;
  ontology: string;
  epistemology: string;
  teleology: string;
  domains: string[];
  tags: string[];
  exclude_scenarios: string[];
  summon_prompt: string;
  summon_count: number;
  effectiveness: { effective: number; partial: number; invalid: number };
  created_at: string;
  updated_at: string;
  practice_observations: PracticeObservation[];
  title: string;
  description: string;
  voice: string;
  mind: string;
  self_declare: string;
  skills_expertise: string[];
  model: string;
  tools: string;
  trigger_keywords: string[];
  compat: string[];
  incompat: string[];
}

export interface PracticeObservation {
  date: string;
  observation: string;
  revision_type: "Confirmed" | "Modified" | "Overturned";
}

export type ConferenceEvent =
  | { type: "soul_token";   soul: string; token: string }
  | { type: "soul_done";    soul: string }
  | { type: "soul_error";   soul: string; error: string }
  | { type: "synthesis_chunk"; content: string }
  | { type: "synthesis_done" }
  | { type: "collision";    from: string; to: string; content: string }
  | { type: "synthesis_started" }
  | { type: "cost";         llm_calls: number; tokens_used: number; estimated_cost: string }
  | { type: "done" }
  | { type: "session_started"; mode: string }
  | { type: "soul_started"; soul: string }
  | { type: "system";       message: string }
  | { type: "error";        message: string; soul?: string }

export interface FailureAlert {
  soul_name: string;
  alert_type: "boundary_review" | "suspension";
}

export interface KnowledgeResult {
  soul_name: string | null;
  content_snippet: string;
  mode: string;
  task_summary: string;
  created_at: string;
  session_id: string;
}

export interface KnowledgeTopic {
  session_id: string;
  title: string;
  mode: string;
  created_at: string;
  soul_names: string[];
  card_summary: string | null;
  synthesis_preview: string | null;
}

export interface KnowledgeCardItem {
  id: string;
  title: string;
  content: string;
  source_soul: string | null;
  source_session: string | null;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export async function fetchSouls(): Promise<SoulListEntry[]> {
  return apiRequest<SoulListEntry[]>('/souls', {
    next: { revalidate: 60 },
    operation: 'fetchSouls',
  });
}

export async function fetchSoul(name: string): Promise<SoulProfile> {
  return apiRequest<SoulProfile>(
    `/souls/${encodeURIComponent(name)}`,
    { next: { revalidate: 60 }, operation: 'fetchSoul' }
  );
}

export async function searchSouls(query: string): Promise<SoulMatch[]> {
  return apiRequest<SoulMatch[]>(
    `/souls/search?q=${encodeURIComponent(query)}`,
    { operation: 'searchSouls' }
  );
}

export async function deleteSoul(name: string): Promise<void> {
  return apiRequest<void>(
    `/souls/${encodeURIComponent(name)}`,
    { method: 'DELETE', operation: 'deleteSoul' }
  );
}

export async function updateSoul(
  name: string,
  data: Record<string, unknown>
): Promise<void> {
  return apiRequest<void>(
    `/souls/${encodeURIComponent(name)}`,
    {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
      operation: 'updateSoul',
    }
  );
}

export async function createSoul(data: Record<string, unknown>): Promise<void> {
  return apiRequest<void>('/souls', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
    operation: 'createSoul',
  });
}

// ── Analytics ──

export async function fetchSoulEffectiveness(
  name: string
): Promise<EffectivenessTrend> {
  return apiRequest<EffectivenessTrend>(
    `/analytics/soul-effectiveness/${encodeURIComponent(name)}`,
    { next: { revalidate: 60 }, operation: 'fetchSoulEffectiveness' }
  );
}

export async function fetchIsmismDistribution(): Promise<IsmismStats> {
  return apiRequest<IsmismStats>('/souls/ismism/distribution', {
    next: { revalidate: 60 },
    operation: 'fetchIsmismDistribution',
  });
}

export interface SummonStatsResponse {
  total_calls: number;
  unique_souls_called: number;
  total_souls_available: number;
  total_tokens: number;
  by_mode: Record<string, number>;
  by_soul: SoulCallStats[];
}

export interface SoulCallStats {
  soul_name: string;
  call_count: number;
  effective_count: number;
  partial_count: number;
  invalid_count: number;
  total_tokens: number;
}

export interface SoulAlert {
  soul_name: string;
  alert_type: string;
  detail: string;
}

export interface BoundaryReview {
  soul_name: string;
  effective_rate: number;
  total_calls: number;
  threshold: number;
  recommendation: string;
}

export interface EffectivenessTrend {
  soul_name: string;
  total_calls: number;
  effective: number;
  partial: number;
  invalid: number;
  effective_rate: number;
}

export interface IsmismStats {
  field_distribution: Record<number, number>;
  ontology_distribution: Record<number, number>;
  epistemology_distribution: Record<number, number>;
  teleology_distribution: Record<number, number>;
  total_souls: number;
}

export async function fetchSummonStats(): Promise<SummonStatsResponse> {
  return apiRequest<SummonStatsResponse>('/analytics/summon-stats', {
    next: { revalidate: 60 },
    operation: 'fetchSummonStats',
  });
}

export async function fetchModeDistribution(): Promise<Record<string, number>> {
  return apiRequest<Record<string, number>>('/analytics/mode-distribution', {
    next: { revalidate: 60 },
    operation: 'fetchModeDistribution',
  });
}

export async function fetchUnsummonedAlerts(days = 30): Promise<SoulAlert[]> {
  return apiRequest<SoulAlert[]>(
    `/analytics/unsummoned?threshold_days=${days}`,
    { next: { revalidate: 60 }, operation: 'fetchUnsummonedAlerts' }
  );
}

export async function fetchLowEffectiveness(threshold = 0.3): Promise<BoundaryReview[]> {
  return apiRequest<BoundaryReview[]>(
    `/analytics/low-effectiveness?threshold=${threshold}`,
    { next: { revalidate: 60 }, operation: 'fetchLowEffectiveness' }
  );
}

export async function fetchAudit(): Promise<FailureAlert[]> {
  return apiRequest<FailureAlert[]>('/analytics/audit', {
    next: { revalidate: 60 },
    operation: 'fetchAudit',
  });
}

// ── Sessions ──

export interface SessionSummary {
  id: string;
  title: string;
  mode: string;
  status: string;
  created_at: string;
  message_count: number;
  soul_count: number;
  total_tokens: number;
}

export interface SessionDetail {
  session: {
    id: string;
    title: string;
    mode: string;
    status: string;
    created_at: string;
    updated_at: string;
  };
  messages: Message[];
}

export interface Message {
  id: string;
  session_id: string;
  role: string;
  soul_name: string | null;
  content: string;
  seq: number;
  created_at: string;
}

export async function fetchSessions(
  limit = 50,
  offset = 0
): Promise<SessionSummary[]> {
  return apiRequest<SessionSummary[]>(
    `/sessions?limit=${limit}&offset=${offset}`,
    { next: { revalidate: 60 }, operation: 'fetchSessions' }
  );
}

export async function fetchSessionDetail(id: string): Promise<SessionDetail> {
  return apiRequest<SessionDetail>(`/sessions/${id}`, {
    next: { revalidate: 60 },
    operation: 'fetchSessionDetail',
  });
}

export async function deleteSession(id: string): Promise<void> {
  return apiRequest<void>(`/sessions/${id}`, {
    method: 'DELETE',
    operation: 'deleteSession',
  });
}

export async function renameSession(id: string, title: string): Promise<void> {
  return apiRequest<void>(`/sessions/${id}/rename`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title }),
    operation: 'renameSession',
  });
}

// ── Possess ──

export interface AnalyzeResponse {
  entry_type: string;
  matched_souls: { name: string; field: string; ismism_code: string; rationale: string }[];
  recommended_mode: string;
  review: { verdict: string; checks: string[]; notes: string; reviewer: string };
  task_cards?: Record<string, string>;
}

export interface AnalyzeStreamEvent {
  phase: "classifying" | "matched" | "reviewing" | "review_done" | "adjusting" | "practice_opening" | "done";
  entry_type?: string;
  souls?: { name: string; field: string; ismism_code: string; rationale: string }[];
  mode?: string;
  reviewer?: string;
  review?: { verdict: string; checks: string[]; notes: string; reviewer: string };
  response?: AnalyzeResponse;
}

export async function analyzeTask(
    task: string,
    reviewer?: string,
    signal?: AbortSignal,
    onEvent?: (event: AnalyzeStreamEvent) => void,
): Promise<AnalyzeResponse> {
    const url = `${API_BASE}/possess/analyze`;
    const res = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ task, reviewer }),
        signal,
    });

    if (!res.ok) {
        let errorDetail = res.statusText;
        try { const ed = await res.json(); errorDetail = ed.message || ed.error || errorDetail; } catch {}
        throw new Error(`API request failed: analyzeTask - ${errorDetail}`);
    }

    if (!res.body) throw new Error("analyzeTask: empty response body");
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    let finalResponse: AnalyzeResponse | null = null;
    let eventCount = 0;

    // 保存中间状态以便在未收到 done 事件时构造响应
    let intermediateEntryType = "conventional";
    let intermediateSouls: any[] = [];
    let intermediateMode = "single";
    let intermediateReview: any = null;

    while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";

        for (const line of lines) {
            const trimmed = line.trim();
            if (!trimmed.startsWith("data:")) continue;
            try {
                const event: AnalyzeStreamEvent = JSON.parse(trimmed.slice(5).trim());
                eventCount++;
                // 保存中间状态
                if (event.entry_type) intermediateEntryType = event.entry_type;
                if (event.souls) intermediateSouls = event.souls;
                if (event.mode) intermediateMode = event.mode;
                if (event.review) intermediateReview = event.review;
                if (event.phase === "done" && event.response) {
                    finalResponse = event.response;
                }
                onEvent?.(event);
            } catch {}
        }
    }

    // 如果没有收到 done 事件，尝试 fallback 机制
    if (!finalResponse) {
        // Fallback 1: 从中间状态构造响应
        if (intermediateSouls.length > 0 || intermediateReview) {
            console.warn("analyzeTask: stream ended without done event, constructing response from intermediate state");
            finalResponse = {
                entry_type: intermediateEntryType,
                matched_souls: intermediateSouls,
                recommended_mode: intermediateMode,
                review: intermediateReview || { verdict: "pass", checks: [], notes: "No review received", reviewer: "" },
                task_cards: {}
            };
        }
        // Fallback 2: 旧版后端返回的是纯 JSON 而非 SSE
        else if (eventCount === 0 && buffer.trim()) {
            try {
                const legacyResponse = JSON.parse(buffer.trim()) as AnalyzeResponse;
                finalResponse = legacyResponse;
            } catch {}
        }
    }

    if (!finalResponse) {
        throw new Error("analyzeTask: stream ended without done event");
    }
    return finalResponse;
}

export interface StartPossessionResponse {
  session_id: string;
  ws_url: string;
}

export async function startPossession(params: {
  task: string;
  mode?: string;
  souls: string[];
  task_cards?: Record<string, string>;
  search_topic?: boolean;
  judgment?: string;
  worry?: string;
  unknown?: string;
}): Promise<StartPossessionResponse> {
  return apiRequest<StartPossessionResponse>('/possess', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
    operation: 'startPossession',
  });
}

export async function exportSessionMarkdown(id: string, title: string): Promise<void> {
  const url = `${API_BASE}/sessions/${id}/export/markdown`;
  const res = await fetch(url);
  if (!res.ok) {
    const error = new Error(`Export failed: ${res.statusText}`) as ApiError;
    error.status = res.status;
    error.url = url;
    error.operation = 'exportSessionMarkdown';
    throw error;
  }
  const blob = await res.blob();
  const downloadUrl = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = downloadUrl;
  a.download = `${title}.md`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(downloadUrl);
}

// ── Review ──

export interface ReviewData {
  most_unexpected?: string;
  already_known?: string;
  self_negation?: string;
  empty_chair?: string;
  effectiveness?: string;
  effectiveness_note?: string;
}

export async function saveReview(sessionId: string, data: ReviewData): Promise<void> {
  return apiRequest(`/sessions/${sessionId}/review`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
    operation: "saveReview",
  });
}

// ── Knowledge ──

export async function searchKnowledge(query: string, limit = 20): Promise<KnowledgeResult[]> {
  return apiRequest<KnowledgeResult[]>(
    `/knowledge/search?q=${encodeURIComponent(query)}&limit=${limit}`,
    { operation: 'searchKnowledge' }
  );
}

export async function rebuildFts(): Promise<{ indexed: number }> {
  return apiRequest<{ indexed: number }>('/knowledge/rebuild', {
    method: 'POST',
    operation: 'rebuildFts',
  });
}

export async function fetchKnowledgeTopics(params?: {
  mode?: string;
  limit?: number;
  offset?: number;
}): Promise<KnowledgeTopic[]> {
  const searchParams = new URLSearchParams();
  if (params?.mode) searchParams.set('mode', params.mode);
  if (params?.limit) searchParams.set('limit', String(params.limit));
  if (params?.offset) searchParams.set('offset', String(params.offset));
  const qs = searchParams.toString();
  return apiRequest<KnowledgeTopic[]>(
    `/knowledge/topics${qs ? `?${qs}` : ''}`,
    { operation: 'fetchKnowledgeTopics' }
  );
}

export async function fetchKnowledgeCards(params?: {
  soul?: string;
  limit?: number;
  offset?: number;
}): Promise<KnowledgeCardItem[]> {
  const searchParams = new URLSearchParams();
  if (params?.soul) searchParams.set('soul', params.soul);
  if (params?.limit) searchParams.set('limit', String(params.limit));
  if (params?.offset) searchParams.set('offset', String(params.offset));
  const qs = searchParams.toString();
  return apiRequest<KnowledgeCardItem[]>(
    `/knowledge/cards${qs ? `?${qs}` : ''}`,
    { operation: 'fetchKnowledgeCards' }
  );
}

export async function saveVerificationKnowledgeCard(params: {
  session_id: string;
  title: string;
  action: string;
  valid_signal: string;
  revision_signal: string;
}): Promise<KnowledgeCardItem> {
  return apiRequest<KnowledgeCardItem>('/knowledge/cards', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      source_session: params.session_id,
      title: params.title,
      content: `## ⏳ 24小时检验项

**检验行动：** ${params.action}

**有效信号：** ${params.valid_signal}

**修正信号：** ${params.revision_signal}`,
      tags: ['实践检验', '24小时'],
      source_soul: 'user',
    }),
    operation: 'saveVerificationKnowledgeCard',
  });
}

// ── Synthesis (structured output §9.5) ──

export interface SynthesisOutput {
  consensus: ConsensusItem[];
  divergence: DivergenceItem[];
  blind_spots: BlindSpotItem[];
  principal_contradiction: Contradiction;
  action_program: ActionItem[];
}

export interface ConsensusItem {
  point: string;
  shared_by: string[];
}

export interface DivergenceItem {
  axis: string;
  positions: Position[];
}

export interface Position {
  soul_name: string;
  stance: string;
}

export interface BlindSpotItem {
  dimension: string;
  missing_perspective: string;
  coverable_by_existing: boolean;
  suggested_soul: string | null;
}

export interface Contradiction {
  description: string;
  parties: string[];
}

export interface ActionItem {
  direction: string;
  rationale: string;
  priority: number;
}

// ── DeepSeek Cache Hint ──
// Indicates the prompt was constructed for maximum prefix cache hit rate

export interface CacheHint {
  provider: "deepseek";
  cache_optimized: boolean;
  estimated_discount: string; // "80-92%"
}

export interface OcrResult {
  filename: string;
  text: string | null;
  error: string | null;
}

export async function ocrFiles(files: File[]): Promise<OcrResult[]> {
  const form = new FormData();
  files.forEach((f) => form.append("files", f));
  const res = await fetch(`${API_BASE}/possess/ocr`, {
    method: "POST",
    body: form,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || `OCR failed: ${res.statusText}`);
  }
  const data = await res.json();
  return data.results;
}

// ── SearXNG ──

export interface SearxngResultItem {
  title: string;
  url: string;
  content: string;
  engine: string;
  engines: string[];
  score: number;
  category: string;
}

export interface SearxngSearchResponse {
  query: string;
  number_of_results: number;
  results: SearxngResultItem[];
  suggestions: string[];
  unresponsive_engines: string[][];
}

export async function searchWeb(params: {
  q: string;
  pageno?: number;
  language?: string;
  categories?: string;
}): Promise<SearxngSearchResponse> {
  const searchParams = new URLSearchParams();
  searchParams.set('q', params.q);
  if (params.pageno) searchParams.set('pageno', String(params.pageno));
  if (params.language) searchParams.set('language', params.language);
  if (params.categories) searchParams.set('categories', params.categories);
  return apiRequest<SearxngSearchResponse>(
    `/searxng/search?${searchParams.toString()}`,
    { operation: 'searchWeb' }
  );
}
