<svelte:head>
  <title>Projects — Kashif</title>
  <meta name="description" content="Side projects and open source work." />
</svelte:head>

<script lang="ts">
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
</script>

<header class="page-header">
  <h1 class="section-title">Projects</h1>
  <p class="section-subtitle">Things I've built or am actively shaping</p>
</header>

<section class="projects-list">
  {#each projects as project, i}
    <article class="project-row stagger" style="--i: {i}">
      <span
        class="project-led"
        class:lit={project.status === 'active'}
        class:shipped={project.status === 'shipped'}
        class:archived={project.status === 'archived'}
        aria-hidden="true"
      ></span>
      <div class="project-body">
        <div class="project-top">
          {#if project.page}
            <a href={project.page} class="project-name-link">
              <span class="project-name">{project.name}</span>
            </a>
          {:else}
            <span class="project-name">{project.name}</span>
          {/if}
          <div class="project-links">
            {#each project.links as link}
              <a
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                class="project-link"
              >
                {link.label}
              </a>
            {/each}
          </div>
        </div>
        <p class="project-desc">{project.desc}</p>
      </div>
    </article>
  {/each}
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .projects-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--line-soft);
  }

  .project-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr);
    align-items: start;
    gap: 20px;
    padding: 22px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--line-soft);
    transition: background 0.15s;
  }

  .project-row:hover {
    background: var(--blueprint-tint);
  }

  .project-led {
    width: 6px;
    height: 6px;
    border-radius: 0;
    border: 1px solid var(--ink-mute);
    background: transparent;
    transform: translateY(10px);
    justify-self: center;
    flex-shrink: 0;
  }

  .project-led.lit {
    background: var(--accent);
    border-color: var(--accent);
    box-shadow: none;
    animation: none;
  }

  .project-led.shipped {
    background: var(--ok);
    border-color: var(--ok);
    box-shadow: none;
  }

  .project-led.archived {
    background: transparent;
    border-style: dashed;
    border-color: var(--ink-mute);
  }

  .project-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }

  .project-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .project-name {
    font-family: var(--font-serif);
    font-size: 1.5rem;
    font-weight: 400;
    letter-spacing: -0.025em;
    text-transform: none;
    color: var(--foreground);
    line-height: 1.2;
  }

  .project-name-link {
    border-bottom: none;
  }

  .project-name-link:hover .project-name {
    color: var(--accent-2);
  }

  .project-links {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .project-link {
    font-family: var(--font-label);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    padding: 5px 11px;
    background: transparent;
    color: var(--muted-foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    text-decoration: none;
    transition:
      color 0.15s var(--ease-inout),
      border-color 0.15s var(--ease-inout);
  }

  .project-link:hover {
    color: var(--foreground);
    background: transparent;
    border-color: var(--border-strong);
  }

  .project-desc {
    font-family: var(--font-sans);
    font-size: 0.95rem;
    line-height: 1.6;
    color: var(--muted-foreground);
    max-width: 720px;
  }

  @media (max-width: 768px) {
    .project-row {
      gap: 16px;
      padding: 18px 10px;
      margin: 0 -10px;
    }

    .project-name {
      font-size: 1.5rem;
    }

    .project-top {
      flex-direction: column;
      align-items: flex-start;
      gap: 10px;
    }
  }
</style>
