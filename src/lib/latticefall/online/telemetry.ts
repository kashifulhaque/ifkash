// Opt-in, anonymous, batched telemetry. Nothing is recorded or sent unless the player
// turned it on (the default is off). Events queue in memory and go out in one request at
// session end, or every 10 minutes while the page stays open. The server keeps them 90
// days. No Three.js or WASM import.

import { postTelemetryBatch } from './api';
import { newSessionId } from './ids';

const KEY = 'ifkash-latticefall-telemetry';
const FLUSH_MS = 10 * 60 * 1000;
const MAX_QUEUE = 300;

export type TelemetryEvent =
  | { k: 'session_start'; build?: string }
  | { k: 'session_end'; playMs: number }
  | { k: 'area_entered'; area: string }
  | { k: 'lantern_lit'; area: string; lantern: string }
  /** `cell` is the 8-meter grid cell, for example `"12,-3"`. */
  | { k: 'death'; area: string; cell: string; cause: string }
  | { k: 'boss_attempt'; boss: string; phase: number; durationMs: number; result: 'win' | 'loss' | 'quit' }
  | { k: 'difficulty'; setting: string; value: string | number | boolean }
  | { k: 'link_click'; target: string };

type Stamped = Record<string, string | number | boolean>;

let queue: Stamped[] = [];
let sessionId = newSessionId();
let timer: ReturnType<typeof setInterval> | null = null;
let flushing = false;

/** Whether the player opted in. Off by default. */
export function telemetryEnabled(): boolean {
  try {
    return localStorage.getItem(KEY) === 'on';
  } catch {
    return false;
  }
}

export function setTelemetryEnabled(on: boolean): void {
  try {
    if (on) localStorage.setItem(KEY, 'on');
    else localStorage.removeItem(KEY);
  } catch {
    /* storage unavailable */
  }
  if (!on) queue = [];
}

/** Queues an event. Does nothing when the player hasn't opted in. */
export function track(event: TelemetryEvent): void {
  if (!telemetryEnabled()) return;
  if (queue.length >= MAX_QUEUE) queue.shift();
  const stamped: Stamped = { t: Date.now() };
  for (const [key, value] of Object.entries(event)) {
    if (value !== undefined) stamped[key] = value;
  }
  queue.push(stamped);
}

/** Sends the queue in one request. A failure keeps the events for the next flush. */
export async function flushTelemetry(): Promise<void> {
  if (flushing || queue.length === 0 || !telemetryEnabled()) return;
  flushing = true;
  const batch = queue.splice(0, queue.length);
  try {
    await postTelemetryBatch(sessionId, batch);
  } catch {
    queue = [...batch, ...queue].slice(-MAX_QUEUE);
  } finally {
    flushing = false;
  }
}

/**
 * Starts a session: queues `session_start`, flushes every 10 minutes, and flushes when the
 * page hides. Returns a function that ends the session (queues `session_end`, flushes,
 * stops the timer).
 */
export function startTelemetrySession(build?: string): () => Promise<void> {
  sessionId = newSessionId();
  const startedAt = Date.now();
  track({ k: 'session_start', build });
  if (timer) clearInterval(timer);
  timer = setInterval(() => void flushTelemetry(), FLUSH_MS);
  const onHide = () => {
    if (document.visibilityState === 'hidden') void flushTelemetry();
  };
  if (typeof document !== 'undefined') document.addEventListener('visibilitychange', onHide);
  return async () => {
    if (timer) clearInterval(timer);
    timer = null;
    if (typeof document !== 'undefined') document.removeEventListener('visibilitychange', onHide);
    track({ k: 'session_end', playMs: Date.now() - startedAt });
    await flushTelemetry();
  };
}
