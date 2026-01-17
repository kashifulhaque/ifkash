<script lang="ts">
  type Project = {
    name: string;
    desc: string;
    links: { label: string; url: string }[];
    featured?: boolean;
  };

  const projects: Project[] = [
    {
      name: "smol-llama",
      desc: "360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, SwiGLU. Single H100, 22hrs, $53.",
      links: [
        { label: "Model", url: "https://huggingface.co/ifkash/smol-llama" },
        { label: "Code", url: "https://github.com/kashifulhaque/smol-llama" },
      ],
      featured: true,
    },
    {
      name: "FineWeb-6B",
      desc: "Curated 6B token dataset from FineWeb. Pre-tokenized with custom 49K BPE vocab.",
      links: [
        {
          label: "Dataset",
          url: "https://huggingface.co/datasets/ifkash/fineweb-6b",
        },
      ],
      featured: true,
    },
    {
      name: "smoltorch",
      desc: "Autograd engine and neural networks in ~500 lines of NumPy. Educational deep learning.",
      links: [
        { label: "Code", url: "https://github.com/kashifulhaque/smoltorch" },
        { label: "PyPI", url: "https://pypi.org/project/smoltorch/" },
      ],
    },
    {
      name: "NoPokeDB",
      desc: "Lightweight vector DB with hnswlib + SQLite. Crash recovery, 2K+ PyPI downloads.",
      links: [
        { label: "Code", url: "https://github.com/kashifulhaque/nopokedb" },
        { label: "PyPI", url: "https://pypi.org/project/nopokedb/" },
      ],
    },
    {
      name: "Boo",
      desc: "AI Discord bot. Natural conversations, image understanding, and generation.",
      links: [
        { label: "Code", url: "https://github.com/VVIP-Kitchen/boo" },
        { label: "Site", url: "https://boo.ifkash.dev" },
      ],
    },
    {
      name: "tinyndarray",
      desc: "NumPy-like ndarray in Rust with Python bindings. Learning project.",
      links: [
        { label: "Code", url: "https://github.com/kashifulhaque/tinyndarray" },
      ],
    },
  ];
</script>

<svelte:head>
  <title>Projects — Kashif</title>
  <meta name="description" content="Side projects and open source work." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <span class="category-badge">Open Source</span>
    <h1 class="page-title">Projects</h1>
    <p class="page-desc">Things I've built or am actively shaping.</p>
  </header>

  <section class="projects">
    {#each projects as project, i}
      <article
        class="project"
        class:featured={project.featured}
        style="--delay: {i * 50}ms"
      >
        <div class="project-header">
          <div class="project-title-row">
            <h2 class="project-name">{project.name}</h2>
            {#if project.featured}
              <span class="featured-badge">Featured</span>
            {/if}
          </div>
          <div class="project-links">
            {#each project.links as link}
              <a
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                class="project-link"
              >
                {link.label} ↗
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
    border-bottom: 1px solid var(--color-border);
  }

  .category-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.15em;
    color: var(--gray-400);
    margin-bottom: 1rem;
  }

  .category-badge::before {
    content: "";
    width: 6px;
    height: 6px;
    background: var(--accent);
    border-radius: 50%;
  }

  .page-title {
    font-family: var(--font-serif);
    font-size: clamp(2.5rem, 6vw, 3.5rem);
    font-weight: 400;
    letter-spacing: -0.02em;
    color: var(--white);
    margin-bottom: 0.75rem;
  }

  .page-desc {
    font-size: 1rem;
    color: var(--gray-500);
  }

  .projects {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .project {
    padding: 2rem 0;
    border-bottom: 1px solid var(--color-border);
    animation: fade-up var(--duration-slow) var(--ease-out) backwards;
    animation-delay: var(--delay);
    transition: all var(--duration-fast) var(--ease-out);
  }

  .project:first-child {
    border-top: 1px solid var(--color-border);
  }

  .project:hover {
    padding-left: 1rem;
    padding-right: 1rem;
    margin-left: -1rem;
    margin-right: -1rem;
    background: var(--glass-bg);
  }

  .project.featured {
    background: linear-gradient(
      135deg,
      rgba(230, 126, 34, 0.03) 0%,
      transparent 100%
    );
  }

  .project.featured:hover {
    background: linear-gradient(
      135deg,
      rgba(230, 126, 34, 0.06) 0%,
      rgba(255, 255, 255, 0.02) 100%
    );
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

  .project-title-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .project-name {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--white);
    letter-spacing: -0.01em;
  }

  .featured-badge {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--accent);
    padding: 0.25rem 0.5rem;
    background: rgba(230, 126, 34, 0.15);
    border-radius: 4px;
  }

  .project-links {
    display: flex;
    gap: 1rem;
  }

  .project-link {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--gray-500);
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
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
