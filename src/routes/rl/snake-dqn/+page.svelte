<svelte:head>
  <title>Snake RL — DQN</title>
  <meta
    name="description"
    content="A snake agent trained from scratch with Dueling Double DQN: a 28-dim egocentric state, a target network, and a 256-unit network. Greedy eval mean 102."
  />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/rl">RL</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">Snake DQN</span>
    </nav>
    <h1 class="page-title">Snake RL</h1>
    <p class="lead">
      A snake agent trained from scratch with Dueling Double Deep Q-Networks
      (DDQN) in PyTorch. The policy learns to survive, chase food, and avoid
      its own tail from raw game state — no hand-coded heuristics.
    </p>
    <div class="btn-row links">
      <a href="/rl/snake-dqn/play" class="btn btn-primary">Play in browser</a>
      <a
        href="https://github.com/kashifulhaque/snake-rl"
        target="_blank"
        rel="noopener noreferrer"
        class="btn"
      >
        Code
      </a>
    </div>
  </header>

  <div class="layout">
    <aside class="facts" aria-label="Experiment facts">
      <dl>
        <div class="fact">
          <dt>Method</dt>
          <dd>Dueling Double DQN</dd>
        </div>
        <div class="fact">
          <dt>Framework</dt>
          <dd>PyTorch</dd>
        </div>
        <div class="fact">
          <dt>State</dt>
          <dd>28 dims</dd>
        </div>
        <div class="fact">
          <dt>Network</dt>
          <dd>28 → 256 → 256</dd>
        </div>
        <div class="fact">
          <dt>Board</dt>
          <dd>32 × 24</dd>
        </div>
        <div class="fact">
          <dt>Eval mean</dt>
          <dd><span class="led" aria-hidden="true"></span>102</dd>
        </div>
      </dl>
    </aside>

    <div class="prose">
      <section class="section">
        <h2><span class="num" aria-hidden="true">01</span>Overview</h2>
        <p>
          The environment is a headless-friendly pygame implementation
          (<code>snake_env.py</code>): a 640×480 grid with a discrete action
          space of three relative moves — <em>straight</em>, <em>turn
          right</em>, <em>turn left</em> — and rewards of +10 for food,
          −10 for dying, and 0 otherwise. Moving into the tail cell is legal
          (it vacates on the same tick), and an idle cap of <code>100 ×
          snake length</code> steps since the last food truncates the episode
          instead of looping forever.
        </p>
        <p>
          The agent (<code>model.py</code>) is a dueling Double DQN: a
          <code>28 → 256 → 256</code> ReLU body, then separate value and
          advantage heads summed dueling-style. It trains against a frozen
          target network (hard update every 1000 steps) with 3-step returns,
          Huber loss, and an ε-greedy schedule decaying from 1.0 to 0.005 over
          150k steps.
        </p>
        <p>
          The state (<code>state.py</code>) encodes 28 egocentric features: a
          full obstacle ray per move direction, flood-fill free space per
          candidate move (what lets the agent avoid sealing itself into a dead
          end), food and tail position in the snake's own frame, and board
          occupancy. The v1 state was an 11-feature encoding that could not see
          its own body shape and plateaued at a mean of ~18.
        </p>
      </section>

      <section class="section">
        <h2><span class="num" aria-hidden="true">02</span>Results</h2>
        <div class="stats-grid">
          <div class="stat">
            <div class="stat-value">102</div>
            <div class="stat-label">Greedy eval mean</div>
          </div>
          <div class="stat">
            <div class="stat-value">143</div>
            <div class="stat-label">Record score</div>
          </div>
          <div class="stat">
            <div class="stat-value">28</div>
            <div class="stat-label">State dims</div>
          </div>
          <div class="stat">
            <div class="stat-value">256</div>
            <div class="stat-label">Hidden units</div>
          </div>
        </div>
        <p>
          Measured on the same 32×24 board: greedy evaluation of
          <code>runs/v2/best.pth</code> averages 102 per game (max 143 in 20
          eval games, 173 in a 50-game run) — roughly a 5× jump over the v1
          mean of ~18. The checkpoint is saved whenever the greedy eval mean
          improves, so it is protected against a late-training dip.
        </p>
        <p>
          The trained weights are also exported to
          <code>src/lib/rl/snake-dqn-weights.json</code> so the same agent can
          play in the browser — a TypeScript port of the env, state encoding,
          and dueling forward pass (no training). <a href="/rl/snake-dqn/play"
            class="inline-link">Watch it play live</a
          >.
        </p>
      </section>

      <section class="section">
        <h2><span class="num" aria-hidden="true">03</span>Run it yourself</h2>
        <div class="code-block">
          <pre><code>uv run train.py   # train a new agent headless (~1500 steps/s)
uv run play.py    # watch the saved best.pth play
uv run plot.py    # render checkpoints/log.csv -&gt; progress.png</code></pre>
        </div>
        <p>
          Dependencies: <code>torch</code>, <code>pygame</code>,
          <code>numpy</code> — pinned in <code>pyproject.toml</code> (Python
          ≥ 3.12, managed with uv). Training is headless by default; add
          <code>--render</code> to watch it learn.
        </p>
      </section>

      <section class="section">
        <h2><span class="num" aria-hidden="true">04</span>Notes &amp; next steps</h2>
        <ul class="feature-list">
          <li>
            <strong>v1 → v2:</strong> the original 11-feature state could only
            see the three cells touching the head, so a long snake had no way
            to perceive that it was sealing itself in — the state was not
            Markov enough to support scores past ~30 no matter how long you
            trained. The 28-feature encoding (rays, flood-fill free space,
            egocentric food/tail geometry) is what unlocks the ~5× jump.
          </li>
          <li>
            <strong>Algorithm fixes:</strong> the v1 update never detached its
            bootstrap target, so gradients flowed into it and the net regressed
            onto itself. v2 uses a frozen target network (hard update every
            1000 steps), Double DQN action selection, a dueling head, 3-step
            returns, and Huber loss with grad clipping.
          </li>
          <li>
            <strong>Next step:</strong> the current encoding is hand-designed.
            The real jump is a convolutional net over a stacked grid
            representation, which removes feature engineering entirely — or a
            Hamiltonian-cycle solver for near-perfect play.
          </li>
        </ul>
      </section>
    </div>
  </div>
</article>

<style>
  .detail {
    animation: rise var(--dur-slow) var(--ease-out) both;
  }

  .links {
    margin-top: var(--s-2);
  }

  /* ── Two-column body: sticky facts + 64ch prose ─────────────── */

  .layout {
    display: grid;
    grid-template-columns: 240px minmax(0, var(--measure));
    gap: var(--s-8);
    align-items: start;
    padding-top: var(--s-8);
  }

  .facts {
    position: sticky;
    top: calc(64px + var(--s-6));
  }

  .facts dl {
    margin: 0;
    border-top: 1px solid var(--line);
  }

  .fact {
    display: grid;
    gap: var(--s-1);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .fact dt {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.02em;
  }

  .fact dd {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin: 0;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }

  /* ── Prose ──────────────────────────────────────────────────── */

  .prose {
    display: grid;
    gap: var(--s-8);
    min-width: 0;
  }

  .section h2 {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    margin: 0 0 var(--s-5);
    font-size: var(--text-xl);
    letter-spacing: -0.022em;
  }

  .num {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    letter-spacing: 0.02em;
    font-variant-numeric: tabular-nums;
    transform: translateY(-0.35em);
  }

  .section p {
    margin: 0 0 var(--s-4);
    color: var(--ink-2);
    font-size: var(--text-base);
    line-height: 1.7;
  }

  .section p:last-child {
    margin-bottom: 0;
  }

  .section em {
    color: var(--ink);
  }

  .inline-link {
    color: var(--ink);
  }

  /* Results: a hairline-ruled readout strip */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    margin: 0 0 var(--s-6);
    border-top: 1px solid var(--line);
    border-bottom: 1px solid var(--line);
  }

  .stat {
    display: grid;
    gap: var(--s-2);
    padding: var(--s-5) var(--s-4) var(--s-4);
    border-left: 1px solid var(--line);
  }

  .stat:first-child {
    padding-left: 0;
    border-left: 0;
  }

  .stat-value {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xl);
    font-weight: 400;
    line-height: 1;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
  }

  .stat-label {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.02em;
  }

  .code-block {
    margin: 0 0 var(--s-5);
  }

  .code-block pre {
    margin: 0;
  }

  .code-block code {
    color: var(--ink);
  }

  /* Notes: hairline-ruled rows instead of bullets */
  .feature-list {
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  .feature-list li {
    padding: var(--s-4) 0;
    border-bottom: 1px solid var(--line);
    color: var(--ink-2);
    font-size: var(--text-base);
    line-height: 1.7;
  }

  .feature-list strong {
    font-weight: 500;
  }

  @media (max-width: 960px) {
    .layout {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--s-7);
      padding-top: var(--s-7);
    }

    .facts {
      position: static;
    }

    .facts dl {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
      column-gap: var(--s-5);
    }

    .prose {
      gap: var(--s-7);
    }
  }

  @media (max-width: 560px) {
    .stats-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .stat:nth-child(3) {
      padding-left: 0;
      border-left: 0;
    }

    .stat:nth-child(n + 3) {
      border-top: 1px solid var(--line);
    }
  }
</style>
