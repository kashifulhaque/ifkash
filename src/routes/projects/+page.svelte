<svelte:head>
  <title>Projects — Kashiful Haque</title>
  <meta name="description" content="Side projects, open source work, and experiments in ML systems." />
</svelte:head>

<script lang="ts">
  type Project = {
    name: string;
    desc: string;
    links: { label: string; url: string }[];
    page?: string;
    surface: string;
    year: string;
  };

  const projects: Project[] = [
    {
      name: 'Llama-3.2-3B-Polite-ORPO',
      desc: 'Llama 3.2 3B fine-tuned with ORPO to strictly decline requests missing the word "please". A small but verifiable post-training experiment in shaping behaviour with explicit rewards.',
      page: '/projects/polite-orpo',
      surface: 'rl · post-training',
      year: '2024',
      links: [
        { label: 'Model', url: 'https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO' }
      ]
    },
    {
      name: 'banana.cpp',
      desc: 'Pure C++ LLM inference engine for SmolLM2, Llama 3.2, and Qwen. Modular architecture with GQA, RoPE, SwiGLU, and a clean kernel boundary so backends can swap underneath.',
      page: '/projects/banana-cpp',
      surface: 'runtime · inference',
      year: '2025',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/banana.cpp' }
      ]
    },
    {
      name: 'smol-llama',
      desc: '360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, SwiGLU. Single H100 in 22 hours for $53 — proof that frontier shapes are reachable on a budget.',
      page: '/projects/smol-llama',
      surface: 'pretraining · model',
      year: '2025',
      links: [
        { label: 'Model', url: 'https://huggingface.co/weights-and-wires/smol-llama' },
        { label: 'Code', url: 'https://github.com/weights-and-wires/smol-llama' }
      ]
    },
    {
      name: 'smoltorch',
      desc: 'An autograd engine and neural network library in roughly 500 lines of NumPy. Built to teach the gradients, not to hide them.',
      page: '/projects/smoltorch',
      surface: 'education · framework',
      year: '2024',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/smoltorch' },
        { label: 'PyPI', url: 'https://pypi.org/project/smoltorch/' }
      ]
    },
    {
      name: 'NoPokeDB',
      desc: 'A lightweight vector database backed by hnswlib and SQLite. Crash recovery, simple schema, 2K+ PyPI downloads from teams that wanted a vector store without the bill.',
      page: '/projects/nopokedb',
      surface: 'data · vector store',
      year: '2024',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/nopokedb' },
        { label: 'PyPI', url: 'https://pypi.org/project/nopokedb/' }
      ]
    },
    {
      name: 'Boo',
      desc: 'An AI Discord bot for natural conversations with image understanding and image generation. Multi-modal context wired into a small, opinionated runtime.',
      page: '/projects/boo',
      surface: 'agent · multimodal',
      year: '2024',
      links: [
        { label: 'Code', url: 'https://github.com/VVIP-Kitchen/boo' },
        { label: 'Site', url: 'https://boo.ifkash.dev' }
      ]
    },
    {
      name: 'ferray',
      desc: 'A NumPy-style ndarray in Rust with Python bindings. A learning project that is now very particular about how it owns memory.',
      page: '/projects/ferray',
      surface: 'systems · rust',
      year: '2024',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/ferray' }
      ]
    },
    {
      name: 'endark',
      desc: 'Monochrome, dark-only CSS library. Glassmorphism meets terminal-editorial aesthetic. Zero dependencies, maximum opinion.',
      surface: 'css · design system',
      year: '2024',
      links: [
        { label: 'Site', url: 'https://endark.ifkash.dev' },
        { label: 'Code', url: 'https://github.com/kashifulhaque/endark' }
      ]
    },
    {
      name: 'Opencode theme · VS Code',
      desc: 'VS Code light and dark themes generated from the OpenCode reference JSON. Color tokens that play well with code, not against it.',
      surface: 'tooling · ide',
      year: '2024',
      links: [
        { label: 'Download', url: 'https://github.com/kashifulhaque/opencode-vscode-theme/releases/latest' },
        { label: 'Code', url: 'https://github.com/kashifulhaque/opencode-vscode-theme' }
      ]
    }
  ];
</script>

<div class="page">
  <header class="page-header">
    <span class="eyebrow">section · 02 · projects</span>
    <h1 class="page-title">
      Things built<br />
      <em>because they should exist.</em>
    </h1>
    <p class="page-desc">
      Inference engines, foundation models, agent runtimes, and the small
      tools that make the work go faster. Open source where possible.
    </p>
  </header>

  <div class="legend">
    <span class="legend-item"><span class="dot dot-cyan"></span>shipping</span>
    <span class="legend-item"><span class="dot dot-amber"></span>iterating</span>
    <span class="legend-count">{projects.length} entries · sorted by recency</span>
  </div>

  <section class="projects">
    {#each projects as project, i}
      <article class="project stagger" style="--i: {i}">
        <div class="project-rail">
          <span class="project-num">[ 0{i + 1} ]</span>
          <span class="project-year">{project.year}</span>
        </div>

        <div class="project-body">
          <div class="project-top">
            {#if project.page}
              <a href={project.page} class="project-name-link">
                <h2 class="project-name">{project.name}</h2>
              </a>
            {:else}
              <h2 class="project-name">{project.name}</h2>
            {/if}
            <span class="project-surface">{project.surface}</span>
          </div>

          <p class="project-desc">{project.desc}</p>

          <div class="project-links">
            {#each project.links as link}
              <a href={link.url} target="_blank" rel="noopener noreferrer" class="project-link">
                <span>{link.label}</span>
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M7 17L17 7M9 7h8v8"/></svg>
              </a>
            {/each}
            {#if project.page}
              <a href={project.page} class="project-link project-link-internal">
                <span>Case</span>
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M5 12h14M13 5l7 7-7 7"/></svg>
              </a>
            {/if}
          </div>
        </div>
      </article>
    {/each}
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-2xl);
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--border);
    max-width: 760px;
  }

  .page-title {
    font-size: clamp(2.5rem, 5vw + 0.5rem, 4.25rem);
    letter-spacing: -0.04em;
    line-height: 1.05;
    color: var(--text-primary);
    font-weight: 400;
  }

  .page-title em {
    font-family: var(--font-serif);
    font-style: italic;
    font-weight: 300;
    color: var(--accent);
    letter-spacing: -0.02em;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.55;
    color: var(--text-tertiary);
    max-width: 56ch;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 1.25rem;
    padding: 0.75rem 1rem;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: 999px;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.14em;
  }

  .legend-item {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .dot-cyan { background: var(--accent); box-shadow: 0 0 6px var(--accent); }
  .dot-amber { background: var(--signal-warm); box-shadow: 0 0 6px var(--signal-warm); }

  .legend-count {
    margin-left: auto;
    color: var(--text-faint);
  }

  .projects {
    display: flex;
    flex-direction: column;
  }

  .project {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.75rem;
    padding: var(--space-xl) 0;
    border-bottom: 1px solid var(--border);
  }

  @media (min-width: 720px) {
    .project {
      grid-template-columns: 120px 1fr;
      gap: 2.5rem;
      padding: var(--space-xl) 0;
    }
  }

  .project:first-child {
    border-top: 1px solid var(--border);
  }

  .project-rail {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    align-items: center;
  }

  @media (min-width: 720px) {
    .project-rail {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.5rem;
      padding-top: 0.5rem;
    }
  }

  .project-num {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: var(--text-faint);
    letter-spacing: 0.06em;
  }

  .project-year {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .project-body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .project-top {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  @media (min-width: 480px) {
    .project-top {
      flex-direction: row;
      align-items: baseline;
      justify-content: space-between;
      gap: 1rem;
    }
  }

  .project-name {
    font-family: var(--font-sans);
    font-size: 1.5rem;
    font-weight: 500;
    color: var(--text-primary);
    letter-spacing: -0.025em;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .project-name-link:hover .project-name {
    color: var(--accent);
  }

  .project-surface {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text-faint);
    padding: 0.2rem 0.6rem;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: 999px;
    flex-shrink: 0;
  }

  .project-desc {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--text-tertiary);
    max-width: 64ch;
  }

  .project-links {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.25rem;
  }

  .project-link {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.85rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: 999px;
    transition: all var(--dur-instant) var(--ease-out-quart);
    letter-spacing: -0.005em;
  }

  .project-link:hover {
    color: var(--accent);
    border-color: var(--accent);
  }

  .project-link svg {
    color: var(--text-faint);
    transition: transform var(--dur-fast) var(--ease-out-quart),
                color var(--dur-instant) var(--ease-out-quart);
  }

  .project-link:hover svg {
    color: var(--accent);
    transform: translate(2px, -2px);
  }

  .project-link-internal {
    background: var(--accent-faint);
    color: var(--accent);
    border-color: transparent;
  }

  .project-link-internal:hover {
    background: var(--accent-faint);
  }

  .project-link-internal:hover svg {
    transform: translateX(3px);
  }
</style>
