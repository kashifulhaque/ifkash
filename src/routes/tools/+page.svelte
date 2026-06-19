<svelte:head>
  <title>Tools — Kashif</title>
  <meta name="description" content="Small in-browser utilities I've built." />
</svelte:head>

<script lang="ts">
  type Tool = {
    name: string;
    desc: string;
    links: { label: string; url: string }[];
    page?: string;
    status?: 'active' | 'shipped' | 'archived';
  };

  const tools: Tool[] = [
    {
      name: 'PDF Annotator',
      desc: 'Annotate PDFs right in the browser — add text, draw freehand, stamp a signature or image, then download. Fully client-side; your files never leave the page.',
      page: '/tools/pdf-annotator',
      status: 'active',
      links: []
    },
    {
      name: 'Expense Splitter',
      desc: 'Split expenses with friends and settle up — a self-hosted Splitwise. Sign in with Google, add friends as names, log expenses with equal/exact/percent/share splits, and get the minimal set of payments to square up.',
      page: '/tools/splitter',
      status: 'active',
      links: []
    }
  ];
</script>

<header class="page-header">
  <h1 class="section-title">Tools.</h1>
  <p class="section-subtitle">Small utilities — most run entirely in your browser.</p>
</header>

<section class="tools-list">
  {#each tools as tool, i}
    <article class="tool-row stagger" style="--i: {i}">
      <span
        class="tool-status"
        class:active={tool.status === 'active'}
        class:shipped={tool.status === 'shipped'}
        class:archived={tool.status === 'archived'}
      ></span>
      <div class="tool-body">
        <div class="tool-top">
          {#if tool.page}
            <a href={tool.page} class="tool-name-link">
              <span class="tool-name">{tool.name}</span>
            </a>
          {:else}
            <span class="tool-name">{tool.name}</span>
          {/if}
          <div class="tool-links">
            {#each tool.links as link}
              <a
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                class="tool-link"
              >
                {link.label}
              </a>
            {/each}
          </div>
        </div>
        <p class="tool-desc">{tool.desc}</p>
      </div>
    </article>
  {/each}
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .tools-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--rule-soft);
  }

  .tool-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr);
    align-items: start;
    gap: 20px;
    padding: 20px 0;
    border-bottom: 1px solid var(--rule-soft);
  }

  .tool-status {
    width: 12px;
    height: 12px;
    border: 1px solid var(--ink-mute);
    background: transparent;
    transform: translateY(8px);
    flex-shrink: 0;
  }

  .tool-status.active {
    background: var(--blueprint);
    border-color: var(--blueprint);
  }

  .tool-status.shipped {
    background: linear-gradient(
      135deg,
      var(--blueprint) 0%,
      var(--blueprint) 50%,
      transparent 50%,
      transparent 100%
    );
    border-color: var(--blueprint);
  }

  .tool-status.archived {
    background: transparent;
    border-style: dashed;
    border-color: var(--ink-mute);
  }

  .tool-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }

  .tool-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .tool-name {
    font-family: var(--font-display);
    font-size: 1.5rem;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    color: var(--ink);
    line-height: 1.05;
  }

  .tool-name-link {
    border-bottom: none;
  }

  .tool-name-link:hover .tool-name {
    color: var(--blueprint);
  }

  .tool-links {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .tool-link {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    padding: 4px 10px;
    background: transparent;
    color: var(--ink-soft);
    border: 1px solid var(--rule-soft);
    text-decoration: none;
    transition: color 0.15s, border-color 0.15s;
  }

  .tool-link:hover {
    color: var(--blueprint);
    border-color: var(--blueprint);
    border-bottom: 1px solid var(--blueprint);
  }

  .tool-desc {
    font-family: var(--font-body);
    font-size: 0.98rem;
    line-height: 1.6;
    color: var(--ink-soft);
    max-width: 760px;
  }

  @media (max-width: 768px) {
    .tool-row {
      gap: 16px;
      padding: 18px 0;
    }

    .tool-name {
      font-size: 1.25rem;
    }

    .tool-top {
      flex-direction: column;
      align-items: flex-start;
      gap: 8px;
    }
  }
</style>
