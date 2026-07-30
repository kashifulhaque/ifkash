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
  <h1 class="section-title">Tools</h1>
  <p class="section-subtitle">Small utilities — most run entirely in your browser</p>
</header>

<section class="tools-list">
  {#each tools as tool, i}
    <a href={tool.page} class="tool-row stagger" style="--i: {i}">
      <span
        class="tool-led"
        class:lit={tool.status === 'active'}
        class:shipped={tool.status === 'shipped'}
        class:archived={tool.status === 'archived'}
        aria-hidden="true"
      ></span>
      <div class="tool-body">
        <div class="tool-top">
          <span class="tool-name">{tool.name}</span>
          <span class="tool-arrow" aria-hidden="true">&rarr;</span>
        </div>
        <p class="tool-desc">{tool.desc}</p>
      </div>
    </a>
  {/each}
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .tools-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--line-soft);
  }

  .tool-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr);
    align-items: start;
    gap: 20px;
    padding: 22px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--line-soft);
    color: var(--ink);
    transition:
      background 0.15s,
      color 0.15s;
  }

  .tool-row:hover {
    background: var(--ink);
    color: var(--void);
    border-bottom-color: var(--ink);
  }

  .tool-led {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    border: 1px solid var(--ink-mute);
    background: transparent;
    transform: translateY(10px);
    justify-self: center;
    flex-shrink: 0;
  }

  .tool-led.lit {
    background: var(--signal);
    border-color: var(--signal);
    box-shadow:
      0 0 6px var(--signal-glow),
      0 0 2px var(--signal);
    animation: led-pulse 2.4s ease-in-out infinite;
  }

  .tool-led.shipped {
    background: var(--ok);
    border-color: var(--ok);
    box-shadow: 0 0 5px rgba(61, 220, 132, 0.4);
  }

  .tool-led.archived {
    background: transparent;
    border-style: dashed;
    border-color: var(--ink-mute);
  }

  .tool-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }

  .tool-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
  }

  .tool-name {
    font-family: var(--font-dots);
    font-size: 1.9rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    line-height: 1;
    color: inherit;
  }

  .tool-arrow {
    font-family: var(--font-mono-g);
    font-size: 1rem;
    color: var(--ink-mute);
    transition:
      transform 0.15s,
      color 0.15s;
  }

  .tool-row:hover .tool-arrow {
    color: var(--void);
    transform: translateX(4px);
  }

  .tool-desc {
    font-family: var(--font-sans-g);
    font-size: 0.95rem;
    line-height: 1.6;
    color: var(--ink-soft);
    max-width: 720px;
  }

  .tool-row:hover .tool-desc {
    color: var(--void);
  }

  @media (max-width: 768px) {
    .tool-row {
      gap: 16px;
      padding: 18px 10px;
      margin: 0 -10px;
    }

    .tool-name {
      font-size: 1.5rem;
    }
  }
</style>
