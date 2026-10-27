# VM agent API

The VM agent (`vm/agent/server.mjs`) runs on the VM and is the only origin behind
the Cloudflare Tunnel at `https://claude.ifkash.dev`. The `/vm` page on
`ifkash.dev` calls it cross-origin. For local development, the agent runs at
`http://127.0.0.1:7690` with `LOCAL_DEV=1`.

## Calling conventions

- Every request uses `credentials: 'include'` so the browser sends the
  Cloudflare Access cookie for `claude.ifkash.dev`.
- Requests use only `GET`, or `POST` with a JSON string body sent as
  `Content-Type: text/plain;charset=UTF-8`. This keeps every request "simple" in
  CORS terms, so the browser never sends a preflight that Access would block.
- Responses are JSON. Errors are `{ "error": "message" }` with a 4xx or 5xx
  status.
- If the Access session is missing or expired, Access answers with a
  cross-origin redirect, so `fetch` rejects with a network error (a
  `TypeError`). Treat that as "signed out" and send the user to
  `GET /login?return=<current page URL>`, as a full-page navigation.
- The agent rejects any `POST` whose `Origin` isn't in its allowlist
  (`https://ifkash.dev` and the local dev origins).

## Endpoints

### Identity and login

| Method | Path | Response |
|---|---|---|
| GET | `/api/health` | `{ ok, email, host, claudeVersion, time }` |
| GET | `/login?return=URL` | `302` to `URL` if its origin is allowed, otherwise to `https://ifkash.dev/vm` |

### System

`GET /api/system` returns the following:

```json
{
  "hostname": "ifkash",
  "uptimeSec": 123456,
  "loadavg": [0.12, 0.2, 0.3],
  "cpus": 2,
  "mem": { "totalBytes": 0, "usedBytes": 0 },
  "disk": { "path": "/", "totalBytes": 0, "usedBytes": 0 },
  "docker": [{ "name": "caddy", "image": "caddy:x", "status": "Up 3 days", "ports": "0.0.0.0:80->80/tcp" }]
}
```

`GET /api/docker/logs?name=NAME&lines=200` returns `{ text }`, the last lines of a
container's logs. `NAME` must be a running container.

### tmux sessions

Interactive Claude Code sessions run inside tmux, so they survive disconnects.

| Method | Path | Body | Response |
|---|---|---|---|
| GET | `/api/tmux` | — | `{ sessions: [{ name, windows, attached, createdAt, cwd, command }] }` |
| POST | `/api/tmux/new` | `{ name, cwd?, start: "claude" \| "shell" }` | `{ ok: true, name }` |
| POST | `/api/tmux/kill` | `{ name }` | `{ ok: true }` |
| POST | `/api/tmux/send` | `{ name, text?, enter?, key? }` | `{ ok: true }` |
| GET | `/api/tmux/capture?name=NAME&lines=300` | — | `{ text }` |

- `name` matches `^[A-Za-z0-9_-]{1,32}$`. `createdAt` is milliseconds since the
  epoch. `command` is the running program in the active pane, for example
  `claude` or `zsh`.
- `cwd` defaults to the home directory and must be inside it.
- `send` types `text` literally, then presses **Enter** if `enter` is `true`.
  `key` sends one special key, from this list: `Escape`, `Enter`, `Tab`,
  `BTab` (Shift+Tab), `C-c`, `Up`, `Down`, `Left`, `Right`.
- `capture` returns the pane's visible text plus scrollback, without color
  codes.

### Headless Claude runs (the chat UI)

A run is one `claude -p` invocation. To continue a conversation, start a new run
with the `sessionId` from the previous run.

| Method | Path | Body | Response |
|---|---|---|---|
| POST | `/api/runs` | `{ prompt, cwd?, sessionId?, permissionMode?, model? }` | `{ runId }` |
| GET | `/api/runs` | — | `{ runs: [Run] }`, newest first |
| GET | `/api/runs/:runId/events` | — | Server-sent events (see the following section) |
| POST | `/api/runs/:runId/stop` | `{}` | `{ ok: true }` |

- `permissionMode` is one of `plan`, `acceptEdits`, `auto`, `bypassPermissions`,
  `dontAsk`, or `manual`. It defaults to `acceptEdits`. A headless run can't ask
  for permission, so in `manual` mode Claude can't use tools that need approval.
- `model` is an alias such as `sonnet`, `opus`, or `haiku`. Omit it for the
  VM's default.
- `cwd` defaults to the home directory and must be inside it.
- `Run` is `{ runId, sessionId, cwd, prompt, permissionMode, model, status,
  startedAt, endedAt, costUsd }`. `status` is `running`, `done`, `error`, or
  `stopped`. `sessionId` is `null` until Claude reports it.

#### Run events

Open the stream with `new EventSource(url, { withCredentials: true })`. Each
message's `data` is one JSON object. The agent replays the run's earlier events
first, so you can reconnect at any time. The stream ends after `agent.exit`.

Claude's own `stream-json` events are passed through unchanged. The ones worth
rendering are these:

- `{ type: "system", subtype: "init", session_id, model, cwd }`
- `{ type: "assistant", message: { content: [Block] } }`, where a `Block` is
  `{ type: "text", text }`, `{ type: "thinking", thinking }`, or
  `{ type: "tool_use", id, name, input }`.
- `{ type: "user", message: { content: [{ type: "tool_result", tool_use_id,
  content, is_error }] } }`, where `content` is a string or an array of
  `{ type: "text", text }`.
- `{ type: "result", subtype, is_error, result, total_cost_usd, session_id,
  num_turns, duration_ms }`

Ignore other event types. The agent adds these two:

- `{ type: "agent.stderr", text }`
- `{ type: "agent.exit", code, status }`

### Claude Code transcripts

| Method | Path | Response |
|---|---|---|
| GET | `/api/projects` | `{ projects: [{ dir, cwd, sessions, updatedAt }] }` |
| GET | `/api/sessions?cwd=PATH` | `{ sessions: [{ id, cwd, title, updatedAt, messages }] }`, newest first, at most 100 |
| GET | `/api/sessions/:id` | `{ id, cwd, messages: [Message] }` |

- `cwd` filters to one project. Omit it for all projects.
- `title` is the first real user prompt, cut to 120 characters.
- `Message` is `{ role: "user" | "assistant", kind: "text" | "thinking" |
  "tool_use" | "tool_result", text, tool, at }`. `tool` is the tool name for
  `tool_use`, otherwise `null`. `at` is an ISO timestamp.
- To resume a transcript in the chat UI, start a run with `sessionId` set to the
  transcript's `id` and `cwd` set to its `cwd`.

### Files (read-only)

`GET /api/fs?path=PATH` reads a path inside the home directory. `PATH` defaults
to the home directory. A directory returns the following:

```json
{ "type": "dir", "path": "/home/ifkash", "parent": null,
  "entries": [{ "name": "docs", "type": "dir", "size": 4096, "mtime": 0 }] }
```

A file returns `{ type: "file", path, parent, size, mtime, binary, truncated,
text }`. `text` holds at most 512 KiB, and is `null` for binary files. `type` for
an entry is `dir`, `file`, or `link`.

## Web terminal

`/tty/*` is proxied to ttyd, which runs `tmux new -A -s NAME`. Connect with
`new WebSocket("wss://claude.ifkash.dev/tty/ws?arg=NAME", ["tty"])` and set
`binaryType = "arraybuffer"`. `NAME` is a tmux session name; the session is
created if it doesn't exist. The agent rejects the upgrade if the `Origin`
isn't allowed.

The ttyd protocol works as follows:

1. After the socket opens, send the text
   `JSON.stringify({ AuthToken: "", columns, rows })`.
2. To send keystrokes, send `"0" + data`. To resize, send
   `"1" + JSON.stringify({ columns, rows })`. Send both as UTF-8 bytes
   (`TextEncoder`).
3. Each server message is binary. The first byte is a command character: `"0"`
   is terminal output (write the remaining bytes to xterm.js), `"1"` is the
   window title, and `"2"` is preferences JSON (ignore it).
