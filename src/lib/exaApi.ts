import { getApiBase } from '$lib/apiBase';

// Wrappers for the Exa-backed Worker routes (api/src/handlers/{radar,ask,fitness_research}.rs).
// The Worker holds the Exa key, caches answers, and rate limits callers; these
// calls are public and need no sign-in.

export type Source = { url: string; title: string };

export type RadarItem = {
  url: string;
  topic: string;
  title: string;
  authors: string;
  /** YYYY-MM-DD */
  published: string;
  summary: string;
  /** Unix seconds when the radar first found it. */
  found: number;
};

export type Radar = {
  updated: number;
  topics: { id: string; label: string }[];
  items: RadarItem[];
};

export type Answer = { answer: string; sources: Source[]; cached: boolean };

export type SearchHit = { url: string; title: string; snippet: string; published: string | null };

/** The training signals the Worker turns into fixed research questions. */
export type ResearchSignal =
  | { kind: 'stall'; exercise: string }
  | { kind: 'low_volume'; muscle: string }
  | { kind: 'weight_rate'; band: string }
  | { kind: 'break'; days: number }
  | { kind: 'low_cardio' };

export type ResearchNote = { question: string; answer: string; sources: Source[] };

async function call<T>(path: string, init: RequestInit = {}): Promise<T> {
  const res = await fetch(`${getApiBase()}${path}`, {
    ...init,
    headers: init.body ? { 'Content-Type': 'application/json' } : undefined
  });
  if (!res.ok) {
    let msg = `request failed (${res.status})`;
    try {
      const body = await res.json();
      if (body?.error) msg = body.error;
    } catch {
      /* not JSON */
    }
    throw new Error(msg);
  }
  return (await res.json()) as T;
}

export const exaApi = {
  radar: () => call<Radar>('/api/radar'),
  ask: (question: string) =>
    call<Answer>('/api/ask', { method: 'POST', body: JSON.stringify({ question }) }),
  search: (q: string) =>
    call<{ results: SearchHit[]; cached: boolean }>(`/api/search?q=${encodeURIComponent(q)}`),
  research: (signals: ResearchSignal[]) =>
    call<ResearchNote[]>('/api/fitness/research', {
      method: 'POST',
      body: JSON.stringify({ signals })
    })
};

/**
 * Split an answer on its `[n]` citation markers so a page can render each marker
 * as a link to source n (1-based). Markers past the last source stay as text.
 */
export function citeParts(answer: string, sources: Source[]): ({ text: string } | { n: number; url: string })[] {
  const out: ({ text: string } | { n: number; url: string })[] = [];
  let last = 0;
  for (const m of answer.matchAll(/\[(\d+)\]/g)) {
    const n = Number(m[1]);
    const src = sources[n - 1];
    if (!src) continue;
    if (m.index! > last) out.push({ text: answer.slice(last, m.index) });
    out.push({ n, url: src.url });
    last = m.index! + m[0].length;
  }
  if (last < answer.length) out.push({ text: answer.slice(last) });
  return out;
}
