export const PROD_BASE = 'https://ifkash.dev';
export const DEV_BASE  = 'http://127.0.0.1:8787';

export function getApiBase(
  win: Window | undefined = typeof window !== 'undefined' ? window : undefined
) {
  if (!win) return PROD_BASE; // SSR safety: default prod
  const url = new URL(win.location.href);

  const qp = url.searchParams.get('api');
  if (qp === 'dev') {
    try { win.localStorage.setItem('api_env', 'dev'); } catch {}
    return DEV_BASE;
  }
  if (qp === 'prod') {
    try { win.localStorage.setItem('api_env', 'prod'); } catch {}
    return PROD_BASE;
  }

  try {
    const persisted = win.localStorage.getItem('api_env');
    if (persisted === 'dev') return DEV_BASE;
    if (persisted === 'prod') return PROD_BASE;
  } catch {}

  const host = win.location.hostname;
  if (host === 'localhost' || host === '127.0.0.1') {
    return DEV_BASE;
  }

  return PROD_BASE;
}
