import { writable } from 'svelte/store';

// Typed client for the VM agent (vm/API.md). Calls go cross-origin to the
// Cloudflare-Access-protected tunnel, so every request is "simple": GET, or
// POST with a text/plain body, which avoids a CORS preflight Access would block.

export const VM_PROD_BASE = 'https://claude.ifkash.dev';
export const VM_DEV_BASE = 'http://127.0.0.1:7690';

export function getVmBase(
  win: Window | undefined = typeof window !== 'undefined' ? window : undefined
) {
  if (!win) return VM_PROD_BASE;
  const url = new URL(win.location.href);

  const qp = url.searchParams.get('vm');
  if (qp === 'dev') {
    try { win.localStorage.setItem('vm_env', 'dev'); } catch {}
    return VM_DEV_BASE;
  }
  if (qp === 'prod') {
    try { win.localStorage.setItem('vm_env', 'prod'); } catch {}
    return VM_PROD_BASE;
  }

  try {
    const persisted = win.localStorage.getItem('vm_env');
    if (persisted === 'dev') return VM_DEV_BASE;
    if (persisted === 'prod') return VM_PROD_BASE;
  } catch {}

  const host = win.location.hostname;
  if (host === 'localhost' || host === '127.0.0.1') return VM_DEV_BASE;
  return VM_PROD_BASE;
}

/** Thrown when fetch fails at the network level, which is how a missing or
 *  expired Cloudflare Access session shows up (a cross-origin redirect). */
export class VmSignedOut extends Error {
  constructor(message = 'Not signed in to the VM console') {
    super(message);
    this.name = 'VmSignedOut';
  }
}

/** True once any call has failed as signed-out; the page shows the sign-in card. */
export const vmSignedOut = writable(false);

export function loginUrl(): string {
  return `${getVmBase()}/login?return=${encodeURIComponent(location.href)}`;
}

// ---- types ----------------------------------------------------------------

export type Health = { ok: boolean; email: string; host: string; claudeVersion: string; time: string | number };

export type DockerContainer = { name: string; image: string; status: string; ports: string };
export type SystemInfo = {
  hostname: string;
  uptimeSec: number;
  loadavg: number[];
  cpus: number;
  mem: { totalBytes: number; usedBytes: number };
  disk: { path: string; totalBytes: number; usedBytes: number };
  docker: DockerContainer[];
};

export type TmuxSession = {
  name: string;
  windows: number;
  attached: boolean | number;
  createdAt: number;
  cwd: string;
  command: string;
};
export type TmuxKey = 'Escape' | 'Enter' | 'Tab' | 'BTab' | 'C-c' | 'Up' | 'Down' | 'Left' | 'Right';

export type PermissionMode = 'plan' | 'acceptEdits' | 'auto' | 'bypassPermissions' | 'dontAsk' | 'manual';
export type RunStatus = 'running' | 'done' | 'error' | 'stopped';
export type Run = {
  runId: string;
  sessionId: string | null;
  cwd: string;
  prompt: string;
  permissionMode: PermissionMode;
  model: string | null;
  status: RunStatus;
  startedAt: number;
  endedAt: number | null;
  costUsd: number | null;
};
export type RunRequest = {
  prompt: string;
  cwd?: string;
  sessionId?: string;
  permissionMode?: PermissionMode;
  model?: string;
};

export type Project = { dir: string; cwd: string; sessions: number; updatedAt: number | string };
export type TranscriptSummary = { id: string; cwd: string; title: string; updatedAt: number | string; messages: number };
export type TranscriptMessage = {
  role: 'user' | 'assistant';
  kind: 'text' | 'thinking' | 'tool_use' | 'tool_result';
  text: string;
  tool: string | null;
  at: string;
};

export type FsEntry = { name: string; type: 'dir' | 'file' | 'link'; size: number; mtime: number };
export type FsDir = { type: 'dir'; path: string; parent: string | null; entries: FsEntry[] };
export type FsFile = {
  type: 'file';
  path: string;
  parent: string | null;
  size: number;
  mtime: number;
  binary: boolean;
  truncated: boolean;
  text: string | null;
};
export type FsResult = FsDir | FsFile;

// ---- transport ------------------------------------------------------------

async function request<T>(path: string, body?: unknown): Promise<T> {
  const init: RequestInit = { credentials: 'include' };
  if (body !== undefined) {
    init.method = 'POST';
    init.headers = { 'Content-Type': 'text/plain;charset=UTF-8' };
    init.body = JSON.stringify(body);
  }
  let res: Response;
  try {
    res = await fetch(getVmBase() + path, init);
  } catch (e) {
    if (e instanceof TypeError) {
      vmSignedOut.set(true);
      throw new VmSignedOut();
    }
    throw e;
  }
  let data: any = null;
  try { data = await res.json(); } catch {}
  if (!res.ok) throw new Error((data && data.error) || `Request failed (${res.status})`);
  return data as T;
}

const q = (params: Record<string, string | number | undefined>) => {
  const sp = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) if (v !== undefined && v !== '') sp.set(k, String(v));
  const s = sp.toString();
  return s ? `?${s}` : '';
};

export const vmApi = {
  health: () => request<Health>('/api/health'),
  system: () => request<SystemInfo>('/api/system'),
  dockerLogs: (name: string, lines = 200) =>
    request<{ text: string }>('/api/docker/logs' + q({ name, lines })),

  listTmux: () => request<{ sessions: TmuxSession[] }>('/api/tmux'),
  newTmux: (b: { name: string; cwd?: string; start: 'claude' | 'shell' }) =>
    request<{ ok: true; name: string }>('/api/tmux/new', b),
  killTmux: (name: string) => request<{ ok: true }>('/api/tmux/kill', { name }),
  sendTmux: (b: { name: string; text?: string; enter?: boolean; key?: TmuxKey }) =>
    request<{ ok: true }>('/api/tmux/send', b),
  captureTmux: (name: string, lines = 300) =>
    request<{ text: string }>('/api/tmux/capture' + q({ name, lines })),

  createRun: (b: RunRequest) => request<{ runId: string }>('/api/runs', b),
  listRuns: () => request<{ runs: Run[] }>('/api/runs'),
  stopRun: (runId: string) => request<{ ok: true }>(`/api/runs/${encodeURIComponent(runId)}/stop`, {}),

  listProjects: () => request<{ projects: Project[] }>('/api/projects'),
  listSessions: (cwd?: string) => request<{ sessions: TranscriptSummary[] }>('/api/sessions' + q({ cwd })),
  getSession: (id: string) =>
    request<{ id: string; cwd: string; messages: TranscriptMessage[] }>(`/api/sessions/${encodeURIComponent(id)}`),

  fs: (path?: string) => request<FsResult>('/api/fs' + q({ path }))
};

// ---- streams --------------------------------------------------------------

/** Server-sent events for a run. The agent replays earlier events on connect. */
export function openRunStream(runId: string): EventSource {
  return new EventSource(`${getVmBase()}/api/runs/${encodeURIComponent(runId)}/events`, {
    withCredentials: true
  });
}

export function ttyUrl(session: string): string {
  const base = getVmBase().replace(/^http/, 'ws');
  return `${base}/tty/ws?arg=${encodeURIComponent(session)}`;
}

/** WebSocket to ttyd, which attaches (or creates) the named tmux session. */
export function openTerminalSocket(session: string): WebSocket {
  const ws = new WebSocket(ttyUrl(session), ['tty']);
  ws.binaryType = 'arraybuffer';
  return ws;
}
