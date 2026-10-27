// Small display helpers for the /vm page. Pure, no I/O.

/** "claude-sonnet-5-5" → "Sonnet 5.5". Unknown strings pass through unchanged. */
export function modelLabel(id: string | null | undefined): string {
  if (!id) return '';
  const m = id.match(/^claude-([a-z]+)-(\d+)-(\d+)/);
  return m ? `${m[1][0].toUpperCase()}${m[1].slice(1)} ${m[2]}.${m[3]}` : id;
}

export function humanBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return '–';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${i === 0 ? n : n.toFixed(n >= 10 ? 0 : 1)} ${units[i]}`;
}

export function humanUptime(sec: number): string {
  const d = Math.floor(sec / 86400);
  const h = Math.floor((sec % 86400) / 3600);
  const m = Math.floor((sec % 3600) / 60);
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

/** "3m ago" style age from epoch ms, epoch seconds, or an ISO string. */
export function relTime(t: number | string | null | undefined): string {
  if (t === null || t === undefined || t === '') return '';
  let ms = typeof t === 'number' ? t : Date.parse(t);
  if (typeof t === 'number' && t < 1e11) ms = t * 1000;
  if (!Number.isFinite(ms)) return '';
  const s = Math.max(0, Math.round((Date.now() - ms) / 1000));
  if (s < 45) return 'just now';
  if (s < 3600) return `${Math.round(s / 60)}m ago`;
  if (s < 86400) return `${Math.round(s / 3600)}h ago`;
  if (s < 86400 * 30) return `${Math.round(s / 86400)}d ago`;
  return new Date(ms).toLocaleDateString();
}

export function fmtDuration(ms: number): string {
  if (!Number.isFinite(ms)) return '';
  if (ms < 1000) return `${Math.round(ms)}ms`;
  const s = ms / 1000;
  if (s < 60) return `${s.toFixed(1)}s`;
  return `${Math.floor(s / 60)}m ${Math.round(s % 60)}s`;
}

export function fmtCost(usd: number | null | undefined): string {
  if (usd === null || usd === undefined) return '';
  return `$${usd < 1 ? usd.toFixed(3) : usd.toFixed(2)}`;
}

export function fmtMtime(t: number): string {
  const ms = t < 1e11 ? t * 1000 : t;
  return Number.isFinite(ms) && ms > 0 ? new Date(ms).toLocaleString() : '';
}

export function clip(s: string, n: number): string {
  s = s.replace(/\s+/g, ' ').trim();
  return s.length > n ? s.slice(0, n - 1) + '…' : s;
}
