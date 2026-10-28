<svelte:head>
  <title>Tools — Kashif</title>
  <meta name="description" content="Small in-browser utilities I've built." />
</svelte:head>

<script lang="ts">
  import { reveal, splitReveal } from '$lib/motion';
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

  const pad = (n: number) => String(n).padStart(2, '0');
</script>

<header class="page-head">
  <p class="eyebrow">04 / Tools</p>
  <h1 class="page-title" use:splitReveal>Tools</h1>
  <p class="lead">Small utilities — most run entirely in your browser</p>
</header>

<ol class="rows list" use:reveal={{ children: 'li', gap: 60 }}>
  {#each tools as tool, i}
    <li>
      <a href={tool.page} class="row">
        <span class="row-n">{pad(i + 1)}</span>
        <span class="row-body">
          <span class="row-name">{tool.name}</span>
          <span class="row-desc">{tool.desc}</span>
        </span>
        <span class="row-meta">
          {#if tool.status === 'active'}
            <span class="led" aria-hidden="true"></span>
          {/if}
          <span class="row-path">{tool.page}</span>
          <span class="row-arrow" aria-hidden="true">→</span>
        </span>
      </a>
    </li>
  {/each}
</ol>

<style>
  .list {
    margin-top: var(--s-7);
  }

  .row {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) auto;
    align-items: baseline;
    column-gap: var(--s-5);
    padding: var(--s-5) 0;
    color: var(--ink);
    text-decoration: none;
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

  .row-name {
    color: var(--ink);
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.25;
    letter-spacing: -0.015em;
    transition: color var(--dur-fast) var(--ease);
  }

  .row-desc {
    max-width: var(--measure);
    color: var(--ink-2);
    font-size: var(--text-base);
    line-height: 1.55;
  }

  .row-meta {
    display: inline-flex;
    align-items: center;
    gap: var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: nowrap;
  }
  .row-meta .led {
    width: 6px;
    height: 6px;
  }

  .row-arrow {
    font-family: var(--font-sans);
    font-size: var(--text-base);
    transition:
      transform var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease);
  }

  .row:hover .row-name,
  .row:hover .row-arrow {
    color: var(--fjord);
  }
  .row:hover .row-arrow {
    transform: translateX(3px);
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
    }

    .row-name {
      font-size: var(--text-md);
    }
  }
</style>
