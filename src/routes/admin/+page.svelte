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
  <h1 class="section-title">Admin</h1>
  <p class="section-subtitle">Signed in as <span class="email">{data.email}</span></p>
</header>

<section class="admin-list">
  {#each entries as entry, i}
    <a href={entry.href} class="admin-row stagger" style="--i: {i}">
      <span class="admin-icon"><svelte:component this={entry.icon} size={18} strokeWidth={1.8} /></span>
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
    font-family: var(--font-mono-g);
    color: var(--signal-hi);
  }

  .admin-list {
    display: flex;
    flex-direction: column;
    margin-top: 8px;
  }

  .admin-row {
    display: grid;
    grid-template-columns: 36px minmax(0, 1fr) 24px;
    align-items: center;
    gap: 20px;
    padding: 22px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--line-soft);
    color: var(--ink);
    transition:
      background 0.15s,
      color 0.15s;
  }

  .admin-row:first-child {
    border-top: 1px solid var(--line-soft);
  }

  .admin-row:hover {
    background: var(--ink);
    color: var(--void);
    border-bottom-color: var(--ink);
  }

  .admin-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    flex-shrink: 0;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    color: var(--signal);
  }

  .admin-row:hover .admin-icon {
    border-color: var(--void);
    color: var(--void);
  }

  .admin-body {
    min-width: 0;
  }

  .admin-name {
    font-family: var(--font-dots);
    font-size: 1.5rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    line-height: 1;
    color: inherit;
  }

  .admin-desc {
    margin: 8px 0 0;
    font-family: var(--font-sans-g);
    font-size: 0.9rem;
    line-height: 1.5;
    color: var(--ink-soft);
    max-width: 640px;
  }

  .admin-row:hover .admin-desc {
    color: var(--void);
  }

  .admin-arrow {
    font-family: var(--font-mono-g);
    font-size: 1rem;
    color: var(--ink-mute);
    transition: color 0.15s, transform 0.15s;
  }

  .admin-row:hover .admin-arrow {
    color: var(--void);
    transform: translateX(4px);
  }

  @media (max-width: 768px) {
    .admin-row {
      grid-template-columns: 36px minmax(0, 1fr);
      padding: 18px 10px;
      margin: 0 -10px;
    }

    .admin-name {
      font-size: 1.25rem;
    }

    .admin-arrow {
      display: none;
    }
  }
</style>
