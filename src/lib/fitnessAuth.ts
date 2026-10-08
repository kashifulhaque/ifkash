// Shared Google Identity helpers for the fitness pages.
// The app authenticates with a Google ID token (a JWT) used directly as the
// bearer. Those tokens expire ~1 hour after issue, and there is no refresh
// token, so sessions silently die mid-use. To soften that, we read the token's
// `exp` claim and re-trigger Google One Tap shortly before it lapses — with an
// active Google session this re-issues a fresh credential, usually with no or
// one tap, instead of a surprise 401 later.

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

/**
 * Schedule a silent-ish credential refresh ~1 min before the current token
 * expires. Returns the timer handle so callers can clear/reschedule. Re-arms
 * itself each time it's called (clear the previous handle first).
 */
export function scheduleTokenRefresh(
  token: string | null,
  clientId: string,
  onCredential: (resp: { credential: string }) => void
): ReturnType<typeof setTimeout> | undefined {
  const exp = tokenExpMs(token);
  if (!exp || !clientId) return undefined;
  const lead = 60_000; // refresh a minute before expiry
  const delay = Math.max(1000, exp - Date.now() - lead);
  return setTimeout(() => {
    const id = googleId();
    if (!id) return;
    id.initialize({ client_id: clientId, callback: onCredential, auto_select: true });
    id.prompt();
  }, delay);
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
