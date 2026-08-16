<svelte:head>
  <title>Snake RL — DQN</title>
  <meta
    name="description"
    content="A snake agent trained from scratch with Dueling Double DQN: a 28-dim egocentric state, a target network, and a 256-unit network. Greedy eval mean 102."
  />
</svelte:head>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/rl">RL</a>
      <span class="separator">/</span>
      <span>snake-dqn</span>
    </div>
    <h1 class="page-title">Snake RL</h1>
    <p class="page-desc">
      A snake agent trained from scratch with Dueling Double Deep Q-Networks
      (DDQN) in PyTorch. The policy learns to survive, chase food, and avoid
      its own tail from raw game state — no hand-coded heuristics.
    </p>
    <div class="project-links">
      <a href="/rl/snake-dqn/play" class="project-link">Play in browser</a>
      <a
        href="https://github.com/kashifulhaque/snake-rl"
        target="_blank"
        rel="noopener noreferrer"
        class="project-link"
      >
        Code
      </a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Overview</h2>
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
    </div>

    <div class="section">
      <h2>Results</h2>
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
    </div>

    <div class="section">
      <h2>Run it yourself</h2>
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
    </div>

    <div class="section">
      <h2>Notes &amp; next steps</h2>
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
    border-bottom: 1px solid var(--line-soft);
  }

  .breadcrumb {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-mute);
  }

  .breadcrumb a {
    color: var(--ink-mute);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .breadcrumb a:hover {
    color: var(--signal-hi);
  }

  .separator {
    margin: 0 0.5rem;
  }

  .page-title {
    font-family: var(--font-dots);
    font-size: clamp(2.4rem, 6vw, 4rem);
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--ink);
    margin: 0;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.6;
    color: var(--ink-soft);
    margin: 0;
  }

  .project-links {
    display: flex;
    gap: 1rem;
  }

  .project-link {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-soft);
    padding: 0.5rem 1rem;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    transition:
      color var(--dur-instant) var(--ease-out-quart),
      background-color var(--dur-instant) var(--ease-out-quart),
      border-color var(--dur-instant) var(--ease-out-quart);
  }

  .project-link:hover {
    background: var(--ink);
    color: var(--void);
    border-color: var(--ink);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2.5rem;
  }

  .section h2 {
    font-size: 1.5rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--ink);
    margin-bottom: 1rem;
  }

  .section p {
    font-size: 0.9375rem;
    line-height: 1.7;
    color: var(--ink-soft);
    margin-bottom: 1rem;
  }

  .section p:last-child {
    margin-bottom: 0;
  }

  .feature-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
    margin-bottom: 1rem;
  }

  .feature-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--ink-soft);
    padding-left: 1.5rem;
    position: relative;
  }

  .feature-list li::before {
    content: "→";
    position: absolute;
    left: 0;
    color: var(--signal);
  }

  .feature-list code,
  p code {
    font-family: var(--font-mono-g);
    font-size: 0.875rem;
    color: var(--ink-soft);
    background: var(--code-bg);
    border: 1px solid var(--line-soft);
    padding: 0.125rem 0.375rem;
    border-radius: var(--radius-sm);
  }

  .inline-link {
    color: var(--ink-soft);
    text-decoration: underline;
    text-decoration-color: var(--ink-mute);
    transition: all var(--dur-instant) var(--ease-out-quart);
  }

  .inline-link:hover {
    color: var(--signal);
    text-decoration-color: var(--signal);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1.25rem;
    margin-bottom: 1.5rem;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 1.25rem;
    background: var(--panel);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
  }

  .stat-value {
    font-family: var(--font-dots);
    font-size: 2rem;
    font-weight: 600;
    color: var(--ink);
    line-height: 1;
  }

  .stat-label {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-mute);
  }

  .code-block {
    background: var(--code-bg);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    overflow-x: auto;
  }

  .code-block pre {
    margin: 0;
    padding: 1.25rem;
  }

  .code-block code {
    font-family: var(--font-mono-g);
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--ink-soft);
    background: transparent;
    padding: 0;
  }

  @keyframes fade-up {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 768px) {
    .section h2 {
      font-size: 1.25rem;
    }

    .stats-grid {
      grid-template-columns: 1fr 1fr;
    }
  }
</style>
