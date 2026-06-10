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

<header class="page-header">
  <h1 class="section-title">Admin.</h1>
  <p class="section-subtitle">Signed in as <span class="email">{data.email}</span></p>
</header>

<section class="admin-list">
  {#each entries as entry, i}
    <a href={entry.href} class="admin-row stagger" style="--i: {i}">
      <span class="admin-icon"><svelte:component this={entry.icon} size={18} /></span>
      <div class="admin-body">
        <span class="admin-name">{entry.name}</span>
        <p class="admin-desc">{entry.desc}</p>
      </div>
      <span class="admin-arrow" aria-hidden="true">&rarr;</span>
    </a>
  {/each}
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .email {
    font-family: var(--font-mono);
    color: var(--blueprint);
  }

  .admin-list {
    display: flex;
    flex-direction: column;
    margin-top: 8px;
  }

  .admin-row {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    padding: 20px 4px;
    border-bottom: 1px solid var(--rule-soft);
    color: inherit;
    transition: background 0.15s;
  }

  .admin-row:first-child {
    border-top: 1px solid var(--rule-soft);
  }

  .admin-row:hover {
    background: var(--blueprint-tint);
    border-bottom-color: var(--rule-soft);
  }

  .admin-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    flex-shrink: 0;
    border: 1px solid var(--rule-soft);
    color: var(--blueprint);
  }

  .admin-body {
    flex: 1;
    min-width: 0;
  }

  .admin-name {
    font-family: var(--font-mono);
    font-size: 0.9rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink);
  }

  .admin-row:hover .admin-name {
    color: var(--blueprint);
  }

  .admin-desc {
    margin: 6px 0 0;
    font-size: 0.92rem;
    line-height: 1.5;
    color: var(--ink-soft);
  }

  .admin-arrow {
    font-size: 1.1rem;
    color: var(--ink-mute);
    transition: color 0.15s, transform 0.15s;
  }

  .admin-row:hover .admin-arrow {
    color: var(--blueprint);
    transform: translateX(4px);
  }
</style>
