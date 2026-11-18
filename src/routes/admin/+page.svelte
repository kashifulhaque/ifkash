<script lang="ts">
  import { FileText, History, NotebookPen } from "lucide-svelte";
  import type { PageData } from "./$types";

  export let data: PageData;

  type AdminEntry = {
    name: string;
    desc: string;
    href: string;
    icon: typeof FileText;
  };

  const entries: AdminEntry[] = [
    {
      name: "Notes",
      desc: "Browse and edit the obsidian-sync vault. Reads and commits straight through the GitHub API — no local Obsidian needed.",
      href: "/admin/notes",
      icon: NotebookPen,
    },
    {
      name: "Resume Editor",
      desc: "Edit the Typst source of the resume, compile it in the browser and publish the latest copy shown on the homepage.",
      href: "/editor",
      icon: FileText,
    },
    {
      name: "Resume History",
      desc: "Browse previously published resume versions and load any of them back into the editor.",
      href: "/editor/history",
      icon: History,
    },
  ];
</script>

<svelte:head>
  <title>Admin — Kashif</title>
  <meta name="robots" content="noindex" />
</svelte:head>

<header class="page-head">
  <p class="eyebrow"><span class="led" aria-hidden="true"></span> Owner only</p>
  <h1 class="page-title">Admin</h1>
  <p class="lead">Signed in as <span class="email">{data.email}</span></p>
</header>

<ol class="rows admin-list">
  {#each entries as entry, i}
    <li class="stagger" style="--i: {i}">
      <a href={entry.href} class="admin-row">
        <span class="admin-index">{String(i + 1).padStart(2, "0")}</span>
        <div class="admin-body">
          <span class="admin-name">
            <span class="admin-icon" aria-hidden="true"><svelte:component this={entry.icon} size={16} strokeWidth={1.6} /></span>
            {entry.name}
          </span>
          <p class="admin-desc">{entry.desc}</p>
        </div>
        <span class="admin-meta">
          <span class="admin-path">{entry.href}</span>
          <span class="admin-arrow" aria-hidden="true">&rarr;</span>
        </span>
      </a>
    </li>
  {/each}
</ol>

<style>
  .email {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: 0.86em;
    overflow-wrap: anywhere;
  }

  .admin-list {
    margin-top: var(--s-7);
  }

  .admin-row {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) auto;
    align-items: baseline;
    gap: var(--s-5);
    padding: var(--s-5) 0;
    color: var(--ink);
  }

  .admin-index {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .admin-body {
    min-width: 0;
  }

  .admin-name {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    color: var(--ink);
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.25;
    letter-spacing: -0.015em;
    transition: color var(--dur-fast) var(--ease);
  }

  .admin-icon {
    display: inline-flex;
    color: var(--ink-3);
    transition: color var(--dur-fast) var(--ease);
  }

  .admin-desc {
    max-width: var(--measure);
    margin: var(--s-2) 0 0;
    color: var(--ink-2);
    font-size: 15px;
    line-height: 1.55;
  }

  .admin-meta {
    display: inline-flex;
    align-items: baseline;
    gap: var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: nowrap;
  }

  .admin-arrow {
    color: var(--ink-3);
    font-size: var(--text-sm);
    transition: transform var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }

  .admin-row:hover .admin-name,
  .admin-row:hover .admin-icon,
  .admin-row:hover .admin-arrow {
    color: var(--fjord);
  }

  .admin-row:hover .admin-arrow {
    transform: translateX(3px);
  }

  @media (max-width: 640px) {
    .admin-row {
      grid-template-columns: 32px minmax(0, 1fr);
      gap: var(--s-3);
      padding: var(--s-4) 0;
    }

    .admin-name {
      font-size: var(--text-md);
    }

    .admin-meta {
      display: none;
    }
  }
</style>
