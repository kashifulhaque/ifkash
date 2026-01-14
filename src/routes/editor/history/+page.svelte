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
  <header class="header">
    <div class="header-left">
      <a href="/editor" class="back-btn"><ArrowLeft size={18} /></a>
      <h1>History</h1>
    </div>
    <span class="count">{history.length} versions</span>
  </header>

  <main class="main">
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Version</th>
            <th>Created</th>
            <th>Notes</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each history as item, i (item.id)}
            <tr style="--delay: {i * 30}ms">
              <td class="version-cell">v{item.version || "?"}</td>
              <td class="date-cell">{formatDate(item.created)}</td>
              <td class="notes-cell">{item.notes || "—"}</td>
              <td class="actions-cell">
                <a href={item.pdf_url} target="_blank" class="action-btn" title="View PDF">
                  <FileText size={14} />
                </a>
                <a href="/editor?id={item.id}" class="action-btn load">
                  <RotateCcw size={12} /> Load
                </a>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </main>
</div>

<style>
  .history-page {
    min-height: 100vh;
    background: var(--black);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    background: var(--gray-950);
    border-bottom: 1px solid var(--gray-800);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .back-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.5rem;
    color: var(--gray-500);
    border-radius: var(--radius-sm);
    transition: all var(--duration-fast) var(--ease-out);
  }

  .back-btn:hover {
    color: var(--white);
    background: var(--gray-900);
  }

  .header h1 {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--white);
  }

  .count {
    font-size: 0.75rem;
    color: var(--gray-600);
  }

  .main {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem 1.5rem;
  }

  .table-container {
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    padding: 0.875rem 1rem;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--gray-600);
    text-align: left;
    background: var(--gray-900);
    border-bottom: 1px solid var(--gray-800);
  }

  tr {
    animation: fade-in 0.3s var(--ease-out) backwards;
    animation-delay: var(--delay);
  }

  tbody tr:hover {
    background: var(--gray-900);
  }

  td {
    padding: 0.875rem 1rem;
    border-bottom: 1px solid var(--gray-900);
  }

  tr:last-child td {
    border-bottom: none;
  }

  .version-cell {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--white);
  }

  .date-cell {
    font-size: 0.8125rem;
    color: var(--gray-500);
  }

  .notes-cell {
    font-size: 0.8125rem;
    color: var(--gray-600);
    font-style: italic;
  }

  .actions-cell {
    text-align: right;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.375rem 0.5rem;
    font-size: 0.75rem;
    color: var(--gray-500);
    border-radius: var(--radius-sm);
    transition: all var(--duration-fast) var(--ease-out);
  }

  .action-btn:hover {
    color: var(--white);
    background: var(--gray-800);
  }

  .action-btn.load {
    margin-left: 0.5rem;
    background: var(--gray-900);
    border: 1px solid var(--gray-800);
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
