<svelte:head>
  <title>Snake RL — DQN</title>
  <meta
    name="description"
    content="A snake agent trained from scratch with DQN: experience replay, a 256-unit network, and an 11-dim state. Best score 44."
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
      A snake agent trained from scratch with Deep Q-Networks (DQN) in PyTorch.
      The policy learns to survive, chase food, and avoid its own tail from raw
      game state — no hand-coded heuristics.
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
        −10 for dying, and 0 otherwise. A frame cap of <code>100 ×
        snake length</code> per episode stops the agent from looping forever
        once the snake gets long.
      </p>
      <p>
        The agent (<code>model.py</code>) is a plain
        <code>Linear_QNet</code>: <code>11 → 256 → 3</code> with ReLU,
        trained with MSELoss against a one-step bootstrap target
        (<code>r + γ·max Q(s′)</code>, γ = 0.9). Training uses two memories —
        a short-memory update after every step plus a replay-buffer update of
        1000 random samples each game (replay capacity 100k) — and an
        ε-greedy schedule that decays from 80/200 to a floor of 5/200 as
        games go on.
      </p>
    </div>

    <div class="section">
      <h2>Results</h2>
      <div class="stats-grid">
        <div class="stat">
          <div class="stat-value">44</div>
          <div class="stat-label">Record score</div>
        </div>
        <div class="stat">
          <div class="stat-value">11</div>
          <div class="stat-label">State dims</div>
        </div>
        <div class="stat">
          <div class="stat-value">3</div>
          <div class="stat-label">Actions</div>
        </div>
        <div class="stat">
          <div class="stat-value">256</div>
          <div class="stat-label">Hidden units</div>
        </div>
      </div>
      <p>
        The final weights are saved to <code>model.pth</code> (the checkpoint
        is re-saved every time a new record is set), and
        <code>play.py</code> loads it back to run evaluation games with
        <code>torch.no_grad()</code>. The training loop (<code>train.py</code>)
        also plots score and mean score live with matplotlib.
      </p>
      <p>
        The trained weights are also exported to
        <code>src/lib/rl/snake-dqn-weights.json</code> so the same agent can
        play in the browser — a TypeScript port of the env and forward pass
        (no training). <a href="/rl/snake-dqn/play" class="inline-link"
          >Watch it play live</a
        >.
      </p>
    </div>

    <div class="section">
      <h2>Run it yourself</h2>
      <div class="code-block">
        <pre><code>uv run train.py   # train a new agent (renders the game live)
uv run play.py    # watch the saved model play 10 games</code></pre>
      </div>
      <p>
        Dependencies: <code>torch</code>, <code>pygame</code>,
        <code>numpy</code>, <code>matplotlib</code> — pinned in
        <code>pyproject.toml</code> (Python ≥ 3.12, managed with uv).
      </p>
    </div>

    <div class="section">
      <h2>Notes &amp; next steps</h2>
      <ul class="feature-list">
        <li>
          <strong>Double DQN / target network:</strong> the current bootstrap
          target uses the same network being trained, which can overestimate
          Q-values. A frozen target network updated every N steps is the
          standard fix.
        </li>
        <li>
          <strong>Richer state:</strong> the 11-dim vector encodes danger in
          the three facing directions, the current direction, and food
          position relative to the head. Adding a vision grid or a full-board
          occupancy view would help it avoid dead ends.
        </li>
        <li>
          <strong>No render for training:</strong> the env supports
          <code>render=False</code>, so training can be run headless and
          batched up for faster iteration.
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
