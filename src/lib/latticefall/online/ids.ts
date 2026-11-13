// Identifiers the online features share. No Three.js or WASM import.

/**
 * Number ranges WP-084 reserved. The client owns these; wiring work uses them when it
 * hooks the online features into the world.
 */
export const ONLINE_RESERVED = {
  /** Registry numbers for message, Afterimage, and Wanderer content. */
  registry: { min: 45000, max: 45999 },
  /** WASM event kind for "an online record was touched" (the host decides the payload). */
  wasmEventKind: 55,
  /** HUD prompt codes: 86 reads a message, 87 replays an Afterimage. */
  hudPrompt: { readMessage: 86, replayAfterimage: 87 }
} as const;

const ANON_KEY = 'ifkash-latticefall-anon';

function randomId(): string {
  const bytes = new Uint8Array(16);
  if (typeof crypto !== 'undefined' && crypto.getRandomValues) crypto.getRandomValues(bytes);
  else for (let i = 0; i < bytes.length; i++) bytes[i] = Math.floor(Math.random() * 256);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

let memoryId: string | null = null;

/**
 * The anonymous per-install ID. It lives in `localStorage` and is sent with writes. The
 * server links it to a Google account only when the player signs in to write.
 */
export function anonId(): string {
  try {
    const saved = localStorage.getItem(ANON_KEY);
    if (saved && /^[A-Za-z0-9_.-]{8,64}$/.test(saved)) return saved;
    const fresh = randomId();
    localStorage.setItem(ANON_KEY, fresh);
    return fresh;
  } catch {
    memoryId ??= randomId();
    return memoryId;
  }
}

/** Forgets the install ID (the next call makes a new one). */
export function resetAnonId(): void {
  memoryId = null;
  try {
    localStorage.removeItem(ANON_KEY);
  } catch {
    /* storage unavailable */
  }
}

/** A fresh ID for one play session (telemetry groups a session's events by it). */
export const newSessionId = randomId;
