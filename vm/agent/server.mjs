// VM agent: the single origin behind the Cloudflare Tunnel at claude.ifkash.dev.
//
// Serves the JSON API in vm/API.md for the /vm page on ifkash.dev, and proxies
// /tty/* (HTTP and WebSocket) to ttyd on a Unix socket. It listens on loopback
// only; cloudflared is the only thing that reaches it from outside.
//
// Every request must carry a Cloudflare Access JWT signed for ACCESS_AUD and
// issued to OWNER_EMAIL, even though cloudflared checks it too. POSTs and
// WebSocket upgrades must also come from an allowed Origin, because the Access
// cookie alone doesn't stop another site from sending requests on the owner's
// behalf.
//
// No dependencies: Node 22 standard library only.

import http from 'node:http';
import net from 'node:net';
import crypto from 'node:crypto';
import os from 'node:os';
import path from 'node:path';
import fs from 'node:fs/promises';
import readline from 'node:readline';
import { createReadStream } from 'node:fs';
import { spawn, execFile } from 'node:child_process';

const HOME = os.homedir();
const env = process.env;
const PORT = Number(env.PORT || 7690);
const HOST = env.HOST || '127.0.0.1';
const LOCAL_DEV = ['1', 'true'].includes(env.LOCAL_DEV || '');
const OWNER_EMAIL = (env.OWNER_EMAIL || '').trim().toLowerCase();
const TEAM_DOMAIN = (env.ACCESS_TEAM_DOMAIN || '').replace(/^https:\/\//, '').replace(/\/$/, '');
const ACCESS_AUD = (env.ACCESS_AUD || '').split(',').map((a) => a.trim()).filter(Boolean);
const ALLOWED_ORIGINS = new Set(
  (env.ALLOWED_ORIGINS || 'https://ifkash.dev').split(',').map((o) => o.trim()).filter(Boolean)
);
const FALLBACK_RETURN = 'https://ifkash.dev/vm';
const TTYD_SOCKET = env.TTYD_SOCKET || path.join(HOME, '.local/run/claude-remote/ttyd.sock');
const CLAUDE_BIN = env.CLAUDE_BIN || path.join(HOME, '.local/bin/claude');
const PROJECTS_DIR = path.join(HOME, '.claude/projects');

const MAX_BODY_BYTES = 1 << 20;
const MAX_FILE_BYTES = 512 * 1024;
const MAX_RUNNING = 3;
const MAX_RUNS_KEPT = 50;
const MAX_EVENTS_PER_RUN = 5000;
const CERTS_TTL_MS = 60 * 60 * 1000;
const CERTS_MIN_REFETCH_MS = 5 * 60 * 1000;
const CLOCK_SKEW_SEC = 30;

const SESSION_NAME = /^[A-Za-z0-9_-]{1,32}$/;
const SESSION_ID = /^[0-9a-f-]{36}$/;
const PERMISSION_MODES = new Set(['plan', 'acceptEdits', 'auto', 'bypassPermissions', 'dontAsk', 'manual']);
const MODELS = new Set(['sonnet', 'opus', 'haiku']);
const TMUX_KEYS = new Set(['Escape', 'Enter', 'Tab', 'BTab', 'C-c', 'Up', 'Down', 'Left', 'Right']);

// Child processes need ~/.local/bin for claude, whatever PATH systemd gave us.
const CHILD_ENV = { ...env, PATH: `${path.join(HOME, '.local/bin')}:${env.PATH || '/usr/bin:/bin'}` };

class HttpError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

// ---------------------------------------------------------------- auth

let certCache = null; // { fetchedAt, keys }

async function signingKey(kid) {
  const now = Date.now();
  const cached = certCache;
  if (cached) {
    const key = cached.keys.find((k) => k.kid === kid);
    const age = now - cached.fetchedAt;
    if (key && age < CERTS_TTL_MS) return key;
    if (!key && age < CERTS_MIN_REFETCH_MS) return null;
  }
  try {
    const res = await fetch(`https://${TEAM_DOMAIN}/cdn-cgi/access/certs`);
    if (!res.ok) throw new Error(`certs ${res.status}`);
    const { keys } = await res.json();
    certCache = { fetchedAt: now, keys };
    return keys.find((k) => k.kid === kid) || null;
  } catch (err) {
    console.error('access certs fetch failed:', err.message);
    // Keep using stale keys if the refresh fails.
    return cached?.keys.find((k) => k.kid === kid) || null;
  }
}

/** Returns the owner's email for a valid Access JWT, or null. */
async function verifyAccessJwt(token) {
  const parts = token.split('.');
  if (parts.length !== 3) return null;
  const [h, p, s] = parts;
  let header, claims;
  try {
    header = JSON.parse(Buffer.from(h, 'base64url'));
    claims = JSON.parse(Buffer.from(p, 'base64url'));
  } catch {
    return null;
  }
  if (header.alg !== 'RS256' || typeof header.kid !== 'string') return null;
  const jwk = await signingKey(header.kid);
  if (!jwk) return null;
  let valid = false;
  try {
    const key = crypto.createPublicKey({ key: jwk, format: 'jwk' });
    valid = crypto.verify('RSA-SHA256', Buffer.from(`${h}.${p}`), key, Buffer.from(s, 'base64url'));
  } catch {
    return null;
  }
  if (!valid) return null;

  const now = Math.floor(Date.now() / 1000);
  if (claims.iss !== `https://${TEAM_DOMAIN}`) return null;
  if (typeof claims.exp !== 'number' || claims.exp + CLOCK_SKEW_SEC <= now) return null;
  if (typeof claims.nbf === 'number' && claims.nbf - CLOCK_SKEW_SEC > now) return null;
  const auds = Array.isArray(claims.aud) ? claims.aud : [claims.aud];
  if (!auds.some((a) => ACCESS_AUD.includes(a))) return null;
  const email = typeof claims.email === 'string' ? claims.email : '';
  return email.toLowerCase() === OWNER_EMAIL ? email : null;
}

async function authenticate(req) {
  if (LOCAL_DEV) return 'local-dev@localhost';
  if (!OWNER_EMAIL || !TEAM_DOMAIN || ACCESS_AUD.length === 0) return null;
  const token = req.headers['cf-access-jwt-assertion'];
  if (typeof token !== 'string' || !token) return null;
  return verifyAccessJwt(token);
}

function originAllowed(req) {
  const origin = req.headers.origin;
  if (!origin) return false;
  if (ALLOWED_ORIGINS.has(origin)) return true;
  // ttyd's own page, opened directly on the tunnel hostname.
  return origin === `https://${req.headers.host}`;
}

// ---------------------------------------------------------------- helpers

function run(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    execFile(
      cmd,
      args,
      { timeout: 15000, maxBuffer: 8 << 20, env: CHILD_ENV, ...opts },
      (err, stdout, stderr) => {
        if (err) {
          err.stderr = stderr;
          reject(err);
        } else {
          resolve({ stdout, stderr });
        }
      }
    );
  });
}

async function readJson(req) {
  let size = 0;
  const chunks = [];
  for await (const chunk of req) {
    size += chunk.length;
    if (size > MAX_BODY_BYTES) throw new HttpError(413, 'body too large');
    chunks.push(chunk);
  }
  if (size === 0) return {};
  try {
    const body = JSON.parse(Buffer.concat(chunks).toString('utf8'));
    if (body && typeof body === 'object' && !Array.isArray(body)) return body;
  } catch {}
  throw new HttpError(400, 'body must be a JSON object');
}

function sendJson(res, status, body) {
  res.writeHead(status, { 'Content-Type': 'application/json', 'Cache-Control': 'no-store' });
  res.end(JSON.stringify(body));
}

function setCors(req, res) {
  const origin = req.headers.origin;
  if (origin && ALLOWED_ORIGINS.has(origin)) {
    res.setHeader('Access-Control-Allow-Origin', origin);
    res.setHeader('Access-Control-Allow-Credentials', 'true');
    res.setHeader('Vary', 'Origin');
  }
}

/** Resolves a path (default home) and requires it to stay inside home after symlinks. */
async function resolveInHome(p) {
  const realHome = await fs.realpath(HOME);
  const target = path.resolve(HOME, typeof p === 'string' && p ? p : HOME);
  let real;
  try {
    real = await fs.realpath(target);
  } catch {
    throw new HttpError(404, 'path not found');
  }
  if (real !== realHome && !real.startsWith(realHome + path.sep)) {
    throw new HttpError(403, 'path is outside the home directory');
  }
  return real;
}

function sessionName(name) {
  if (typeof name !== 'string' || !SESSION_NAME.test(name)) {
    throw new HttpError(400, 'invalid session name');
  }
  return name;
}

// ---------------------------------------------------------------- system

let claudeVersion = null;
async function getClaudeVersion() {
  if (claudeVersion) return claudeVersion;
  try {
    // `claude --version` prints "2.1.311 (Claude Code)"; keep the number.
    claudeVersion = (await run(CLAUDE_BIN, ['--version'])).stdout.trim().replace(/\s*\(Claude Code\)$/, '');
  } catch {
    claudeVersion = 'unknown';
  }
  return claudeVersion;
}

async function dockerContainers() {
  try {
    const { stdout } = await run('docker', ['ps', '--format', '{{json .}}']);
    return stdout
      .split('\n')
      .filter(Boolean)
      .map((line) => JSON.parse(line))
      .map((c) => ({ name: c.Names, image: c.Image, status: c.Status, ports: c.Ports }));
  } catch {
    return [];
  }
}

async function systemInfo() {
  const meminfo = await fs.readFile('/proc/meminfo', 'utf8').catch(() => '');
  const kb = (field) => Number(meminfo.match(new RegExp(`^${field}:\\s+(\\d+)`, 'm'))?.[1] || 0) * 1024;
  const totalMem = kb('MemTotal') || os.totalmem();
  const availMem = kb('MemAvailable') || os.freemem();
  const disk = await fs.statfs('/');
  return {
    hostname: os.hostname(),
    uptimeSec: Math.round(os.uptime()),
    loadavg: os.loadavg(),
    cpus: os.cpus().length,
    mem: { totalBytes: totalMem, usedBytes: totalMem - availMem },
    disk: {
      path: '/',
      totalBytes: disk.blocks * disk.bsize,
      usedBytes: (disk.blocks - disk.bfree) * disk.bsize,
    },
    docker: await dockerContainers(),
  };
}

async function dockerLogs(name, lines) {
  const containers = await dockerContainers();
  if (!containers.some((c) => c.name === name)) throw new HttpError(404, 'no such running container');
  const { stdout, stderr } = await run('docker', ['logs', '--tail', String(lines), name]);
  return stdout + stderr;
}

// ---------------------------------------------------------------- tmux

async function tmuxSessions() {
  const format = [
    '#{session_name}',
    '#{session_windows}',
    '#{session_attached}',
    '#{session_created}',
    '#{pane_current_path}',
    '#{pane_current_command}',
  ].join('\t');
  try {
    const { stdout } = await run('tmux', ['list-sessions', '-F', format]);
    return stdout
      .split('\n')
      .filter(Boolean)
      .map((line) => {
        const [name, windows, attached, created, cwd, command] = line.split('\t');
        return {
          name,
          windows: Number(windows),
          attached: Number(attached) > 0,
          createdAt: Number(created) * 1000,
          cwd,
          command,
        };
      });
  } catch {
    return []; // No tmux server running.
  }
}

async function tmux(args) {
  try {
    return await run('tmux', args);
  } catch (err) {
    throw new HttpError(400, (err.stderr || err.message).trim());
  }
}

// ---------------------------------------------------------------- runs

const runs = new Map(); // runId -> run

function publicRun(r) {
  return {
    runId: r.runId,
    sessionId: r.sessionId,
    cwd: r.cwd,
    prompt: r.prompt,
    permissionMode: r.permissionMode,
    model: r.model,
    status: r.status,
    startedAt: r.startedAt,
    endedAt: r.endedAt,
    costUsd: r.costUsd,
  };
}

function pushEvent(r, event) {
  if (r.events.length < MAX_EVENTS_PER_RUN) r.events.push(event);
  for (const send of r.listeners) send(event);
}

function pruneRuns() {
  const finished = [...runs.values()]
    .filter((r) => r.status !== 'running')
    .sort((a, b) => a.startedAt - b.startedAt);
  while (runs.size > MAX_RUNS_KEPT && finished.length) runs.delete(finished.shift().runId);
}

async function startRun(body) {
  const prompt = typeof body.prompt === 'string' ? body.prompt.trim() : '';
  if (!prompt) throw new HttpError(400, 'prompt is required');
  const permissionMode = body.permissionMode ?? 'acceptEdits';
  if (!PERMISSION_MODES.has(permissionMode)) throw new HttpError(400, 'invalid permissionMode');
  const model = body.model || null;
  if (model && !MODELS.has(model)) throw new HttpError(400, 'invalid model');
  const sessionId = body.sessionId || null;
  if (sessionId && !SESSION_ID.test(sessionId)) throw new HttpError(400, 'invalid sessionId');
  const cwd = await resolveInHome(body.cwd);
  const running = [...runs.values()].filter((r) => r.status === 'running').length;
  if (running >= MAX_RUNNING) throw new HttpError(429, `at most ${MAX_RUNNING} runs at once`);

  const args = ['-p', '--output-format', 'stream-json', '--verbose', '--permission-mode', permissionMode];
  if (model) args.push('--model', model);
  if (sessionId) args.push('--resume', sessionId);

  // The prompt goes in on stdin so text starting with "-" can't become a flag.
  const child = spawn(CLAUDE_BIN, args, { cwd, env: CHILD_ENV, stdio: ['pipe', 'pipe', 'pipe'] });
  const r = {
    runId: crypto.randomUUID(),
    sessionId,
    cwd,
    prompt,
    permissionMode,
    model,
    status: 'running',
    startedAt: Date.now(),
    endedAt: null,
    costUsd: null,
    events: [],
    listeners: new Set(),
    child,
    stopping: false,
  };
  runs.set(r.runId, r);
  pruneRuns();

  child.stdin.end(prompt);
  readline.createInterface({ input: child.stdout }).on('line', (line) => {
    if (!line.trim()) return;
    let event;
    try {
      event = JSON.parse(line);
    } catch {
      event = { type: 'agent.stdout', text: line };
    }
    if (event.session_id) r.sessionId = event.session_id;
    if (event.type === 'result' && typeof event.total_cost_usd === 'number') r.costUsd = event.total_cost_usd;
    pushEvent(r, event);
  });
  readline.createInterface({ input: child.stderr }).on('line', (text) => {
    pushEvent(r, { type: 'agent.stderr', text });
  });
  child.on('error', (err) => pushEvent(r, { type: 'agent.stderr', text: err.message }));
  child.on('close', (code) => {
    r.status = r.stopping ? 'stopped' : code === 0 ? 'done' : 'error';
    r.endedAt = Date.now();
    r.child = null;
    pushEvent(r, { type: 'agent.exit', code, status: r.status });
  });
  return r;
}

function stopRun(r) {
  if (!r.child) return;
  r.stopping = true;
  r.child.kill('SIGINT');
  const child = r.child;
  setTimeout(() => child.exitCode === null && child.kill('SIGKILL'), 5000).unref();
}

function streamRun(req, res, r) {
  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-store',
    'X-Accel-Buffering': 'no',
  });
  const send = (event) => {
    res.write(`data: ${JSON.stringify(event)}\n\n`);
    if (event.type === 'agent.exit') res.end();
  };
  for (const event of r.events) send(event);
  if (r.status !== 'running') {
    if (!r.events.some((e) => e.type === 'agent.exit')) res.end();
    return;
  }
  r.listeners.add(send);
  const heartbeat = setInterval(() => res.write(': ping\n\n'), 20000);
  const cleanup = () => {
    clearInterval(heartbeat);
    r.listeners.delete(send);
  };
  res.on('close', cleanup);
  res.on('finish', cleanup);
}

// ---------------------------------------------------------------- transcripts

const summaryCache = new Map(); // file -> { mtimeMs, size, summary }

function userPromptText(entry) {
  if (entry.type !== 'user' || entry.isMeta) return null;
  const content = entry.message?.content;
  let text = null;
  if (typeof content === 'string') text = content;
  else if (Array.isArray(content)) text = content.find((b) => b.type === 'text')?.text ?? null;
  if (!text || text.startsWith('<')) return null; // Slash commands and caveats.
  return text;
}

async function summarize(file) {
  const stat = await fs.stat(file);
  const hit = summaryCache.get(file);
  if (hit && hit.mtimeMs === stat.mtimeMs && hit.size === stat.size) return hit.summary;

  let cwd = null;
  let title = null;
  let messages = 0;
  const lines = readline.createInterface({ input: createReadStream(file), crlfDelay: Infinity });
  for await (const line of lines) {
    let entry;
    try {
      entry = JSON.parse(line);
    } catch {
      continue;
    }
    if (!cwd && entry.cwd) cwd = entry.cwd;
    if (entry.type === 'user' || entry.type === 'assistant') messages += 1;
    if (!title) {
      const text = userPromptText(entry);
      if (text) title = text.replace(/\s+/g, ' ').trim().slice(0, 120);
    }
  }
  const summary = {
    id: path.basename(file, '.jsonl'),
    cwd,
    title: title || '(no prompt)',
    updatedAt: stat.mtimeMs,
    messages,
  };
  summaryCache.set(file, { mtimeMs: stat.mtimeMs, size: stat.size, summary });
  return summary;
}

async function transcriptFiles() {
  let dirs = [];
  try {
    dirs = await fs.readdir(PROJECTS_DIR, { withFileTypes: true });
  } catch {
    return [];
  }
  const files = [];
  for (const d of dirs) {
    if (!d.isDirectory()) continue;
    const dir = path.join(PROJECTS_DIR, d.name);
    for (const name of await fs.readdir(dir).catch(() => [])) {
      if (name.endsWith('.jsonl') && SESSION_ID.test(name.slice(0, -6))) {
        files.push({ dir: d.name, file: path.join(dir, name) });
      }
    }
  }
  return files;
}

async function listSessions(cwdFilter) {
  const summaries = [];
  for (const { file } of await transcriptFiles()) {
    const s = await summarize(file).catch(() => null);
    if (s && (!cwdFilter || s.cwd === cwdFilter)) summaries.push(s);
  }
  return summaries.sort((a, b) => b.updatedAt - a.updatedAt).slice(0, 100);
}

async function listProjects() {
  const byDir = new Map();
  for (const { dir, file } of await transcriptFiles()) {
    const s = await summarize(file).catch(() => null);
    if (!s) continue;
    const p = byDir.get(dir) || { dir, cwd: null, sessions: 0, updatedAt: 0 };
    p.cwd ||= s.cwd;
    p.sessions += 1;
    p.updatedAt = Math.max(p.updatedAt, s.updatedAt);
    byDir.set(dir, p);
  }
  return [...byDir.values()].sort((a, b) => b.updatedAt - a.updatedAt);
}

function clip(text, max) {
  return text.length > max ? `${text.slice(0, max)}\n…` : text;
}

function toolResultText(content) {
  if (typeof content === 'string') return content;
  if (Array.isArray(content)) {
    return content.map((b) => (b.type === 'text' ? b.text : `[${b.type}]`)).join('\n');
  }
  return '';
}

async function readSession(id) {
  if (!SESSION_ID.test(id)) throw new HttpError(400, 'invalid session id');
  const match = (await transcriptFiles()).find(({ file }) => path.basename(file) === `${id}.jsonl`);
  if (!match) throw new HttpError(404, 'session not found');

  let cwd = null;
  const messages = [];
  const lines = readline.createInterface({ input: createReadStream(match.file), crlfDelay: Infinity });
  for await (const line of lines) {
    let entry;
    try {
      entry = JSON.parse(line);
    } catch {
      continue;
    }
    if (!cwd && entry.cwd) cwd = entry.cwd;
    const at = entry.timestamp || null;
    if (entry.type === 'user') {
      const prompt = userPromptText(entry);
      if (prompt) messages.push({ role: 'user', kind: 'text', text: prompt, tool: null, at });
      const content = entry.message?.content;
      if (Array.isArray(content)) {
        for (const b of content) {
          if (b.type === 'tool_result') {
            messages.push({
              role: 'user',
              kind: 'tool_result',
              text: clip(toolResultText(b.content), 4000),
              tool: null,
              at,
            });
          }
        }
      }
    } else if (entry.type === 'assistant') {
      for (const b of entry.message?.content || []) {
        if (b.type === 'text' && b.text) {
          messages.push({ role: 'assistant', kind: 'text', text: b.text, tool: null, at });
        } else if (b.type === 'thinking' && b.thinking) {
          messages.push({ role: 'assistant', kind: 'thinking', text: b.thinking, tool: null, at });
        } else if (b.type === 'tool_use') {
          messages.push({
            role: 'assistant',
            kind: 'tool_use',
            text: clip(JSON.stringify(b.input, null, 2), 2000),
            tool: b.name,
            at,
          });
        }
      }
    }
  }
  return { id, cwd, messages };
}

// ---------------------------------------------------------------- files

async function readPath(p) {
  const realHome = await fs.realpath(HOME);
  const real = await resolveInHome(p);
  const parent = real === realHome ? null : path.dirname(real);
  const stat = await fs.stat(real);
  if (stat.isDirectory()) {
    const dirents = (await fs.readdir(real, { withFileTypes: true })).slice(0, 2000);
    const entries = await Promise.all(
      dirents.map(async (d) => {
        const st = await fs.lstat(path.join(real, d.name)).catch(() => null);
        return {
          name: d.name,
          type: d.isSymbolicLink() ? 'link' : d.isDirectory() ? 'dir' : 'file',
          size: st?.size ?? 0,
          mtime: st?.mtimeMs ?? 0,
        };
      })
    );
    // Directories first, then by name.
    entries.sort((a, b) => (b.type === 'dir') - (a.type === 'dir') || a.name.localeCompare(b.name));
    return { type: 'dir', path: real, parent, entries };
  }
  const handle = await fs.open(real, 'r');
  try {
    const buf = Buffer.alloc(Math.min(stat.size, MAX_FILE_BYTES));
    const { bytesRead } = await handle.read(buf, 0, buf.length, 0);
    const data = buf.subarray(0, bytesRead);
    const binary = data.subarray(0, 8192).includes(0);
    return {
      type: 'file',
      path: real,
      parent,
      size: stat.size,
      mtime: stat.mtimeMs,
      binary,
      truncated: stat.size > MAX_FILE_BYTES,
      text: binary ? null : data.toString('utf8'),
    };
  } finally {
    await handle.close();
  }
}

// ---------------------------------------------------------------- ttyd proxy

function proxyTtyHttp(req, res) {
  const upstream = http.request(
    { socketPath: TTYD_SOCKET, path: req.url, method: req.method, headers: req.headers },
    (up) => {
      res.writeHead(up.statusCode || 502, up.headers);
      up.pipe(res);
    }
  );
  upstream.on('error', () => {
    if (!res.headersSent) sendJson(res, 502, { error: 'terminal is not running' });
    else res.destroy();
  });
  req.pipe(upstream);
}

function proxyTtyUpgrade(req, socket, head) {
  const upstream = net.connect(TTYD_SOCKET, () => {
    const headers = Object.entries(req.headers)
      .map(([k, v]) => `${k}: ${Array.isArray(v) ? v.join(', ') : v}`)
      .join('\r\n');
    upstream.write(`${req.method} ${req.url} HTTP/1.1\r\n${headers}\r\n\r\n`);
    if (head?.length) upstream.write(head);
    upstream.pipe(socket);
    socket.pipe(upstream);
  });
  const close = () => {
    upstream.destroy();
    socket.destroy();
  };
  upstream.on('error', close);
  socket.on('error', close);
  upstream.on('close', close);
  socket.on('close', close);
}

// ---------------------------------------------------------------- routes

async function handleApi(req, res, url, email) {
  const p = url.pathname;
  const q = url.searchParams;
  const method = req.method;
  const body = method === 'POST' ? await readJson(req) : null;
  const lines = (fallback) => Math.min(Math.max(Number(q.get('lines')) || fallback, 1), 5000);

  if (method === 'GET' && p === '/api/health') {
    return sendJson(res, 200, {
      ok: true,
      email,
      host: os.hostname(),
      claudeVersion: await getClaudeVersion(),
      time: new Date().toISOString(),
    });
  }
  if (method === 'GET' && p === '/api/system') return sendJson(res, 200, await systemInfo());
  if (method === 'GET' && p === '/api/docker/logs') {
    return sendJson(res, 200, { text: await dockerLogs(q.get('name') || '', lines(200)) });
  }

  if (method === 'GET' && p === '/api/tmux') return sendJson(res, 200, { sessions: await tmuxSessions() });
  if (method === 'POST' && p === '/api/tmux/new') {
    const name = sessionName(body.name);
    const cwd = await resolveInHome(body.cwd);
    const args = ['new-session', '-d', '-s', name, '-c', cwd];
    if ((body.start ?? 'claude') === 'claude') args.push(CLAUDE_BIN);
    await tmux(args);
    return sendJson(res, 200, { ok: true, name });
  }
  if (method === 'POST' && p === '/api/tmux/kill') {
    await tmux(['kill-session', '-t', `=${sessionName(body.name)}`]);
    return sendJson(res, 200, { ok: true });
  }
  if (method === 'POST' && p === '/api/tmux/send') {
    const target = `=${sessionName(body.name)}:`;
    if (body.key !== undefined && !TMUX_KEYS.has(body.key)) throw new HttpError(400, 'invalid key');
    if (typeof body.text === 'string' && body.text) await tmux(['send-keys', '-t', target, '-l', body.text]);
    if (body.key) await tmux(['send-keys', '-t', target, body.key]);
    if (body.enter) await tmux(['send-keys', '-t', target, 'Enter']);
    return sendJson(res, 200, { ok: true });
  }
  if (method === 'GET' && p === '/api/tmux/capture') {
    const target = `=${sessionName(q.get('name'))}:`;
    const { stdout } = await tmux(['capture-pane', '-p', '-J', '-t', target, '-S', `-${lines(300)}`]);
    return sendJson(res, 200, { text: stdout.replace(/\s+$/, '') });
  }

  if (method === 'GET' && p === '/api/runs') {
    const list = [...runs.values()].sort((a, b) => b.startedAt - a.startedAt).map(publicRun);
    return sendJson(res, 200, { runs: list });
  }
  if (method === 'POST' && p === '/api/runs') {
    const r = await startRun(body);
    return sendJson(res, 200, { runId: r.runId });
  }
  const runMatch = p.match(/^\/api\/runs\/([0-9a-f-]{36})\/(events|stop)$/);
  if (runMatch) {
    const r = runs.get(runMatch[1]);
    if (!r) throw new HttpError(404, 'run not found');
    if (method === 'GET' && runMatch[2] === 'events') return streamRun(req, res, r);
    if (method === 'POST' && runMatch[2] === 'stop') {
      stopRun(r);
      return sendJson(res, 200, { ok: true });
    }
  }

  if (method === 'GET' && p === '/api/projects') return sendJson(res, 200, { projects: await listProjects() });
  if (method === 'GET' && p === '/api/sessions') {
    return sendJson(res, 200, { sessions: await listSessions(q.get('cwd')) });
  }
  const sessionMatch = p.match(/^\/api\/sessions\/([^/]+)$/);
  if (method === 'GET' && sessionMatch) return sendJson(res, 200, await readSession(sessionMatch[1]));

  if (method === 'GET' && p === '/api/fs') return sendJson(res, 200, await readPath(q.get('path')));

  throw new HttpError(404, 'not found');
}

function safeReturn(value) {
  try {
    const u = new URL(value);
    if (ALLOWED_ORIGINS.has(u.origin)) return u.href;
  } catch {}
  return FALLBACK_RETURN;
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, 'http://localhost');
  try {
    setCors(req, res);
    if (req.method === 'OPTIONS') {
      res.writeHead(204);
      return res.end();
    }
    const email = await authenticate(req);
    if (!email) throw new HttpError(401, 'unauthorized');
    if (req.method !== 'GET' && !originAllowed(req)) throw new HttpError(403, 'origin not allowed');

    if (url.pathname === '/login') {
      res.writeHead(302, { Location: safeReturn(url.searchParams.get('return')) });
      return res.end();
    }
    if (url.pathname === '/tty' || url.pathname.startsWith('/tty/')) return proxyTtyHttp(req, res);
    if (url.pathname.startsWith('/api/')) return await handleApi(req, res, url, email);
    throw new HttpError(404, 'not found');
  } catch (err) {
    const status = err instanceof HttpError ? err.status : 500;
    if (status === 500) console.error(err);
    if (!res.headersSent) sendJson(res, status, { error: status === 500 ? 'internal error' : err.message });
    else res.end();
  }
});

server.on('upgrade', async (req, socket, head) => {
  const reject = (status) => {
    socket.end(`HTTP/1.1 ${status} ${http.STATUS_CODES[status]}\r\nConnection: close\r\n\r\n`);
  };
  const url = new URL(req.url, 'http://localhost');
  if (!url.pathname.startsWith('/tty/')) return reject(404);
  if (!originAllowed(req)) return reject(403);
  if (!(await authenticate(req))) return reject(401);
  proxyTtyUpgrade(req, socket, head);
});

server.listen(PORT, HOST, () => {
  const mode = LOCAL_DEV ? 'LOCAL_DEV (auth bypassed)' : `Access team ${TEAM_DOMAIN || '(unset)'}`;
  console.log(`vm-agent listening on http://${HOST}:${PORT} — ${mode}`);
  if (!LOCAL_DEV && (!OWNER_EMAIL || !TEAM_DOMAIN || ACCESS_AUD.length === 0)) {
    console.warn('OWNER_EMAIL, ACCESS_TEAM_DOMAIN, or ACCESS_AUD is empty: every request gets 401.');
  }
});
