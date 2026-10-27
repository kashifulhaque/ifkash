# VM console (`ifkash.dev/vm`)

The VM console lets you drive Claude Code on the VM from a browser, including a
phone. It has a chat UI for headless Claude runs, tmux session management, a web
terminal, system stats, Docker logs, and a read-only file browser.

## How it fits together

```
ifkash.dev/vm (Pages) ──fetch / SSE / WebSocket──▶ claude.ifkash.dev
                                                    │ Cloudflare Access (owner only)
                                                    ▼
                                               Cloudflare Tunnel (outbound only)
                                                    │ cloudflared checks the Access JWT
                                                    ▼
                                               VM agent  127.0.0.1:7690
                                               ├─ /api/*  JSON API (vm/API.md)
                                               └─ /tty/*  → ttyd (Unix socket) → tmux
```

- **`agent/server.mjs`**: the VM agent, a dependency-free Node 22 server that
  listens on loopback only. It checks the Access JWT on every request, even
  though cloudflared checks it too. It accepts `POST` requests and terminal
  WebSockets only from an allowed `Origin`, so another site can't use your Access
  cookie to send commands.
- **`agent/vm-tty-attach`**: what ttyd runs for each terminal. It attaches to,
  or creates, the tmux session named in the URL.
- **`systemd/`**: the `claude-remote-agent` and `claude-remote-ttyd` services.
  Both use `KillMode=process`, so restarting them leaves tmux sessions and
  Claude runs alive.
- **`API.md`**: the contract between the agent and the `/vm` page.
- The page lives in `src/routes/vm` and `src/lib/components/vm`, and its client is
  `src/lib/vmApi.ts`.

On the VM, the agent is installed in `~/docs/infra/claude-remote/`. Its settings
are in `~/docs/infra/claude-remote/.env` (mode 600). For the settings, see
`agent/env.example`.

## Deploy agent changes

To copy the agent to the VM and restart both services, run the following
command:

```sh
vm/deploy.sh
```

The script installs ttyd if it's missing and creates `.env` from
`env.example` on the first deploy. It never overwrites an existing `.env`.

## First-time setup

These steps happen in the Cloudflare dashboard and on the VM, and only need to
be done once.

1. Create the Access application for the console:
   1. In the [Zero Trust dashboard](https://one.dash.cloudflare.com), go to
      **Access > Applications** and click **Add an application**.
   2. Select **Self-hosted**.
   3. Set **Application name** to `ifkash vm`, and set the destination to the
      subdomain `claude` on the domain `ifkash.dev`, with no path.
   4. Set **Session duration** to a short value, such as `2 hours`. The console
      can run `sudo`, so a stolen session is worth more than one for the
      résumé editor.
   5. Add a policy: **Allow**, with the rule **Emails** set to
      `haque.kashiful7@gmail.com`.
   6. Under login methods, we recommend Google with two-step verification
      instead of an emailed one-time PIN.
   7. Save, then copy the **Application Audience (AUD) Tag** from the app's
      **Overview** tab.
2. Create the tunnel:
   1. Go to **Networks > Tunnels**, click **Create a tunnel**, and select
      **Cloudflared**.
   2. Name it `ifkash-vm`.
   3. Under **Install and run a connector**, select **Debian** and **64-bit**.
      Connect to the VM by using SSH and run the commands that the dashboard
      shows. The last command is `sudo cloudflared service install TOKEN`. The
      token is a secret, so keep it out of chat and out of the repo.
   4. When the connector shows as **Connected**, click **Next**.
3. Route `claude.ifkash.dev` to the agent:
   1. Add a public hostname with subdomain `claude`, domain `ifkash.dev`, and
      service **HTTP** `localhost:7690`.
   2. Under **Additional application settings > Access**, turn on
      **Protect with Access**. Set the team name to `ifkash` and the AUD tag to
      the one from step 1.
   3. Save. Cloudflare creates the proxied `claude` DNS record for you.
4. Give the agent the AUD tag. On the VM, set `ACCESS_AUD` in
   `~/docs/infra/claude-remote/.env`, then run the following command:

   ```sh
   sudo systemctl restart claude-remote-agent
   ```

5. Open `https://ifkash.dev/vm` and click **Sign in**.

## Troubleshooting

| Symptom | Likely cause |
|---|---|
| The page keeps asking you to sign in | `ACCESS_AUD` in the VM's `.env` doesn't match the `ifkash vm` app, or the agent wasn't restarted after the change. To check, run `journalctl -u claude-remote-agent -n 20` on the VM. |
| Requests fail with `403 origin not allowed` | You're using the page from an origin that's not in `ALLOWED_ORIGINS`. For local development, add `http://localhost:5180`. |
| The terminal says it can't connect | The `claude-remote-ttyd` service is down. To check, run `systemctl status claude-remote-ttyd`. |
| A run fails immediately | Claude Code on the VM isn't signed in, or the working directory is outside your home directory. Check the run's error events. |

## Local development

To test the page against the real VM without the tunnel, forward the agent port
and run a second agent instance with authentication turned off:

```sh
ssh -i ~/.ssh/id_ed25519_kh -L 7690:127.0.0.1:7691 ifkash@vm.ifkash.dev \
  'cd ~/docs/infra/claude-remote && LOCAL_DEV=1 PORT=7691 \
   ALLOWED_ORIGINS=http://localhost:5180 node server.mjs'
```

Then open `http://localhost:5180/vm`. Stop the command when you're done. While
it runs, any process on the VM can use the agent without signing in.
