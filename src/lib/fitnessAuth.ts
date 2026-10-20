// Shared Google Identity helpers for the fitness pages.
//
// A Google ID token dies about an hour after issue, and there is no refresh
// token. A phone with the screen off suspends timers, so the old "refresh one
// minute before expiry" schedule often never fired and the next save signed
// the user out mid-workout. Two layers cover that:
//   - `adoptSession` exchanges the Google token for a Worker session that
//     lasts well past a workout and renews when the app is opened.
//   - `startAuthWatch` asks Google for a new ID token when that session is
//     actually running out, including when the page becomes visible again.
//     It stays quiet while the session still has time left, so One Tap does
//     not pop over a set.

import { getApiBase } from '$lib/apiBase';
import { loadGoogleToken, loadSession, saveSession, type MintedSession } from '$lib/splitterApi';

/** Epoch-ms expiry from a JWT's `exp` claim, or null if it can't be read. */
export function tokenExpMs(token: string | null): number | null {
  if (!token) return null;
  try {
    const part = token.split('.')[1];
    if (!part) return null;
    const json = atob(part.replace(/-/g, '+').replace(/_/g, '/'));
    const payload = JSON.parse(json);
    return typeof payload.exp === 'number' ? payload.exp * 1000 : null;
  } catch {
    return null;
  }
}

type GoogleId = {
  initialize: (opts: Record<string, unknown>) => void;
  prompt: () => void;
};

function googleId(): GoogleId | undefined {
  return (window as unknown as { google?: { accounts?: { id?: GoogleId } } }).google?.accounts?.id;
}

type CredentialCallback = (resp: { credential: string }) => void;

/**
 * Exchange a live Google ID token for a Worker session. No-op when a session
 * still has more than 12 hours left, or when there is no Google token to
 * exchange (offline, local dev, or the ID token has already expired).
 */
export async function adoptSession(): Promise<MintedSession | null> {
  const existing = loadSession();
  if (existing && existing.exp * 1000 - Date.now() > 12 * 60 * 60 * 1000) return existing;
  const google = loadGoogleToken();
  if (!google || google === 'local-dev') return existing;
  const exp = tokenExpMs(google);
  if (exp !== null && exp <= Date.now() + 5_000) return existing;
  try {
    const res = await fetch(`${getApiBase()}/api/auth/session`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${google}` }
    });
    if (!res.ok) return existing;
    const body = (await res.json()) as MintedSession;
    if (!body?.token || typeof body.exp !== 'number') return existing;
    saveSession(body);
    return body;
  } catch {
    return existing;
  }
}

/**
 * Keep the Google credential fresh while the page is open, and again when
 * the phone wakes. Prompts only once the Worker session is inside its last
 * half hour (or there is no session) and the Google token is inside its last
 * 15 minutes. Returns a cleanup function.
 */
export function startAuthWatch(clientId: string, onCredential: CredentialCallback): () => void {
  if (!clientId || typeof window === 'undefined') return () => {};
  let lastPrompt = 0;
  let initialized = false;

  const maybePrompt = () => {
    if (document.visibilityState === 'hidden') return;
    const sessionExp = loadSession();
    const sessionMs = sessionExp ? sessionExp.exp * 1000 : 0;
    if (sessionMs - Date.now() > 30 * 60 * 1000) return;
    const google = loadGoogleToken();
    if (!google || google === 'local-dev') return;
    const exp = tokenExpMs(google);
    if (exp !== null && exp - Date.now() > 15 * 60 * 1000) return;
    if (Date.now() - lastPrompt < 10 * 60 * 1000) return;
    const id = googleId();
    if (!id) return;
    lastPrompt = Date.now();
    if (!initialized) {
      id.initialize({ client_id: clientId, callback: onCredential, auto_select: true });
      initialized = true;
    }
    id.prompt();
  };

  const onVisible = () => {
    if (document.visibilityState === 'visible') maybePrompt();
  };
  document.addEventListener('visibilitychange', onVisible);
  window.addEventListener('focus', maybePrompt);
  const timer = setInterval(maybePrompt, 60_000);
  maybePrompt();
  return () => {
    document.removeEventListener('visibilitychange', onVisible);
    window.removeEventListener('focus', maybePrompt);
    clearInterval(timer);
  };
}

/** Load the Google Identity Services script once; resolves when it's ready. */
export function loadGis(): Promise<void> {
  if (googleId()) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const s = document.createElement('script');
    s.src = 'https://accounts.google.com/gsi/client';
    s.async = true;
    s.onload = () => resolve();
    s.onerror = () => reject();
    document.head.appendChild(s);
  });
}

/** Render the "Sign in with Google" button into `el`. No-op until GIS has loaded. */
export function renderGoogleButton(
  el: HTMLElement | undefined,
  clientId: string,
  onCredential: (resp: { credential: string }) => void
): void {
  const google = (window as any).google;
  if (!google?.accounts?.id || !el) return;
  google.accounts.id.initialize({ client_id: clientId, callback: onCredential });
  el.innerHTML = '';
  google.accounts.id.renderButton(el, {
    theme: 'filled_black',
    size: 'large',
    text: 'signin_with'
  });
}
