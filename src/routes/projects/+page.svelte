<svelte:head>
  <title>Projects — Kashif</title>
  <meta name="description" content="Side projects and open source work." />
</svelte:head>

<script lang="ts">
  import { reveal, splitReveal } from '$lib/motion';
  type Project = {
    name: string;
    desc: string;
    links: { label: string; url: string }[];
    page?: string;
    status?: 'active' | 'shipped' | 'archived';
  };

  const projects: Project[] = [
    {
      name: 'PoliteLlama',
      desc: 'Llama 3.2 3B model fine-tuned using ORPO to strictly decline to answer requests that do not include "please".',
      page: '/projects/polite-orpo',
      status: 'active',
      links: [
        { label: 'Model', url: 'https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO' }
      ]
    },
    {
      name: 'banana.cpp',
      desc: 'Pure C++ LLM inference engine. SmolLM2, Llama 3.2, Qwen. Modular architecture with GQA, RoPE, SwiGLU.',
      page: '/projects/banana-cpp',
      status: 'active',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/banana.cpp' }
      ]
    },
    {
      name: 'smol-llama',
      desc: '360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, SwiGLU. Single H100, 22hrs, $53.',
      page: '/projects/smol-llama',
      status: 'active',
      links: [
        { label: 'Model', url: 'https://huggingface.co/weights-and-wires/smol-llama' },
        { label: 'Code', url: 'https://github.com/weights-and-wires/smol-llama' }
      ]
    },
    {
      name: 'Micro-Llama on Vicharak Shrike-Lite',
      desc: 'Train a tiny Llama on TinyStories, then run bare-metal inference on the Vicharak Shrike-Lite (RP2040 + FPGA, 264KB SRAM). 213K params, C firmware.',
      page: '/projects/vicharak',
      status: 'active',
      links: [
        { label: 'Code', url: 'https://github.com/weights-and-wires/vicharak-llm' },
        { label: 'Model', url: 'https://huggingface.co/weights-and-wires/vicharak-micro-llama' }
      ]
    },
    {
      name: 'smoltorch',
      desc: 'Autograd engine and neural networks in ~500 lines of NumPy. Educational deep learning.',
      page: '/projects/smoltorch',
      status: 'active',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/smoltorch' },
        { label: 'PyPI', url: 'https://pypi.org/project/smoltorch/' }
      ]
    },
    {
      name: 'NoPokeDB',
      desc: 'Lightweight vector DB with hnswlib + SQLite. Crash recovery, 2K+ PyPI downloads.',
      page: '/projects/nopokedb',
      status: 'active',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/nopokedb' },
        { label: 'PyPI', url: 'https://pypi.org/project/nopokedb/' }
      ]
    },
    {
      name: 'Boo',
      desc: 'AI Discord bot. Natural conversations, image understanding, and generation.',
      page: '/projects/boo',
      status: 'active',
      links: [
        { label: 'Code', url: 'https://github.com/VVIP-Kitchen/boo' },
        { label: 'Site', url: 'https://boo.ifkash.dev' }
      ]
    },
    {
      name: 'ferray',
      desc: 'NumPy-like ndarray in Rust with Python bindings. Learning project.',
      page: '/projects/ferray',
      status: 'active',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/ferray' }
      ]
    },
    {
      name: 'endark',
      desc: 'Monochrome, dark-only CSS library. Glassmorphism meets terminal-editorial aesthetic. Zero dependencies.',
      status: 'active',
      links: [
        { label: 'Site', url: 'https://endark.ifkash.dev' },
        { label: 'Code', url: 'https://github.com/kashifulhaque/endark' }
      ]
    },
    {
      name: 'Opencode theme for VS Code',
      desc: 'VS Code light and dark theme generated from OpenCode reference JSON.',
      status: 'active',
      links: [
        { label: 'Download', url: 'https://github.com/kashifulhaque/opencode-vscode-theme/releases/latest' },
        { label: 'Code', url: 'https://github.com/kashifulhaque/opencode-vscode-theme' }
      ]
    }
  ];

  const statusLabel = { active: 'Active', shipped: 'Shipped', archived: 'Archived' } as const;
  const pad = (n: number) => String(n).padStart(2, '0');
</script>

<header class="page-head">
  <p class="eyebrow">02 / Projects</p>
  <h1 class="page-title" use:splitReveal>Projects</h1>
  <p class="lead">Things I've built or am actively shaping</p>
</header>

<ol class="rows list" use:reveal={{ children: '.row', gap: 45 }}>
  {#each projects as project, i}
    <li class="row" class:linked={!!project.page}>
      <span class="row-n">{pad(i + 1)}</span>

      <div class="row-body">
        {#if project.page}
          <a href={project.page} class="row-title">
            <span class="row-name">{project.name}</span>
            <span class="row-arrow" aria-hidden="true">→</span>
          </a>
        {:else}
          <span class="row-title">
            <span class="row-name">{project.name}</span>
          </span>
        {/if}
        <p class="row-desc">{project.desc}</p>
      </div>

      <div class="row-meta">
        {#if project.status}
          <span class="tag status">
            <span
              class="led"
              class:led-hollow={project.status === 'archived'}
              aria-hidden="true"
            ></span>
            {statusLabel[project.status]}
          </span>
        {/if}
        {#if project.links.length}
          <ul class="row-links">
            {#each project.links as link}
              <li>
                <a href={link.url} target="_blank" rel="noopener noreferrer" class="row-link">
                  {link.label}<span aria-hidden="true">↗</span>
                </a>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </li>
  {/each}
</ol>

<style>
  .list {
    margin-top: var(--s-7);
  }

  .row {
    position: relative;
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) minmax(176px, auto);
    align-items: baseline;
    column-gap: var(--s-5);
    padding: var(--s-5) 0;
  }

  .row-n {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .row-body {
    display: grid;
    gap: var(--s-2);
    min-width: 0;
  }

  .row-title {
    display: inline-flex;
    align-items: baseline;
    gap: var(--s-2);
    justify-self: start;
    color: var(--ink);
    text-decoration: none;
  }
  /* Stretch the detail link over the whole row; the outbound links sit above it. */
  a.row-title::after {
    content: '';
    position: absolute;
    inset: 0;
  }

  .row-name {
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.25;
    letter-spacing: -0.015em;
    transition: color var(--dur-fast) var(--ease);
  }

  .row-arrow {
    color: var(--ink-3);
    font-size: var(--text-base);
    transition:
      transform var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease);
  }

  .row.linked:hover .row-name,
  .row.linked:hover .row-arrow {
    color: var(--fjord);
  }
  .row.linked:hover .row-arrow {
    transform: translateX(3px);
  }

  .row-desc {
    max-width: var(--measure);
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-base);
    line-height: 1.55;
  }

  .row-meta {
    display: grid;
    justify-items: end;
    align-content: start;
    gap: var(--s-3);
  }

  .status {
    color: var(--ink-2);
  }
  .status .led {
    width: 6px;
    height: 6px;
  }

  .row-links {
    position: relative;
    z-index: 1;
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--s-1) var(--s-4);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row-link {
    display: inline-flex;
    align-items: baseline;
    gap: 3px;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    text-decoration: none;
    transition: color var(--dur-fast) var(--ease);
  }
  .row-link span {
    color: var(--ink-3);
    transition: color var(--dur-fast) var(--ease);
  }
  .row-link:hover,
  .row-link:hover span {
    color: var(--fjord);
  }

  @media (max-width: 720px) {
    .list {
      margin-top: var(--s-6);
    }

    .row {
      grid-template-columns: 32px minmax(0, 1fr);
      row-gap: var(--s-3);
      column-gap: var(--s-3);
    }

    .row-meta {
      grid-column: 2;
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-items: start;
      gap: var(--s-3) var(--s-4);
    }

    .row-links {
      justify-content: flex-start;
    }

    .row-name {
      font-size: var(--text-md);
    }
  }
</style>
