<script lang="ts">
  type Node = { id: string; x: number; y: number; r: number; label?: string; kind?: 'agent' | 'tool' | 'core' | 'signal' };
  type Edge = { from: string; to: string; phase: number };

  const nodes: Node[] = [
    { id: 'core', x: 50, y: 50, r: 7, kind: 'core', label: 'orchestrator' },
    { id: 'a1', x: 18, y: 22, r: 4, kind: 'agent', label: 'agent.plan' },
    { id: 'a2', x: 82, y: 18, r: 4, kind: 'agent', label: 'agent.code' },
    { id: 'a3', x: 88, y: 70, r: 4, kind: 'agent', label: 'agent.eval' },
    { id: 'a4', x: 14, y: 78, r: 4, kind: 'agent', label: 'agent.review' },
    { id: 't1', x: 35, y: 8, r: 2.4, kind: 'tool' },
    { id: 't2', x: 64, y: 6, r: 2.4, kind: 'tool' },
    { id: 't3', x: 95, y: 44, r: 2.4, kind: 'tool' },
    { id: 't4', x: 64, y: 92, r: 2.4, kind: 'tool' },
    { id: 't5', x: 32, y: 92, r: 2.4, kind: 'tool' },
    { id: 't6', x: 5, y: 50, r: 2.4, kind: 'tool' },
    { id: 's1', x: 50, y: 22, r: 1.6, kind: 'signal' },
    { id: 's2', x: 72, y: 50, r: 1.6, kind: 'signal' },
    { id: 's3', x: 50, y: 78, r: 1.6, kind: 'signal' },
    { id: 's4', x: 28, y: 50, r: 1.6, kind: 'signal' },
  ];

  const edges: Edge[] = [
    { from: 'core', to: 'a1', phase: 0 },
    { from: 'core', to: 'a2', phase: 0.6 },
    { from: 'core', to: 'a3', phase: 1.2 },
    { from: 'core', to: 'a4', phase: 1.8 },
    { from: 'a1', to: 't1', phase: 2.4 },
    { from: 'a1', to: 't6', phase: 3.0 },
    { from: 'a2', to: 't2', phase: 3.6 },
    { from: 'a2', to: 't3', phase: 4.2 },
    { from: 'a3', to: 't3', phase: 4.8 },
    { from: 'a3', to: 't4', phase: 5.4 },
    { from: 'a4', to: 't5', phase: 6.0 },
    { from: 'a4', to: 't6', phase: 6.6 },
    { from: 'a1', to: 's1', phase: 0.3 },
    { from: 'a2', to: 's2', phase: 0.9 },
    { from: 'a3', to: 's3', phase: 1.5 },
    { from: 'a4', to: 's4', phase: 2.1 },
    { from: 's1', to: 's2', phase: 1.0 },
    { from: 's2', to: 's3', phase: 2.0 },
    { from: 's3', to: 's4', phase: 3.0 },
    { from: 's4', to: 's1', phase: 4.0 },
  ];

  function nodeById(id: string) {
    return nodes.find((n) => n.id === id)!;
  }
</script>

<div class="mesh">
  <svg viewBox="0 0 100 100" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
    <defs>
      <radialGradient id="coreGlow" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="var(--accent-bright)" stop-opacity="0.9" />
        <stop offset="40%" stop-color="var(--accent)" stop-opacity="0.5" />
        <stop offset="100%" stop-color="var(--accent)" stop-opacity="0" />
      </radialGradient>
      <radialGradient id="agentGlow" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.7" />
        <stop offset="100%" stop-color="var(--accent)" stop-opacity="0" />
      </radialGradient>
      <linearGradient id="traceLine" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0%" stop-color="var(--accent)" stop-opacity="0" />
        <stop offset="50%" stop-color="var(--accent-bright)" stop-opacity="0.9" />
        <stop offset="100%" stop-color="var(--accent)" stop-opacity="0" />
      </linearGradient>
    </defs>

    <!-- Concentric rings -->
    <g class="rings">
      <circle cx="50" cy="50" r="46" />
      <circle cx="50" cy="50" r="34" />
      <circle cx="50" cy="50" r="22" />
      <circle cx="50" cy="50" r="12" />
    </g>

    <!-- Core glow -->
    <circle cx="50" cy="50" r="22" fill="url(#coreGlow)" opacity="0.55" />

    <!-- Static base edges -->
    <g class="edges-base">
      {#each edges as e}
        {@const a = nodeById(e.from)}
        {@const b = nodeById(e.to)}
        <line x1={a.x} y1={a.y} x2={b.x} y2={b.y} />
      {/each}
    </g>

    <!-- Animated traces -->
    <g class="edges-trace">
      {#each edges as e, i}
        {@const a = nodeById(e.from)}
        {@const b = nodeById(e.to)}
        <line
          x1={a.x}
          y1={a.y}
          x2={b.x}
          y2={b.y}
          stroke="url(#traceLine)"
          style="--phase: {e.phase}s; --i: {i};"
        />
      {/each}
    </g>

    <!-- Nodes -->
    <g class="nodes">
      {#each nodes as n}
        {#if n.kind === 'core'}
          <circle class="node node-core" cx={n.x} cy={n.y} r={n.r} fill="url(#agentGlow)" />
          <circle class="node node-core-dot" cx={n.x} cy={n.y} r="1.6" />
        {:else if n.kind === 'agent'}
          <circle class="node-halo" cx={n.x} cy={n.y} r={n.r + 2} />
          <circle class="node node-agent" cx={n.x} cy={n.y} r={n.r} />
          <circle class="node-pulse" cx={n.x} cy={n.y} r="1.4" style="--delay: {n.x * 0.01}s;" />
        {:else if n.kind === 'tool'}
          <rect
            class="node node-tool"
            x={n.x - n.r}
            y={n.y - n.r}
            width={n.r * 2}
            height={n.r * 2}
            rx="0.5"
          />
        {:else}
          <circle class="node node-signal" cx={n.x} cy={n.y} r={n.r} />
        {/if}
      {/each}
    </g>

    <!-- Labels -->
    <g class="labels">
      <text x="50" y="51.4" class="lbl-core">orchestrator</text>
      <text x="18" y="16" class="lbl-agent">agent.plan</text>
      <text x="82" y="12" class="lbl-agent" text-anchor="end">agent.code</text>
      <text x="88" y="76" class="lbl-agent" text-anchor="end">agent.eval</text>
      <text x="14" y="84" class="lbl-agent">agent.review</text>
    </g>
  </svg>

  <div class="frame-corner tl"></div>
  <div class="frame-corner tr"></div>
  <div class="frame-corner bl"></div>
  <div class="frame-corner br"></div>

  <div class="meta">
    <span class="meta-dot"></span>
    <span class="meta-text">live trace</span>
  </div>
</div>

<style>
  .mesh {
    position: relative;
    width: 100%;
    aspect-ratio: 1 / 1;
    border-radius: var(--radius-lg);
    background:
      radial-gradient(circle at 50% 50%, var(--surface-mesh), transparent 70%),
      var(--surface-glass);
    border: 1px solid var(--border);
    overflow: hidden;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  svg {
    width: 100%;
    height: 100%;
    display: block;
  }

  .rings circle {
    fill: none;
    stroke: var(--border-strong);
    stroke-width: 0.08;
    opacity: 0.45;
  }

  .edges-base line {
    stroke: var(--border-strong);
    stroke-width: 0.12;
    opacity: 0.45;
  }

  .edges-trace line {
    stroke-width: 0.22;
    stroke-dasharray: 12 60;
    stroke-dashoffset: 0;
    opacity: 0;
    animation: trace-flow 4.5s linear infinite;
    animation-delay: var(--phase, 0s);
  }

  .nodes circle,
  .nodes rect {
    transition: all var(--dur-fast) var(--ease-out-quart);
  }

  .node-core {
    fill: var(--accent);
    opacity: 0.4;
  }

  .node-core-dot {
    fill: var(--accent-bright);
    filter: drop-shadow(0 0 1.5px var(--accent));
  }

  .node-agent {
    fill: var(--paper);
    stroke: var(--accent);
    stroke-width: 0.25;
  }

  .node-halo {
    fill: var(--accent);
    opacity: 0.12;
  }

  .node-pulse {
    fill: var(--accent-bright);
    animation: mesh-pulse 2.5s ease-in-out infinite;
    animation-delay: var(--delay, 0s);
  }

  .node-tool {
    fill: var(--paper-warm);
    stroke: var(--text-tertiary);
    stroke-width: 0.15;
  }

  .node-signal {
    fill: var(--signal);
    opacity: 0.6;
  }

  .lbl-core {
    font-family: var(--font-mono);
    font-size: 1.3px;
    fill: var(--text-primary);
    text-anchor: middle;
    font-weight: 600;
    letter-spacing: 0.06em;
  }

  .lbl-agent {
    font-family: var(--font-mono);
    font-size: 1.2px;
    fill: var(--text-secondary);
    letter-spacing: 0.04em;
  }

  .frame-corner {
    position: absolute;
    width: 18px;
    height: 18px;
    border-color: var(--accent);
    opacity: 0.7;
  }
  .frame-corner.tl { top: 14px; left: 14px; border-top: 1px solid; border-left: 1px solid; }
  .frame-corner.tr { top: 14px; right: 14px; border-top: 1px solid; border-right: 1px solid; }
  .frame-corner.bl { bottom: 14px; left: 14px; border-bottom: 1px solid; border-left: 1px solid; }
  .frame-corner.br { bottom: 14px; right: 14px; border-bottom: 1px solid; border-right: 1px solid; }

  .meta {
    position: absolute;
    bottom: 18px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 0.75rem;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: 999px;
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
  }

  .meta-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent-bright);
    box-shadow: 0 0 8px var(--accent);
    animation: pulse-soft 1.6s ease-in-out infinite;
  }

  .meta-text {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--text-secondary);
  }
</style>
