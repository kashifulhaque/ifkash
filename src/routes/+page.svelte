<script lang="ts">
  import { auth } from "$lib/stores/auth";
  import { dev } from "$app/environment";
  import AgentMesh from "$lib/components/AgentMesh.svelte";
  import ExecutionStack from "$lib/components/ExecutionStack.svelte";
  import MetricStrip from "$lib/components/MetricStrip.svelte";

  const resumeUrl = dev
    ? "http://localhost:8787/api/resume?format=view"
    : "/api/resume?format=view";

  const metrics = [
    { value: "360", suffix: "M", label: "params trained · single H100" },
    { value: "142", suffix: "tok/s", label: "cpp inference · throughput" },
    { value: "6", suffix: "B", label: "tokens · pretrain corpus" },
    { value: "$53", label: "compute · end-to-end" },
  ];

  const capabilities = [
    {
      title: "Agents & orchestration",
      desc: "Multi-agent pipelines that plan, code, and verify across sandboxed compute. Governed by traces, not vibes.",
      tag: "agent.surface",
    },
    {
      title: "Foundation model training",
      desc: "Pretraining and post-training of compact LLaMA-style models from scratch. RL alignment with verifiable rewards.",
      tag: "model.kernel",
    },
    {
      title: "Inference at the metal",
      desc: "C++ and Rust inference engines: GQA, RoPE, SwiGLU, fused attention. CPU and CUDA backends.",
      tag: "runtime.core",
    },
    {
      title: "Developer surfaces",
      desc: "IDE extensions, TUIs, vector stores. Tools that move with the engineer, not against them.",
      tag: "surface.dev",
    },
  ];

  const selectedProjects = [
    {
      name: "banana.cpp",
      role: "C++ inference engine",
      year: "2025",
      desc: "Pure C++ runtime for SmolLM2, Llama 3.2, Qwen. GQA, RoPE, SwiGLU. Modular kernels.",
      href: "/projects/banana-cpp",
    },
    {
      name: "smol-llama",
      role: "360M LLaMA · trained from scratch",
      year: "2025",
      desc: "Single H100, 22 hours, $53. 6B token corpus. Open weights on Hugging Face.",
      href: "/projects/smol-llama",
    },
    {
      name: "Llama-3.2-3B-Polite-ORPO",
      role: "RL post-training experiment",
      year: "2024",
      desc: "ORPO fine-tune that strictly declines requests missing the word 'please'. Verifiable reward shaping.",
      href: "/projects/polite-orpo",
    },
  ];
</script>

<svelte:head>
  <title>Kashiful Haque · ML Systems & Agent Infrastructure</title>
  <meta
    name="description"
    content="ML Engineer building foundation models, low-level inference engines, and agent infrastructure. Pretraining, RL, and high-performance runtimes in C++ and Rust."
  />
</svelte:head>

<div class="home">
  <!-- ───────── HERO ───────── -->
  <section class="hero">
    <div class="hero-text">
      <span class="eyebrow">v.04 · ml engineer · wand ai</span>

      <h1 class="hero-name">
        I build the&nbsp;<em>models.</em><br />
        Then I make them <em>fly.</em>
      </h1>

      <p class="hero-bio">
        Foundation models, low-level inference engines, and the agent
        infrastructure that wraps them. Pretraining transformers from
        scratch on single GPUs. Shipping inference in C++ and Rust where
        every microsecond is accounted for.
      </p>

      <div class="hero-cta">
        <a href={resumeUrl} class="btn-primary" target="_blank" rel="noopener noreferrer">
          Read the resume
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M7 17L17 7M9 7h8v8"/></svg>
        </a>
        <a href="/projects" class="btn-ghost">
          Tour the work
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M5 12h14M13 5l7 7-7 7"/></svg>
        </a>
        {#if $auth.isAuthenticated}
          <a href="/editor" class="btn-ghost">Open editor</a>
        {/if}
      </div>

      <dl class="hero-meta">
        <div>
          <dt>Currently</dt>
          <dd>ML Engineer · <a href="https://wand.ai" target="_blank" rel="noopener noreferrer">Wand AI</a></dd>
        </div>
        <div>
          <dt>Latest ship</dt>
          <dd>
            <a
              href="https://marketplace.visualstudio.com/items?itemName=ifkash.kernel-orbit"
              target="_blank"
              rel="noopener noreferrer"
            >Kernel Orbit · VS Code extension</a>
          </dd>
        </div>
        <div>
          <dt>Located</dt>
          <dd>Palo Alto, CA · remote-first</dd>
        </div>
      </dl>
    </div>

    <aside class="hero-mesh">
      <AgentMesh />
    </aside>
  </section>

  <!-- ───────── METRICS ───────── -->
  <section class="metrics">
    <MetricStrip {metrics} />
  </section>

  <!-- ───────── STACK / EXECUTION ───────── -->
  <section class="section">
    <div class="section-head">
      <span class="eyebrow">02 · the stack</span>
      <h2 class="section-title">
        Outcomes inside the boundary.<br />
        <span class="serif-italic">Not tokens, traces.</span>
      </h2>
      <p class="section-lede">
        Every model I ship runs through three surfaces: a sandbox where the
        agent moves freely, an evaluator that gates with verifiable rewards,
        and a review log that stays auditable end to end.
      </p>
    </div>

    <ExecutionStack />
  </section>

  <!-- ───────── CAPABILITIES ───────── -->
  <section class="section">
    <div class="section-head">
      <span class="eyebrow">03 · capabilities</span>
      <h2 class="section-title">
        Four surfaces.<br />
        <span class="serif-italic">One discipline.</span>
      </h2>
    </div>

    <div class="cap-grid">
      {#each capabilities as cap, i}
        <article class="cap-card stagger" style="--i: {i}">
          <div class="cap-top">
            <span class="cap-num">0{i + 1}</span>
            <span class="cap-tag">{cap.tag}</span>
          </div>
          <h3 class="cap-title">{cap.title}</h3>
          <p class="cap-desc">{cap.desc}</p>
        </article>
      {/each}
    </div>
  </section>

  <!-- ───────── SELECTED PROJECTS ───────── -->
  <section class="section">
    <div class="section-head row">
      <div>
        <span class="eyebrow">04 · selected work</span>
        <h2 class="section-title">
          Things I shipped<br />
          <span class="serif-italic">because they should exist.</span>
        </h2>
      </div>
      <a href="/projects" class="head-link">
        All projects
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M5 12h14M13 5l7 7-7 7"/></svg>
      </a>
    </div>

    <ul class="proj-list">
      {#each selectedProjects as p, i}
        <li class="proj-row stagger" style="--i: {i}">
          <a href={p.href} class="proj-link">
            <div class="proj-num">[ 0{i + 1} ]</div>
            <div class="proj-main">
              <div class="proj-head">
                <h3 class="proj-name">{p.name}</h3>
                <span class="proj-role">{p.role}</span>
              </div>
              <p class="proj-desc">{p.desc}</p>
            </div>
            <div class="proj-aside">
              <span class="proj-year">{p.year}</span>
              <span class="proj-arrow">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"><path d="M7 17L17 7M9 7h8v8"/></svg>
              </span>
            </div>
          </a>
        </li>
      {/each}
    </ul>
  </section>

  <!-- ───────── CTA STRIP ───────── -->
  <section class="cta">
    <div class="cta-inner">
      <div>
        <span class="eyebrow">05 · channels</span>
        <h2 class="cta-title">
          Working on something hard?<br />
          <span class="serif-italic">Let's compare notes.</span>
        </h2>
      </div>
      <div class="cta-links">
        <a href="https://github.com/kashifulhaque" target="_blank" rel="noopener noreferrer" class="cta-link">
          <span class="cta-link-label">GitHub</span>
          <span class="cta-link-handle">@kashifulhaque</span>
        </a>
        <a href="https://hf.co/ifkash" target="_blank" rel="noopener noreferrer" class="cta-link">
          <span class="cta-link-label">Hugging Face</span>
          <span class="cta-link-handle">@ifkash</span>
        </a>
        <a href="https://linkedin.com/in/kashifulhaque" target="_blank" rel="noopener noreferrer" class="cta-link">
          <span class="cta-link-label">LinkedIn</span>
          <span class="cta-link-handle">in/kashifulhaque</span>
        </a>
      </div>
    </div>
  </section>
</div>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: var(--space-3xl);
  }

  /* ───────── HERO ───────── */

  .hero {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-2xl);
    padding-top: var(--space-xl);
    align-items: center;
  }

  @media (min-width: 960px) {
    .hero {
      grid-template-columns: 1.25fr 1fr;
      gap: var(--space-3xl);
      padding-top: var(--space-2xl);
    }
  }

  .hero-text {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .hero-name {
    font-size: clamp(2.75rem, 6vw + 0.5rem, 5.25rem);
    line-height: 1.02;
    letter-spacing: -0.045em;
    color: var(--text-primary);
    margin-top: 0.25rem;
    font-weight: 400;
  }

  .hero-name em {
    font-family: var(--font-serif);
    font-style: italic;
    font-weight: 300;
    letter-spacing: -0.02em;
    color: var(--accent);
  }

  .hero-bio {
    font-size: 1.0625rem;
    line-height: 1.6;
    color: var(--text-secondary);
    max-width: 52ch;
  }

  .hero-cta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .hero-meta {
    margin: var(--space-md) 0 0;
    padding-top: var(--space-lg);
    border-top: 1px solid var(--border);
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.75rem;
  }

  @media (min-width: 640px) {
    .hero-meta {
      grid-template-columns: repeat(3, 1fr);
      gap: var(--space-lg);
    }
  }

  .hero-meta > div {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .hero-meta dt {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--text-faint);
  }

  .hero-meta dd {
    margin: 0;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .hero-meta dd a {
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
    transition: color var(--dur-instant), border-color var(--dur-instant);
  }

  .hero-meta dd a:hover {
    color: var(--accent);
    border-color: var(--accent);
  }

  .hero-mesh {
    width: 100%;
    max-width: 520px;
    margin: 0 auto;
  }

  @media (min-width: 960px) {
    .hero-mesh {
      max-width: none;
    }
  }

  /* ───────── METRICS ───────── */

  .metrics {
    margin-top: var(--space-md);
  }

  /* ───────── SECTION SHARED ───────── */

  .section {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  .section-head {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    max-width: 720px;
  }

  .section-head.row {
    flex-direction: column;
    gap: var(--space-md);
    max-width: none;
  }

  @media (min-width: 720px) {
    .section-head.row {
      flex-direction: row;
      align-items: flex-end;
      justify-content: space-between;
      gap: var(--space-xl);
    }

    .section-head.row > div {
      max-width: 640px;
    }
  }

  .section-title {
    font-size: clamp(2rem, 4vw + 0.5rem, 3.25rem);
    letter-spacing: -0.035em;
    color: var(--text-primary);
    line-height: 1.05;
    font-weight: 400;
  }

  .section-title :global(.serif-italic) {
    color: var(--text-tertiary);
  }

  .section-lede {
    font-size: 1rem;
    line-height: 1.6;
    color: var(--text-tertiary);
    max-width: 60ch;
  }

  .head-link {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text-secondary);
    padding: 0.5rem 0.85rem;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface-glass);
    transition: all var(--dur-fast) var(--ease-out-quart);
    white-space: nowrap;
  }

  .head-link:hover {
    color: var(--accent);
    border-color: var(--accent);
  }

  .head-link svg {
    transition: transform var(--dur-fast) var(--ease-out-quart);
  }

  .head-link:hover svg {
    transform: translate(2px, -2px);
  }

  /* ───────── CAPABILITIES ───────── */

  .cap-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-md);
  }

  @media (min-width: 720px) {
    .cap-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  .cap-card {
    position: relative;
    padding: 1.6rem 1.5rem;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    transition: border-color var(--dur-fast), transform var(--dur-fast);
    overflow: hidden;
  }

  .cap-card::after {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--accent), transparent);
    opacity: 0;
    transition: opacity var(--dur-fast);
  }

  .cap-card:hover {
    border-color: var(--border-bright);
    transform: translateY(-2px);
  }

  .cap-card:hover::after {
    opacity: 0.7;
  }

  .cap-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.85rem;
  }

  .cap-num {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--accent);
    letter-spacing: 0.05em;
  }

  .cap-tag {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text-faint);
    padding: 0.25rem 0.55rem;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: 999px;
  }

  .cap-title {
    font-size: 1.25rem;
    font-weight: 500;
    color: var(--text-primary);
    letter-spacing: -0.02em;
    margin-bottom: 0.5rem;
  }

  .cap-desc {
    font-size: 0.9375rem;
    line-height: 1.55;
    color: var(--text-tertiary);
  }

  /* ───────── PROJECTS ───────── */

  .proj-list {
    list-style: none;
    padding: 0;
    margin: 0;
    border-top: 1px solid var(--border);
  }

  .proj-row {
    border-bottom: 1px solid var(--border);
  }

  .proj-link {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 1rem;
    align-items: start;
    padding: 1.5rem 0;
    color: inherit;
    transition: padding var(--dur-fast), background var(--dur-fast);
  }

  @media (min-width: 720px) {
    .proj-link {
      gap: 2rem;
      padding: 1.75rem 1.25rem;
      margin-left: -1.25rem;
      margin-right: -1.25rem;
      border-radius: var(--radius-md);
    }
    .proj-link:hover {
      background: var(--surface-mesh);
    }
  }

  .proj-num {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: var(--text-faint);
    letter-spacing: 0.06em;
    padding-top: 0.5rem;
  }

  .proj-link:hover .proj-num {
    color: var(--accent);
  }

  .proj-main {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
  }

  .proj-head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.75rem;
  }

  .proj-name {
    font-size: 1.25rem;
    font-weight: 500;
    color: var(--text-primary);
    letter-spacing: -0.02em;
    transition: color var(--dur-fast);
  }

  .proj-link:hover .proj-name {
    color: var(--accent);
  }

  .proj-role {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--text-faint);
  }

  .proj-desc {
    font-size: 0.9375rem;
    line-height: 1.55;
    color: var(--text-tertiary);
    max-width: 56ch;
  }

  .proj-aside {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding-top: 0.5rem;
  }

  .proj-year {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .proj-arrow {
    color: var(--text-faint);
    transition: color var(--dur-fast), transform var(--dur-fast);
  }

  .proj-link:hover .proj-arrow {
    color: var(--accent);
    transform: translate(3px, -3px);
  }

  /* ───────── CTA ───────── */

  .cta {
    position: relative;
    padding: var(--space-xl) 1.75rem;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    overflow: hidden;
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
  }

  .cta::before {
    content: "";
    position: absolute;
    top: -120px;
    right: -120px;
    width: 320px;
    height: 320px;
    background: radial-gradient(circle, var(--accent-glow), transparent 65%);
    pointer-events: none;
    filter: blur(24px);
  }

  @media (min-width: 720px) {
    .cta {
      padding: var(--space-2xl) var(--space-2xl);
    }
  }

  .cta-inner {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  @media (min-width: 900px) {
    .cta-inner {
      flex-direction: row;
      align-items: flex-end;
      justify-content: space-between;
      gap: var(--space-2xl);
    }
  }

  .cta-title {
    font-size: clamp(1.75rem, 3vw + 0.5rem, 2.5rem);
    letter-spacing: -0.03em;
    margin-top: 0.75rem;
    line-height: 1.1;
    font-weight: 400;
  }

  .cta-title :global(.serif-italic) {
    color: var(--accent);
  }

  .cta-links {
    display: flex;
    flex-direction: column;
    gap: 0;
    border-top: 1px solid var(--border);
    min-width: 280px;
  }

  .cta-link {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 0.85rem 0;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    transition: color var(--dur-fast), padding var(--dur-fast);
  }

  .cta-link:hover {
    color: var(--accent);
    padding-left: 0.5rem;
  }

  .cta-link-label {
    font-size: 0.9375rem;
    font-weight: 500;
  }

  .cta-link-handle {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-faint);
    letter-spacing: -0.005em;
  }

  .cta-link:hover .cta-link-handle {
    color: var(--accent);
  }
</style>
