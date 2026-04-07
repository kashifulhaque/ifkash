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
  };

  const projects: Project[] = [
    {
      name: 'Llama-3.2-3B-Polite-ORPO',
      desc: 'Llama 3.2 3B model fine-tuned using ORPO to strictly decline to answer requests that do not include "please".',
      page: '/projects/polite-orpo',
      links: [
        { label: 'Model', url: 'https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO' }
      ]
    },
    {
      name: 'banana.cpp',
      desc: 'Pure C++ LLM inference engine. SmolLM2, Llama 3.2, Qwen. Modular architecture with GQA, RoPE, SwiGLU.',
      page: '/projects/banana-cpp',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/banana.cpp' }
      ]
    },
    {
      name: 'smol-llama',
      desc: '360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, SwiGLU. Single H100, 22hrs, $53.',
      page: '/projects/smol-llama',
      links: [
        { label: 'Model', url: 'https://huggingface.co/weights-and-wires/smol-llama' },
        { label: 'Code', url: 'https://github.com/weights-and-wires/smol-llama' }
      ]
    },
    {
      name: 'smoltorch',
      desc: 'Autograd engine and neural networks in ~500 lines of NumPy. Educational deep learning.',
      page: '/projects/smoltorch',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/smoltorch' },
        { label: 'PyPI', url: 'https://pypi.org/project/smoltorch/' }
      ]
    },
    {
      name: 'NoPokeDB',
      desc: 'Lightweight vector DB with hnswlib + SQLite. Crash recovery, 2K+ PyPI downloads.',
      page: '/projects/nopokedb',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/nopokedb' },
        { label: 'PyPI', url: 'https://pypi.org/project/nopokedb/' }
      ]
    },
    {
      name: 'Boo',
      desc: 'AI Discord bot. Natural conversations, image understanding, and generation.',
      page: '/projects/boo',
      links: [
        { label: 'Code', url: 'https://github.com/VVIP-Kitchen/boo' },
        { label: 'Site', url: 'https://boo.ifkash.dev' }
      ]
    },
    {
      name: 'ferray',
      desc: 'NumPy-like ndarray in Rust with Python bindings. Learning project.',
      page: '/projects/ferray',
      links: [
        { label: 'Code', url: 'https://github.com/kashifulhaque/ferray' }
      ]
    },
    {
      name: 'endark',
      desc: 'Monochrome, dark-only CSS library. Glassmorphism meets terminal-editorial aesthetic. Zero dependencies.',
      links: [
        { label: 'Site', url: 'https://endark.ifkash.dev' },
        { label: 'Code', url: 'https://github.com/kashifulhaque/endark' }
      ]
    },
    {
      name: "Opencode theme for VS Code",
      desc: "VS Code light and dark theme generated from OpenCode reference JSON.",
      links: [
        { label: "Download", url: "https://github.com/kashifulhaque/opencode-vscode-theme/releases/latest" },
        { label: "Code", url: "https://github.com/kashifulhaque/opencode-vscode-theme" }
      ]
    }
  ];
</script>

<div class="page">
  <header class="page-header">
    <h1 class="page-title">Projects</h1>
    <p class="page-desc">Things I've built or am actively shaping.</p>
  </header>

  <section class="projects">
    {#each projects as project, i}
      <article class="project stagger" style="--i: {i}">
        <div class="project-top">
          {#if project.page}
            <a href={project.page} class="project-name-link">
              <h2 class="project-name">{project.name}</h2>
            </a>
          {:else}
            <h2 class="project-name">{project.name}</h2>
          {/if}
          <div class="project-links">
            {#each project.links as link}
              <a href={link.url} target="_blank" rel="noopener noreferrer" class="project-link">
                {link.label}
              </a>
            {/each}
          </div>
        </div>
        <p class="project-desc">{project.desc}</p>
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
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--border);
  }

  .page-title {
    font-size: clamp(2rem, 5vw + 0.5rem, 3rem);
    font-weight: 400;
    letter-spacing: -0.03em;
    color: var(--text-primary);
    margin-bottom: 0.375rem;
  }

  .page-desc {
    font-size: 1rem;
    color: var(--text-tertiary);
  }

  .projects {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  .project {
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--border-subtle);
  }

  .project:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .project-top {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  @media (min-width: 480px) {
    .project-top {
      flex-direction: row;
      align-items: baseline;
      justify-content: space-between;
    }
  }

  .project-name {
    font-family: var(--font-sans);
    font-size: 1.0625rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .project-name-link {
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .project-name-link:hover .project-name {
    color: var(--accent);
  }

  .project-links {
    display: flex;
    gap: 1rem;
  }

  .project-link {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .project-link:hover {
    color: var(--accent);
  }

  .project-desc {
    font-size: 0.9375rem;
    line-height: 1.65;
    color: var(--text-secondary);
  }
</style>
