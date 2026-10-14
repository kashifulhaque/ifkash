<svelte:head>
  <title>Boo — Discord AI Bot</title>
  <meta name="description" content="AI Discord bot. Natural conversations, image understanding, and generation." />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/projects">Projects</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">Boo</span>
    </nav>
    <h1 class="page-title">Boo — the (un)friendly Discord AI bot</h1>
    <p class="lead">A snark-powered Discord bot that blends LLM chat, image understanding, and handy utilities into a single docker-compose stack. Built for servers that want a fast, context-aware assistant with guardrails, a tiny admin API, and a smooth UX.</p>
    <div class="btn-row">
      <a href="https://github.com/VVIP-Kitchen/boo" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Code <span class="ext" aria-hidden="true">↗</span>
      </a>
      <a href="https://boo.ifkash.dev" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Site <span class="ext" aria-hidden="true">↗</span>
      </a>
    </div>
  </header>

  <div class="detail-body">
    <aside class="facts" aria-label="Project facts">
      <dl>
        <div class="fact">
          <dt>Status</dt>
          <dd><span class="status"><span class="led" aria-hidden="true"></span>Production-ready</span></dd>
        </div>
        <div class="fact">
          <dt>Stack</dt>
          <dd><span>Python 3.12</span><span>discord.py 2.5</span><span>Go (Gin)</span></dd>
        </div>
        <div class="fact">
          <dt>Storage</dt>
          <dd><span>PostgreSQL</span><span>Redis</span></dd>
        </div>
        <div class="fact">
          <dt>Services</dt>
          <dd>4 · Docker Compose</dd>
        </div>
        <div class="fact">
          <dt>Commands</dt>
          <dd>9</dd>
        </div>
        <div class="fact">
          <dt>Links</dt>
          <dd>
            <a href="https://github.com/VVIP-Kitchen/boo" target="_blank" rel="noopener noreferrer">GitHub <span aria-hidden="true">↗</span></a>
            <a href="https://boo.ifkash.dev" target="_blank" rel="noopener noreferrer">boo.ifkash.dev <span aria-hidden="true">↗</span></a>
          </dd>
        </div>
      </dl>
    </aside>

    <div class="copy">
      <section class="section">
        <h2><span class="num">01</span>Overview</h2>
        <p>
          Boo is a production-ready Discord bot that brings AI-powered conversations, automatic image analysis, and useful
          utilities to your server. Unlike simple chatbots, Boo maintains context across conversations, understands images,
          and provides a web-based admin panel for managing system prompts and monitoring usage.
        </p>
        <p>
          The entire stack runs in Docker Compose—no complex setup required. Just configure your API keys and you're ready to go.
        </p>
      </section>

      <section class="section">
        <h2><span class="num">02</span>Key Features</h2>
        <div class="tiles">
          <div class="tile">
            <h3>AI Chat</h3>
            <p>
              Conversational replies via OpenRouter with per-server editable system prompts. Supports mentions, replies,
              and DMs with rolling context from Redis.
            </p>
          </div>
          <div class="tile">
            <h3>Vision</h3>
            <p>
              Automatic image caption and analysis on upload. Multi-image messages are processed in sequence with full context.
            </p>
          </div>
          <div class="tile">
            <h3>Utilities</h3>
            <p>
              Weather forecasts, bonk GIFs, channel summaries, and more. Everything you need for a lively server.
            </p>
          </div>
          <div class="tile">
            <h3>Admin Panel</h3>
            <p>
              Web-based prompt editor served by a lightweight Go API. View token usage stats and manage prompts per guild.
            </p>
          </div>
          <div class="tile">
            <h3>Analytics</h3>
            <p>
              Track token usage, message counts, and API calls. Built-in stats with daily, weekly, monthly, and yearly views.
            </p>
          </div>
          <div class="tile">
            <h3>Guardrails</h3>
            <p>
              Inclusive language nudges, rate limiting, and oversized reply handling. Thoughtful UX and moderation features built-in.
            </p>
          </div>
        </div>
      </section>

      <section class="section">
        <h2><span class="num">03</span>Architecture</h2>
        <p>Boo is built as a multi-service architecture running in Docker Compose:</p>
        <ul class="defs">
          <li><span class="term">Discord Bot</span><span class="desc">Python 3.12 container with discord.py 2.5, cogs for commands, and service integrations</span></li>
          <li><span class="term">Manager API</span><span class="desc">Go (Gin) API serving prompts, messages, and token stats with a Tailwind UI</span></li>
          <li><span class="term">Redis</span><span class="desc">15-minute rolling message buffer per channel for AI summaries and context</span></li>
          <li><span class="term">Postgres</span><span class="desc">Persistent storage for prompts, archived messages, and token usage analytics</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">04</span>Quick Start</h2>
        <pre><code># Clone the repository
git clone https://github.com/VVIP-Kitchen/boo.git
cd boo

# Create .env file with your API keys
cp .env.example .env
# Edit .env with your Discord token, API keys, etc.

# Bring the stack up
docker compose up -d

# Open admin panel at http://localhost:8080</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">05</span>Commands</h2>
        <ul class="defs">
          <li><span class="term mono">/ping</span><span class="desc">Check bot latency and response time</span></li>
          <li><span class="term mono">/weather</span><span class="desc">Real-time weather data via Tomorrow.io</span></li>
          <li><span class="term mono">/bonk @user</span><span class="desc">Send a random bonk GIF from Tenor</span></li>
          <li><span class="term mono">/summary</span><span class="desc">TL;DR of the last 15 minutes in the channel</span></li>
          <li><span class="term mono">/get_prompt</span><span class="desc">Download the current system prompt</span></li>
          <li><span class="term mono">/update_prompt</span><span class="desc">Update system prompt from uploaded file</span></li>
          <li><span class="term mono">/add_prompt</span><span class="desc">Add system prompt from uploaded file</span></li>
          <li><span class="term mono">reset chat</span><span class="desc">Type in channel to clear context buffer</span></li>
          <li><span class="term mono">!@sync</span><span class="desc">Owner-only command to resync slash commands</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">06</span>Configuration</h2>
        <p>Boo requires several API keys and configuration options:</p>
        <pre><code># Discord
DISCORD_TOKEN=YOUR_DISCORD_BOT_TOKEN
ADMIN_LIST=123456789012345678,987654321098765432
CONTEXT_LIMIT=40

# APIs
TENOR_API_KEY=XXXXXXXXXXXX
TOMORROW_IO_API_KEY=XXXXXXXXXXXX
OPENROUTER_API_KEY=XXXXXXXXXXXX
OPENROUTER_MODEL=meta-llama/llama-4-maverick
TAVILY_API_KEY=XXXXXXXXXXXX
MANAGER_API_TOKEN=super-secure-shared-secret

# Database (compose wires these for containers)
POSTGRES_USER=db-user
POSTGRES_PASSWORD=db-password
POSTGRES_DB=db-name</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">07</span>Manager API</h2>
        <p>The Go-based manager API provides a RESTful interface for managing prompts and viewing analytics:</p>
        <ul class="defs">
          <li><span class="term"><code>GET /docs</code></span><span class="desc">Swagger UI documentation</span></li>
          <li><span class="term"><code>GET /admin</code></span><span class="desc">Web-based prompt manager UI</span></li>
          <li><span class="term"><code>GET /prompt?guild_id=...</code></span><span class="desc">Fetch per-guild system prompt</span></li>
          <li><span class="term"><code>POST /prompt</code></span><span class="desc">Add new prompt</span></li>
          <li><span class="term"><code>PUT /prompt?guild_id=...</code></span><span class="desc">Update existing prompt</span></li>
          <li><span class="term"><code>POST /message</code></span><span class="desc">Archive messages</span></li>
          <li><span class="term"><code>POST /token</code></span><span class="desc">Record token usage</span></li>
          <li><span class="term"><code>GET /token/stats</code></span><span class="desc">View usage statistics</span></li>
        </ul>
        <p class="note">
          All API calls require <code>Authorization: Bearer &lt;MANAGER_API_TOKEN&gt;</code> header for authentication.
        </p>
      </section>

      <section class="section">
        <h2><span class="num">08</span>Tech Stack</h2>
        <ul class="defs">
          <li>
            <span class="term">Backend</span>
            <span class="tags">
              <span class="tag">Python 3.12</span>
              <span class="tag">discord.py 2.5</span>
              <span class="tag">Go (Gin)</span>
            </span>
          </li>
          <li>
            <span class="term">AI & APIs</span>
            <span class="tags">
              <span class="tag">OpenRouter</span>
              <span class="tag">Tomorrow.io</span>
              <span class="tag">Tenor</span>
              <span class="tag">Tavily</span>
            </span>
          </li>
          <li>
            <span class="term">Storage</span>
            <span class="tags">
              <span class="tag">PostgreSQL</span>
              <span class="tag">Redis</span>
            </span>
          </li>
          <li>
            <span class="term">Frontend</span>
            <span class="tags">
              <span class="tag">Tailwind CSS</span>
              <span class="tag">Swagger UI</span>
            </span>
          </li>
          <li>
            <span class="term">DevOps</span>
            <span class="tags">
              <span class="tag">Docker</span>
              <span class="tag">Docker Compose</span>
            </span>
          </li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">09</span>Features in Detail</h2>

        <h3>Context Management</h3>
        <p>
          Boo maintains conversation context using Redis as a rolling buffer. Each channel gets its own context window
          (configurable via CONTEXT_LIMIT), and messages older than 15 minutes are automatically pruned. This ensures
          relevant, recent context without unbounded memory growth.
        </p>

        <h3>Image Understanding</h3>
        <p>
          When images are uploaded, Boo automatically analyzes them using vision models via OpenRouter. Multi-image messages
          are processed sequentially, and the bot incorporates visual understanding into its conversational responses.
        </p>

        <h3>Inclusive Language</h3>
        <p>
          Boo includes optional "guys-check" functionality that gently nudges users toward more inclusive language.
          This feature can be customized or disabled based on your server's preferences.
        </p>

        <h3>Rate Limiting</h3>
        <p>
          Built-in rate limit handling ensures the bot gracefully handles API throttling. If OpenRouter returns a 429
          status, Boo relays the retry ETA to users instead of silently failing.
        </p>
      </section>

      <section class="section">
        <h2><span class="num">10</span>Extensibility</h2>
        <ul class="defs">
          <li><span class="term">Add commands</span><span class="desc">Create a new cog in <code>src/commands</code> and load it in <code>bot/bot.py</code></span></li>
          <li><span class="term">Swap LLMs</span><span class="desc">Change <code>OPENROUTER_MODEL</code> or extend <code>LLMService</code> for different providers</span></li>
          <li><span class="term">Storage</span><span class="desc">Extend manager internal services and Python <code>DBService</code> for more analytics</span></li>
          <li><span class="term">Emoji/stickers</span><span class="desc">Tweak <code>utils/emoji_utils.py</code> to adjust patterns and rendering</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">11</span>Repository Structure</h2>
        <ul class="defs">
          <li><span class="term"><code>compose.yml</code></span><span class="desc">4-service orchestration (postgres, redis, manager, bot)</span></li>
          <li><span class="term"><code>Dockerfile</code></span><span class="desc">Python 3.12 base with uv for the Discord bot</span></li>
          <li><span class="term"><code>src/</code></span><span class="desc">Discord bot implementation with cogs, commands, and services</span></li>
          <li><span class="term"><code>manager/</code></span><span class="desc">Go API + static Tailwind UI for prompts and analytics</span></li>
          <li><span class="term"><code>modal/</code></span><span class="desc">Serverless functions for ASR and embeddings</span></li>
          <li><span class="term"><code>sandbox/</code></span><span class="desc">Isolated Python execution environment</span></li>
        </ul>
      </section>
    </div>
  </div>
</article>

<style>
  /* ── Detail page (shared across project write-ups) ───────────── */

  .page-title {
    max-width: 18ch;
    overflow-wrap: break-word;
  }

  /* Sections carry their own top hairline, so the head drops its rule */
  .page-head {
    border-bottom: 0;
  }

  .page-head .lead {
    margin: 0;
  }

  .page-head .btn-row {
    margin-top: var(--s-2);
  }

  .ext {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .btn:hover .ext {
    color: inherit;
    transform: translate(2px, -2px);
  }

  .detail-body {
    display: grid;
    gap: var(--s-7);
  }

  /* Facts: mono summary column */
  .facts dl {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--s-5);
    margin: 0;
    border-top: 1px solid var(--line);
  }

  .fact {
    min-width: 0;
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .fact dt {
    margin-bottom: var(--s-1);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .fact dd {
    display: grid;
    gap: 2px;
    margin: 0;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.5;
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
  }

  .fact a {
    display: inline-flex;
    align-items: baseline;
    gap: var(--s-2);
    color: var(--ink);
    text-decoration: none;
  }

  .fact a:hover {
    color: var(--fjord);
  }

  .fact a span {
    color: var(--ink-3);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .fact a:hover span {
    color: var(--fjord);
    transform: translate(2px, -2px);
  }

  /* Reading column (not .prose: that name belongs to Tailwind Typography) */
  .copy {
    min-width: 0;
  }

  .section {
    padding-top: var(--s-4);
    border-top: 1px solid var(--line);
  }

  .section + .section {
    margin-top: var(--s-8);
  }

  .section > * {
    margin: 0;
  }

  .section > * + * {
    margin-top: var(--s-4);
  }

  .section h2 {
    display: flex;
    align-items: baseline;
    gap: var(--s-4);
    font-size: var(--text-xl);
    letter-spacing: -0.022em;
  }

  .num {
    flex: none;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    letter-spacing: 0.02em;
    font-variant-numeric: tabular-nums;
  }

  .section > h2 + * {
    margin-top: var(--s-5);
  }

  .section p {
    line-height: 1.65;
  }

  @media (min-width: 960px) {
    .detail-body {
      grid-template-columns: 240px minmax(0, var(--measure));
      column-gap: var(--s-8);
      align-items: start;
      padding-top: var(--s-4);
    }

    .facts {
      position: sticky;
      top: calc(64px + var(--s-6));
    }

    .facts dl {
      grid-template-columns: minmax(0, 1fr);
    }

    .section + .section {
      margin-top: var(--s-9);
    }
  }

  /* Sub-headings inside a section */
  .section > * + h3 {
    margin-top: var(--s-7);
  }

  .section > h3 + * {
    margin-top: var(--s-3);
  }

  .section h3 {
    font-size: var(--text-md);
    letter-spacing: -0.01em;
  }

  .status {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
  }

  /* Aside paragraph set off by a hairline */
  .note {
    padding-left: var(--s-4);
    border-left: 1px solid var(--line-strong);
    color: var(--ink-2);
  }

  /* Term / description rows */
  .defs {
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  .defs > li {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-1) var(--s-5);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .term {
    min-width: 0;
    color: var(--ink);
    font-weight: 500;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }


  .desc {
    color: var(--ink-2);
    line-height: 1.55;
  }

  @media (min-width: 600px) {
    .defs > li {
      grid-template-columns: minmax(0, 12rem) minmax(0, 1fr);
    }
  }

  .term code {
    padding: 0;
    background: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
  }

  .term.mono {
    padding: 0;
    background: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
  }

  /* Two-up notes, each topped by a hairline */
  .tiles {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-5) var(--s-6);
  }

  .tile {
    padding-top: var(--s-3);
    border-top: 1px solid var(--line);
  }

  .tile h3 {
    margin: 0 0 var(--s-1);
    font-size: var(--text-base);
    letter-spacing: -0.005em;
  }

  .tile p {
    margin: 0;
    line-height: 1.55;
  }

  @media (min-width: 600px) {
    .tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
</style>
