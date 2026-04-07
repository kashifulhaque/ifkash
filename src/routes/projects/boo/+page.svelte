<svelte:head>
  <title>Boo — Discord AI Bot</title>
  <meta name="description" content="AI Discord bot. Natural conversations, image understanding, and generation." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/projects">Projects</a>
      <span class="separator">/</span>
      <span>Boo</span>
    </div>
    <h1 class="page-title">Boo — the (un)friendly Discord AI bot</h1>
    <p class="page-desc">A snark-powered Discord bot that blends LLM chat, image understanding, and handy utilities into a single docker-compose stack. Built for servers that want a fast, context-aware assistant with guardrails, a tiny admin API, and a smooth UX.</p>
    <div class="project-links">
      <a href="https://github.com/VVIP-Kitchen/boo" target="_blank" rel="noopener noreferrer" class="project-link">
        Code
      </a>
      <a href="https://boo.ifkash.dev" target="_blank" rel="noopener noreferrer" class="project-link">
        Site
      </a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Overview</h2>
      <p>
        Boo is a production-ready Discord bot that brings AI-powered conversations, automatic image analysis, and useful 
        utilities to your server. Unlike simple chatbots, Boo maintains context across conversations, understands images, 
        and provides a web-based admin panel for managing system prompts and monitoring usage.
      </p>
      <p>
        The entire stack runs in Docker Compose—no complex setup required. Just configure your API keys and you're ready to go.
      </p>
    </div>

    <div class="section">
      <h2>Key Features</h2>
      <div class="features-grid">
        <div class="feature-card">
          <div class="feature-icon">💬</div>
          <div class="feature-title">AI Chat</div>
          <div class="feature-desc">
            Conversational replies via OpenRouter with per-server editable system prompts. Supports mentions, replies, 
            and DMs with rolling context from Redis.
          </div>
        </div>
        <div class="feature-card">
          <div class="feature-icon">👁️</div>
          <div class="feature-title">Vision</div>
          <div class="feature-desc">
            Automatic image caption and analysis on upload. Multi-image messages are processed in sequence with full context.
          </div>
        </div>
        <div class="feature-card">
          <div class="feature-icon">🛠️</div>
          <div class="feature-title">Utilities</div>
          <div class="feature-desc">
            Weather forecasts, bonk GIFs, channel summaries, and more. Everything you need for a lively server.
          </div>
        </div>
        <div class="feature-card">
          <div class="feature-icon">⚙️</div>
          <div class="feature-title">Admin Panel</div>
          <div class="feature-desc">
            Web-based prompt editor served by a lightweight Go API. View token usage stats and manage prompts per guild.
          </div>
        </div>
        <div class="feature-card">
          <div class="feature-icon">📊</div>
          <div class="feature-title">Analytics</div>
          <div class="feature-desc">
            Track token usage, message counts, and API calls. Built-in stats with daily, weekly, monthly, and yearly views.
          </div>
        </div>
        <div class="feature-card">
          <div class="feature-icon">🔒</div>
          <div class="feature-title">Guardrails</div>
          <div class="feature-desc">
            Inclusive language nudges, rate limiting, and oversized reply handling. Thoughtful UX and moderation features built-in.
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Architecture</h2>
      <p>Boo is built as a multi-service architecture running in Docker Compose:</p>
      <div class="arch-list">
        <div class="arch-item">
          <div class="arch-name">Discord Bot</div>
          <div class="arch-desc">Python 3.12 container with discord.py 2.5, cogs for commands, and service integrations</div>
        </div>
        <div class="arch-item">
          <div class="arch-name">Manager API</div>
          <div class="arch-desc">Go (Gin) API serving prompts, messages, and token stats with a Tailwind UI</div>
        </div>
        <div class="arch-item">
          <div class="arch-name">Redis</div>
          <div class="arch-desc">15-minute rolling message buffer per channel for AI summaries and context</div>
        </div>
        <div class="arch-item">
          <div class="arch-name">Postgres</div>
          <div class="arch-desc">Persistent storage for prompts, archived messages, and token usage analytics</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Quick Start</h2>
      <div class="code-block">
        <pre><code># Clone the repository
git clone https://github.com/VVIP-Kitchen/boo.git
cd boo

# Create .env file with your API keys
cp .env.example .env
# Edit .env with your Discord token, API keys, etc.

# Bring the stack up
docker compose up -d

# Open admin panel at http://localhost:8080</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Commands</h2>
      <div class="commands-grid">
        <div class="command-item">
          <div class="command-name">/ping</div>
          <div class="command-desc">Check bot latency and response time</div>
        </div>
        <div class="command-item">
          <div class="command-name">/weather</div>
          <div class="command-desc">Real-time weather data via Tomorrow.io</div>
        </div>
        <div class="command-item">
          <div class="command-name">/bonk @user</div>
          <div class="command-desc">Send a random bonk GIF from Tenor</div>
        </div>
        <div class="command-item">
          <div class="command-name">/summary</div>
          <div class="command-desc">TL;DR of the last 15 minutes in the channel</div>
        </div>
        <div class="command-item">
          <div class="command-name">/get_prompt</div>
          <div class="command-desc">Download the current system prompt</div>
        </div>
        <div class="command-item">
          <div class="command-name">/update_prompt</div>
          <div class="command-desc">Update system prompt from uploaded file</div>
        </div>
        <div class="command-item">
          <div class="command-name">/add_prompt</div>
          <div class="command-desc">Add system prompt from uploaded file</div>
        </div>
        <div class="command-item">
          <div class="command-name">reset chat</div>
          <div class="command-desc">Type in channel to clear context buffer</div>
        </div>
        <div class="command-item">
          <div class="command-name">!@sync</div>
          <div class="command-desc">Owner-only command to resync slash commands</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Configuration</h2>
      <p>Boo requires several API keys and configuration options:</p>
      <div class="code-block">
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
      </div>
    </div>

    <div class="section">
      <h2>Manager API</h2>
      <p>The Go-based manager API provides a RESTful interface for managing prompts and viewing analytics:</p>
      <ul class="api-list">
        <li><code>GET /docs</code> — Swagger UI documentation</li>
        <li><code>GET /admin</code> — Web-based prompt manager UI</li>
        <li><code>GET /prompt?guild_id=...</code> — Fetch per-guild system prompt</li>
        <li><code>POST /prompt</code> — Add new prompt</li>
        <li><code>PUT /prompt?guild_id=...</code> — Update existing prompt</li>
        <li><code>POST /message</code> — Archive messages</li>
        <li><code>POST /token</code> — Record token usage</li>
        <li><code>GET /token/stats</code> — View usage statistics</li>
      </ul>
      <p class="api-note">
        🔐 All API calls require <code>Authorization: Bearer &lt;MANAGER_API_TOKEN&gt;</code> header for authentication.
      </p>
    </div>

    <div class="section">
      <h2>Tech Stack</h2>
      <div class="tech-grid">
        <div class="tech-item">
          <div class="tech-category">Backend</div>
          <div class="tech-list">
            <span class="tech-tag">Python 3.12</span>
            <span class="tech-tag">discord.py 2.5</span>
            <span class="tech-tag">Go (Gin)</span>
          </div>
        </div>
        <div class="tech-item">
          <div class="tech-category">AI & APIs</div>
          <div class="tech-list">
            <span class="tech-tag">OpenRouter</span>
            <span class="tech-tag">Tomorrow.io</span>
            <span class="tech-tag">Tenor</span>
            <span class="tech-tag">Tavily</span>
          </div>
        </div>
        <div class="tech-item">
          <div class="tech-category">Storage</div>
          <div class="tech-list">
            <span class="tech-tag">PostgreSQL</span>
            <span class="tech-tag">Redis</span>
          </div>
        </div>
        <div class="tech-item">
          <div class="tech-category">Frontend</div>
          <div class="tech-list">
            <span class="tech-tag">Tailwind CSS</span>
            <span class="tech-tag">Swagger UI</span>
          </div>
        </div>
        <div class="tech-item">
          <div class="tech-category">DevOps</div>
          <div class="tech-list">
            <span class="tech-tag">Docker</span>
            <span class="tech-tag">Docker Compose</span>
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Features in Detail</h2>
      
      <h3 class="subsection-title">Context Management</h3>
      <p>
        Boo maintains conversation context using Redis as a rolling buffer. Each channel gets its own context window 
        (configurable via CONTEXT_LIMIT), and messages older than 15 minutes are automatically pruned. This ensures 
        relevant, recent context without unbounded memory growth.
      </p>

      <h3 class="subsection-title">Image Understanding</h3>
      <p>
        When images are uploaded, Boo automatically analyzes them using vision models via OpenRouter. Multi-image messages 
        are processed sequentially, and the bot incorporates visual understanding into its conversational responses.
      </p>

      <h3 class="subsection-title">Inclusive Language</h3>
      <p>
        Boo includes optional "guys-check" functionality that gently nudges users toward more inclusive language. 
        This feature can be customized or disabled based on your server's preferences.
      </p>

      <h3 class="subsection-title">Rate Limiting</h3>
      <p>
        Built-in rate limit handling ensures the bot gracefully handles API throttling. If OpenRouter returns a 429 
        status, Boo relays the retry ETA to users instead of silently failing.
      </p>
    </div>

    <div class="section">
      <h2>Extensibility</h2>
      <ul class="extend-list">
        <li><strong>Add commands</strong>: Create a new cog in <code>src/commands</code> and load it in <code>bot/bot.py</code></li>
        <li><strong>Swap LLMs</strong>: Change <code>OPENROUTER_MODEL</code> or extend <code>LLMService</code> for different providers</li>
        <li><strong>Storage</strong>: Extend manager internal services and Python <code>DBService</code> for more analytics</li>
        <li><strong>Emoji/stickers</strong>: Tweak <code>utils/emoji_utils.py</code> to adjust patterns and rendering</li>
      </ul>
    </div>

    <div class="section">
      <h2>Repository Structure</h2>
      <ul class="structure-list">
        <li><code>compose.yml</code>: 4-service orchestration (postgres, redis, manager, bot)</li>
        <li><code>Dockerfile</code>: Python 3.12 base with uv for the Discord bot</li>
        <li><code>src/</code>: Discord bot implementation with cogs, commands, and services</li>
        <li><code>manager/</code>: Go API + static Tailwind UI for prompts and analytics</li>
        <li><code>modal/</code>: Serverless functions for ASR and embeddings</li>
        <li><code>sandbox/</code>: Isolated Python execution environment</li>
      </ul>
    </div>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 3rem;
    max-width: 48rem;
    animation: fade-up var(--dur-base) var(--ease-out-quart);
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--border);
  }

  .breadcrumb {
    font-size: 0.875rem;
    color: var(--text-tertiary);
  }

  .breadcrumb a {
    color: var(--text-tertiary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .breadcrumb a:hover {
    color: var(--text-primary);
  }

  .separator {
    margin: 0 0.5rem;
  }

  .page-title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin: 0;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.6;
    color: var(--text-secondary);
    margin: 0;
  }

  .project-links {
    display: flex;
    gap: 1rem;
  }

  .project-link {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    transition: color var(--dur-instant) var(--ease-out-quart);
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
  }

  .project-link:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2.5rem;
  }

  .section h2 {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
    margin-bottom: 1rem;
  }

  .section h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.75rem;
  }

  .subsection-title {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
  }

  .section p {
    font-size: 0.9375rem;
    line-height: 1.7;
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .section p:last-child {
    margin-bottom: 0;
  }

  .features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 1rem;
  }

  .feature-card {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1.25rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
  }

  .feature-icon {
    font-size: 2rem;
    line-height: 1;
  }

  .feature-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .feature-desc {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .arch-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .arch-item {
    padding: 1rem;
    background: var(--surface-raised);
    border-left: 2px solid var(--border);
  }

  .arch-name {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.375rem;
  }

  .arch-desc {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .commands-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
  }

  .command-item {
    padding: 0.875rem 1rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
  }

  .command-name {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--paper-dim);
    margin-bottom: 0.375rem;
  }

  .command-desc {
    font-size: 0.8125rem;
    line-height: 1.5;
    color: var(--text-tertiary);
  }

  .api-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    list-style: none;
    padding: 0;
  }

  .api-list li {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--text-secondary);
    padding: 0.5rem 0.75rem;
    background: var(--surface-raised);
    border-left: 2px solid var(--border);
  }

  .api-list code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .api-note {
    margin-top: 1rem;
    padding: 1rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  .api-note code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.875rem;
    color: var(--text-secondary);
    background: var(--surface-sunken);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .tech-grid {
    display: grid;
    gap: 1rem;
  }

  .tech-item {
    padding: 1rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
  }

  .tech-category {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .tech-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .tech-tag {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    background: var(--surface-sunken);
    padding: 0.375rem 0.75rem;
    border-radius: 0.25rem;
    border: 1px solid var(--border);
  }

  .extend-list,
  .structure-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
  }

  .extend-list li,
  .structure-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--text-secondary);
    padding-left: 1.5rem;
    position: relative;
  }

  .extend-list li::before,
  .structure-list li::before {
    content: '→';
    position: absolute;
    left: 0;
    color: var(--text-faint);
  }

  .extend-list code,
  .structure-list code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.875rem;
    color: var(--text-secondary);
    background: var(--surface-raised);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .code-block {
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow-x: auto;
  }

  .code-block pre {
    margin: 0;
    padding: 1.25rem;
  }

  .code-block code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @media (max-width: 768px) {
    .page-title {
      font-size: 2rem;
    }

    .section h2 {
      font-size: 1.25rem;
    }

    .features-grid {
      grid-template-columns: 1fr;
    }

    .commands-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
