<svelte:head>
  <title>Projects — Kashif</title>
  <meta name="description" content="Side projects and open source work." />
</svelte:head>

<script lang="ts">
  type Project = {
    name: string;
    desc: string;
    links: { label: string; url: string }[];
    page?: string; // Optional internal page link
  };

  const projects: Project[] = [
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
      <article class="project" style="--delay: {i * 50}ms">
        <div class="project-header">
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
    gap: 3rem;
  }
  
  .page-header {
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--gray-800);
  }
  
  .page-title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--white);
    margin-bottom: 0.5rem;
  }
  
  .page-desc {
    font-size: 1rem;
    color: var(--gray-500);
  }

  .projects {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }
  
  .project {
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--gray-900);
    animation: fade-up var(--duration-slow) var(--ease-out) backwards;
    animation-delay: var(--delay);
  }
  
  .project:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }
  
  .project-header {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }
  
  @media (min-width: 480px) {
    .project-header {
      flex-direction: row;
      align-items: center;
      justify-content: space-between;
    }
  }
  
  .project-name {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--white);
    letter-spacing: -0.01em;
  }

  .project-name-link {
    transition: opacity var(--duration-fast) var(--ease-out);
  }

  .project-name-link:hover {
    opacity: 0.7;
  }

  .project-name-link .project-name {
    margin: 0;
  }
  
  .project-links {
    display: flex;
    gap: 1rem;
  }
  
  .project-link {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--gray-600);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .project-link:hover {
    color: var(--white);
  }
  
  .project-desc {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--gray-400);
  }
  
  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
