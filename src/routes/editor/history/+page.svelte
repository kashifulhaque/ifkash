<script lang="ts">
  import { ArrowLeft, RotateCcw, FileText } from "lucide-svelte";
  import type { PageData } from "./$types";

  export let data: PageData;
  $: history = data.history;

  function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleString();
  }
</script>

<svelte:head>
  <title>History — Kashif</title>
</svelte:head>

<div class="history-page">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/editor" class="back"><ArrowLeft size={14} /> Editor</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">History</span>
    </nav>
    <div class="head-row">
      <h1 class="page-title">History</h1>
      <span class="count">{history.length} versions</span>
    </div>
  </header>

  <section class="main">
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Version</th>
            <th>Created</th>
            <th>Notes</th>
            <th><span class="sr-only">Actions</span></th>
          </tr>
        </thead>
        <tbody>
          {#each history as item, i (item.id)}
            <tr class="stagger" style="--i: {Math.min(i, 12)}">
              <td class="version-cell">v{item.version || "?"}</td>
              <td class="date-cell">{formatDate(item.created)}</td>
              <td class="notes-cell">{item.notes || "—"}</td>
              <td class="actions-cell">
                <a href={item.pdf_url} target="_blank" class="btn btn-sm btn--ghost icon" title="View PDF" aria-label="View PDF">
                  <FileText size={14} />
                </a>
                <a href="/editor?id={item.id}" class="btn btn-sm load">
                  <RotateCcw size={12} /> Load
                </a>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>
</div>

<style>
  .page-head {
    padding-top: var(--s-7);
    padding-bottom: var(--s-6);
  }

  .back {
    display: inline-flex;
    align-items: center;
    gap: var(--s-1);
  }

  .breadcrumb [aria-current="page"] {
    color: var(--ink);
  }

  .head-row {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-2) var(--s-5);
  }

  .page-title {
    font-size: var(--text-2xl);
  }

  .count {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .main {
    padding-top: var(--s-6);
  }

  .table-container {
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
    overflow: hidden;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    height: 36px;
    padding: 0 var(--s-4);
    border-bottom: 1px solid var(--line);
    background: var(--surface-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    text-align: left;
  }

  tbody tr {
    transition: background-color var(--dur-fast) var(--ease);
  }

  tbody tr:hover {
    background: var(--hover-wash);
  }

  td {
    height: 56px;
    padding: var(--s-2) var(--s-4);
    border-bottom: 1px solid var(--line);
    vertical-align: middle;
  }

  tr:last-child td {
    border-bottom: none;
  }

  .version-cell {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .date-cell {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .notes-cell {
    color: var(--ink-2);
    font-size: var(--text-sm);
  }

  .actions-cell {
    text-align: right;
    white-space: nowrap;
  }

  .btn.icon {
    width: 32px;
    padding: 0;
    vertical-align: middle;
  }

  .load {
    gap: var(--s-1);
    margin-left: var(--s-1);
    vertical-align: middle;
  }

  /* Small screens: each version becomes a stacked row. */
  @media (max-width: 639px) {
    thead {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip: rect(0 0 0 0);
    }

    table,
    tbody {
      display: block;
    }

    tbody tr {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      grid-template-areas:
        "version actions"
        "date actions"
        "notes notes";
      gap: var(--s-1) var(--s-3);
      padding: var(--s-4);
      border-bottom: 1px solid var(--line);
    }

    tbody tr:last-child {
      border-bottom: none;
    }

    td {
      height: auto;
      padding: 0;
      border: none;
    }

    .version-cell {
      grid-area: version;
    }
    .date-cell {
      grid-area: date;
    }
    .notes-cell {
      grid-area: notes;
      margin-top: var(--s-1);
    }
    .actions-cell {
      grid-area: actions;
      align-self: center;
    }
  }
</style>
