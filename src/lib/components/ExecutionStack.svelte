<script lang="ts">
  type Surface = {
    title: string;
    subtitle: string;
    accent: 'cyan' | 'violet' | 'amber';
    lines: { kind: 'in' | 'out' | 'pass' | 'fail' | 'note'; text: string }[];
  };

  export let surfaces: Surface[] = [
    {
      title: 'sandbox.ts',
      subtitle: 'agent.code · isolated',
      accent: 'cyan',
      lines: [
        { kind: 'in', text: '$ run inference --model 360M --batch 4' },
        { kind: 'note', text: 'compiling kernels  fused-attn · rmsnorm' },
        { kind: 'pass', text: 'tokens/s  142.7   p99 14ms   mem 1.8gb' },
      ],
    },
    {
      title: 'evals.spec',
      subtitle: 'agent.eval · verifier',
      accent: 'violet',
      lines: [
        { kind: 'in', text: '> assert latency p99 < 20ms' },
        { kind: 'pass', text: 'passed  14ms  ✓' },
        { kind: 'in', text: '> assert exact-match >= 0.92' },
        { kind: 'pass', text: 'passed  0.943  ✓' },
        { kind: 'note', text: '23 / 23 verified · sealing artifact' },
      ],
    },
    {
      title: 'trace.log',
      subtitle: 'agent.review · governance',
      accent: 'amber',
      lines: [
        { kind: 'note', text: 'orchestrator → agent.plan' },
        { kind: 'out', text: 'plan: refactor inference loop · 4 steps' },
        { kind: 'note', text: 'agent.plan → agent.code' },
        { kind: 'out', text: 'diff +124 / −38 · sandbox sealed' },
        { kind: 'note', text: 'review: cleared · escalation = none' },
      ],
    },
  ];
</script>

<div class="stack">
  {#each surfaces as s, i}
    <article class="surface accent-{s.accent} stagger" style="--i: {i}">
      <header class="surface-head">
        <div class="dots">
          <span></span><span></span><span></span>
        </div>
        <div class="head-meta">
          <span class="head-title">{s.title}</span>
          <span class="head-sub">{s.subtitle}</span>
        </div>
        <div class="head-status">
          <span class="status-dot"></span>
          <span class="status-text">live</span>
        </div>
      </header>

      <div class="body">
        {#each s.lines as line, j}
          <div class="line line-{line.kind}" style="animation-delay: {(i * 0.4 + j * 0.18)}s">
            <span class="prefix">
              {#if line.kind === 'in'}
                <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2 1.5l3 2.5-3 2.5"/></svg>
              {:else if line.kind === 'out'}
                <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6 1.5l-3 2.5 3 2.5"/></svg>
              {:else if line.kind === 'pass'}
                <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M1 4l2 2 4-4"/></svg>
              {:else if line.kind === 'fail'}
                <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M1.5 1.5l5 5M6.5 1.5l-5 5"/></svg>
              {:else}
                <svg width="8" height="8" viewBox="0 0 8 8" fill="none"><circle cx="4" cy="4" r="1.2" fill="currentColor"/></svg>
              {/if}
            </span>
            <span class="text">{line.text}</span>
          </div>
        {/each}
      </div>
    </article>
  {/each}
</div>

<style>
  .stack {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-md);
  }

  @media (min-width: 720px) {
    .stack {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .surface {
    position: relative;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    transition: border-color var(--dur-fast) var(--ease-out-quart),
                transform var(--dur-fast) var(--ease-out-quart);
  }

  .surface::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: linear-gradient(180deg, var(--accent-glow) 0%, transparent 25%);
    opacity: 0.5;
  }

  .surface:hover {
    border-color: var(--border-bright);
    transform: translateY(-2px);
  }

  .accent-cyan { --surf-accent: var(--accent); }
  .accent-violet { --surf-accent: var(--signal-violet); }
  .accent-amber { --surf-accent: var(--signal-warm); }

  .surface-head {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.65rem 0.9rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--surface-mesh);
  }

  .dots {
    display: flex;
    gap: 4px;
  }

  .dots span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--border-strong);
  }

  .dots span:first-child { background: var(--surf-accent); opacity: 0.85; }

  .head-meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.05rem;
    min-width: 0;
  }

  .head-title {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .head-sub {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--text-faint);
  }

  .head-status {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--surf-accent);
    box-shadow: 0 0 8px var(--surf-accent);
    animation: pulse-soft 1.6s ease-in-out infinite;
  }

  .status-text {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--text-tertiary);
  }

  .body {
    padding: 0.9rem 0.9rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    min-height: 170px;
  }

  .line {
    display: flex;
    align-items: flex-start;
    gap: 0.55rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    line-height: 1.45;
    color: var(--text-secondary);
    opacity: 0;
    animation: enter var(--dur-slow) var(--ease-out-expo) both;
  }

  .prefix {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 12px;
    height: 14px;
    margin-top: 1px;
  }

  .line-in .prefix { color: var(--surf-accent); }
  .line-out .prefix { color: var(--text-tertiary); }
  .line-pass .prefix { color: var(--accent-bright); }
  .line-fail .prefix { color: var(--signal-warm); }
  .line-note .prefix { color: var(--text-faint); }

  .line-pass .text { color: var(--text-primary); }
  .line-fail .text { color: var(--signal-warm); }
  .line-note .text { color: var(--text-tertiary); font-style: italic; opacity: 0.85; }
  .line-in .text { color: var(--text-primary); }
</style>
