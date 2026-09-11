<svelte:head>
  <title>Roadmap — Kashif</title>
  <meta
    name="description"
    content="An interview roadmap for LLM pre-training, post-training, RL, and inference systems, anchored to shipped work."
  />
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import {
    ARTIFACTS,
    MODULES,
    TOTAL_ITEMS,
    KIND_LABEL,
    KIND_ORDER,
    type Item,
    type ItemKind
  } from '$lib/roadmap/curriculum';
  import { loadDone, saveDone, loadOpen, saveOpen } from '$lib/roadmap/progress';

  // Progress lives in localStorage, so it is read after mount. The server
  // render shows every item unchecked and every module collapsed.
  let done = new Set<string>();
  let open = new Set<string>();

  onMount(() => {
    done = loadDone();
    open = loadOpen();
  });

  function toggleItem(id: string): void {
    if (done.has(id)) done.delete(id);
    else done.add(id);
    done = done;
    saveDone(done);
  }

  function toggleModule(id: string): void {
    if (open.has(id)) open.delete(id);
    else open.add(id);
    open = open;
    saveOpen(open);
  }

  function resetProgress(): void {
    if (!confirm('Clear every checked item on this page?')) return;
    done = new Set();
    saveDone(done);
  }

  // `done` is passed explicitly so the template re-evaluates when it changes.
  function countDone(items: Item[], doneSet: Set<string>): number {
    return items.reduce((n, item) => n + (doneSet.has(item.id) ? 1 : 0), 0);
  }

  function ofKind(items: Item[], kind: ItemKind): Item[] {
    return items.filter((item) => item.kind === kind);
  }

  $: doneCount = done.size;
  $: percent = TOTAL_ITEMS === 0 ? 0 : Math.round((doneCount / TOTAL_ITEMS) * 100);
</script>

<header class="page-header">
  <h1 class="section-title">Roadmap</h1>
  <p class="section-subtitle">
    What has been built, and the course that turns it into interview answers
  </p>
</header>

<section class="progress" aria-label="Overall progress">
  <div class="progress-row">
    <span class="progress-count">
      <strong>{doneCount}</strong> of {TOTAL_ITEMS} items
    </span>
    <span class="progress-pct">{percent}%</span>
  </div>
  <div class="progress-bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={percent}>
    <span class="progress-fill" style="width: {percent}%"></span>
  </div>
  <p class="progress-note">
    Progress is stored in this browser. Work through the modules in order. Check an item when
    you can do it or answer it without notes, and uncheck it when you cannot.
  </p>
</section>

<section class="block" aria-labelledby="built-heading">
  <div class="block-heading">
    <h2 id="built-heading">Built so far</h2>
    <p>The numbers to have ready. Every module that follows points back to these.</p>
  </div>

  <div class="artifact-list">
    {#each ARTIFACTS as artifact, i}
      <details class="artifact stagger" style="--i: {i}">
        <summary class="artifact-summary">
          <span class="artifact-title">{artifact.title}</span>
          <span class="artifact-meta">
            {artifact.facts.length} facts · {artifact.gaps.length} to shore up
          </span>
          <span class="chev" aria-hidden="true">+</span>
        </summary>
        <div class="artifact-body">
          <div class="artifact-links">
            <a href={artifact.href} class="chip">Page</a>
            {#each artifact.links as link}
              <a
                href={link.url}
                class="chip"
                target={link.url.startsWith('http') ? '_blank' : undefined}
                rel={link.url.startsWith('http') ? 'noopener noreferrer' : undefined}
              >
                {link.label}
              </a>
            {/each}
          </div>
          <p class="group-label">Talking points</p>
          <ul class="fact-list">
            {#each artifact.facts as fact}
              <li>{fact}</li>
            {/each}
          </ul>
          <p class="group-label gap-label">Shore up</p>
          <ul class="fact-list gap-list">
            {#each artifact.gaps as gap}
              <li>{gap}</li>
            {/each}
          </ul>
        </div>
      </details>
    {/each}
  </div>
</section>

<section class="block" aria-labelledby="course-heading">
  <div class="block-heading">
    <h2 id="course-heading">Course</h2>
    <p>Twelve modules, in order. Each one lists what you already did, what to read, what to build, and the questions to answer cold.</p>
  </div>

  <ol class="module-list">
    {#each MODULES as mod, i}
      {@const total = mod.items.length}
      {@const finished = countDone(mod.items, done)}
      {@const isOpen = open.has(mod.id)}
      <li class="module stagger" class:open={isOpen} style="--i: {i}">
        <button
          type="button"
          class="module-head"
          aria-expanded={isOpen}
          aria-controls="module-{mod.id}"
          on:click={() => toggleModule(mod.id)}
        >
          <span class="module-n">{mod.n}</span>
          <span class="module-main">
            <span class="module-title">{mod.title}</span>
            <span class="module-why">{mod.why}</span>
          </span>
          <span class="module-progress" class:complete={finished === total && total > 0}>
            {finished}/{total}
          </span>
          <span class="chev" aria-hidden="true">{isOpen ? '−' : '+'}</span>
        </button>

        {#if isOpen}
          <div class="module-body" id="module-{mod.id}">
            {#if mod.anchors.length}
              <p class="group-label">You already did</p>
              <ul class="anchor-list">
                {#each mod.anchors as anchor}
                  <li>
                    <a
                      href={anchor.href}
                      class="anchor-link"
                      target={anchor.href.startsWith('http') ? '_blank' : undefined}
                      rel={anchor.href.startsWith('http') ? 'noopener noreferrer' : undefined}
                    >
                      {anchor.label}
                    </a>
                    <span class="anchor-note">{anchor.note}</span>
                  </li>
                {/each}
              </ul>
            {/if}

            {#each KIND_ORDER as kind}
              {@const items = ofKind(mod.items, kind)}
              {#if items.length}
                <p class="group-label">
                  {KIND_LABEL[kind]}
                  <span class="group-count">{countDone(items, done)}/{items.length}</span>
                </p>
                <ul class="item-list">
                  {#each items as item}
                    <li class="item" class:done={done.has(item.id)}>
                      <label class="item-row">
                        <input
                          type="checkbox"
                          checked={done.has(item.id)}
                          on:change={() => toggleItem(item.id)}
                        />
                        <span class="item-title">{item.title}</span>
                      </label>
                      {#if item.links?.length}
                        <div class="item-links">
                          {#each item.links as link}
                            <a
                              href={link.url}
                              class="chip"
                              target="_blank"
                              rel="noopener noreferrer"
                            >
                              {link.label}
                            </a>
                          {/each}
                        </div>
                      {/if}
                      {#if item.detail}
                        {#if item.kind === 'question'}
                          <details class="answer">
                            <summary>Answer sketch</summary>
                            <p class="detail">{item.detail}</p>
                          </details>
                        {:else}
                          <p class="detail">{item.detail}</p>
                        {/if}
                      {/if}
                    </li>
                  {/each}
                </ul>
              {/if}
            {/each}
          </div>
        {/if}
      </li>
    {/each}
  </ol>
</section>

<footer class="page-foot">
  <p>
    To add a question from an interview, append it to
    <code>src/lib/roadmap/curriculum.ts</code> under the module it belongs to.
  </p>
  <button type="button" class="btn btn--ghost" on:click={resetProgress}>Reset progress</button>
</footer>

<style>
  .page-header {
    padding-top: 16px;
  }

  .section-subtitle {
    margin-bottom: 32px;
  }

  /* Overall progress */
  .progress {
    max-width: 720px;
    padding-bottom: 40px;
  }

  .progress-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 10px;
  }

  .progress-count {
    color: var(--ink-2);
    font-size: 14px;
  }

  .progress-count strong {
    color: var(--foreground);
    font-weight: 500;
  }

  .progress-pct {
    color: var(--ink-3);
    font-family: var(--font-label);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .progress-bar {
    position: relative;
    height: 6px;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--fill);
  }

  .progress-fill {
    display: block;
    height: 100%;
    background: var(--accent);
    transition: width 240ms var(--ease-inout);
  }

  .progress-note {
    margin: 14px 0 0;
    color: var(--ink-3);
    font-size: 13px;
    line-height: 1.6;
  }

  /* Section blocks */
  .block {
    padding: 40px 0 0;
  }

  .block-heading {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
    gap: 24px;
    margin-bottom: 24px;
  }

  .block-heading h2 {
    font-size: 28px;
  }

  .block-heading p {
    max-width: 48ch;
    margin: 0;
    color: var(--ink-3);
    font-size: 13px;
    text-align: right;
  }

  .chev {
    justify-self: end;
    color: var(--ink-3);
    font-family: var(--font-label);
    font-size: 14px;
    line-height: 1;
  }

  .chip {
    display: inline-block;
    padding: 4px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--muted-foreground);
    font-family: var(--font-label);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    text-decoration: none;
    transition:
      color 0.15s var(--ease-inout),
      border-color 0.15s var(--ease-inout);
  }

  .chip:hover {
    border-color: var(--border-strong);
    color: var(--foreground);
  }

  .group-label {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin: 28px 0 10px;
    color: var(--ink-3);
    font-family: var(--font-label);
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .group-label:first-child {
    margin-top: 0;
  }

  .group-count {
    color: var(--ink-3);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0;
  }

  /* Built so far */
  .artifact-list {
    border-top: 1px solid var(--border);
  }

  .artifact {
    border-bottom: 1px solid var(--border);
  }

  .artifact-summary {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto 20px;
    align-items: baseline;
    gap: 24px;
    padding: 18px 12px;
    margin: 0 -12px;
    list-style: none;
    color: var(--foreground);
  }

  .artifact-summary::-webkit-details-marker {
    display: none;
  }

  .artifact-summary:hover {
    background: var(--fill);
  }

  .artifact[open] .chev {
    color: var(--accent);
  }

  .artifact-title {
    font-size: 18px;
    font-weight: 500;
  }

  .artifact-meta {
    color: var(--ink-3);
    font-family: var(--font-label);
    font-size: 11px;
    letter-spacing: 0.04em;
    white-space: nowrap;
  }

  .artifact-body {
    padding: 4px 0 26px 0;
  }

  .artifact-links {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 8px;
  }

  .fact-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 78ch;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .fact-list li {
    position: relative;
    padding-left: 18px;
    color: var(--ink-2);
    font-size: 14.5px;
    line-height: 1.6;
  }

  .fact-list li::before {
    content: '–';
    position: absolute;
    left: 0;
    color: var(--ink-3);
  }

  .gap-label {
    color: var(--accent);
  }

  .gap-list li::before {
    content: '!';
    color: var(--accent);
    font-family: var(--font-label);
  }

  /* Course */
  .module-list {
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--border);
    list-style: none;
  }

  .module {
    border-bottom: 1px solid var(--border);
  }

  .module-head {
    display: grid;
    grid-template-columns: 40px minmax(0, 1fr) auto 20px;
    align-items: baseline;
    gap: 24px;
    width: 100%;
    padding: 20px 12px;
    margin: 0 -12px;
    border: 0;
    background: transparent;
    color: var(--foreground);
    font: inherit;
    text-align: left;
  }

  .module-head:hover {
    background: var(--fill);
  }

  .module.open .module-head {
    background: transparent;
  }

  .module-n {
    color: var(--ink-3);
    font-family: var(--font-label);
    font-size: 13px;
  }

  .module-main {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .module-title {
    font-size: 20px;
    font-weight: 500;
    line-height: 1.25;
  }

  .module-why {
    max-width: 72ch;
    color: var(--ink-3);
    font-size: 13.5px;
    line-height: 1.55;
  }

  .module-progress {
    color: var(--ink-3);
    font-family: var(--font-label);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .module-progress.complete {
    color: var(--ok);
  }

  .module.open .chev {
    color: var(--accent);
  }

  .module-body {
    padding: 4px 0 32px 64px;
  }

  .anchor-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 78ch;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .anchor-list li {
    color: var(--ink-2);
    font-size: 14.5px;
    line-height: 1.6;
  }

  .anchor-link {
    margin-right: 8px;
    color: var(--foreground);
    font-weight: 500;
    text-decoration: underline;
    text-decoration-color: var(--border-strong);
    text-underline-offset: 3px;
  }

  .anchor-link:hover {
    text-decoration-color: var(--accent);
  }

  .anchor-note {
    color: var(--ink-2);
  }

  .item-list {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .item {
    padding: 12px 0;
    border-bottom: 1px solid var(--input);
  }

  .item:last-child {
    border-bottom: 0;
  }

  .item-row {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr);
    align-items: start;
    gap: 12px;
    color: var(--foreground);
    font-size: 15px;
    line-height: 1.55;
  }

  .item-row input {
    width: 16px;
    height: 16px;
    margin: 4px 0 0;
    accent-color: var(--accent);
  }

  .item.done .item-title {
    color: var(--ink-3);
    text-decoration: line-through;
    text-decoration-color: var(--ink-3);
  }

  .item-links {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 8px 0 0 30px;
  }

  .detail {
    max-width: 76ch;
    margin: 8px 0 0 30px;
    color: var(--ink-2);
    font-size: 14px;
    line-height: 1.65;
    white-space: pre-line;
  }

  .answer {
    margin: 8px 0 0 30px;
  }

  .answer summary {
    width: fit-content;
    color: var(--ink-3);
    font-family: var(--font-label);
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    list-style: none;
  }

  .answer summary::-webkit-details-marker {
    display: none;
  }

  .answer summary::before {
    content: '+ ';
  }

  .answer[open] summary::before {
    content: '− ';
  }

  .answer summary:hover,
  .answer[open] summary {
    color: var(--accent);
  }

  .answer .detail {
    margin-left: 0;
    padding: 10px 14px;
    border-left: 2px solid var(--border-strong);
  }

  /* Footer */
  .page-foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-top: 48px;
    padding-top: 24px;
    border-top: 1px solid var(--border);
  }

  .page-foot p {
    margin: 0;
    color: var(--ink-3);
    font-size: 13px;
  }

  .page-foot code {
    padding: 1px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--code-bg);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 12px;
  }

  @media (max-width: 768px) {
    .block-heading {
      grid-template-columns: 1fr;
      gap: 8px;
    }

    .block-heading p {
      text-align: left;
    }

    .artifact-summary {
      grid-template-columns: minmax(0, 1fr) 20px;
      gap: 12px 16px;
      padding: 16px 10px;
      margin: 0 -10px;
    }

    .artifact-meta {
      grid-column: 1;
      white-space: normal;
    }

    .module-head {
      grid-template-columns: 32px minmax(0, 1fr) 20px;
      gap: 8px 12px;
      padding: 16px 10px;
      margin: 0 -10px;
    }

    .module-progress {
      grid-column: 2;
    }

    .module-body {
      padding-left: 0;
    }

    .module-title {
      font-size: 18px;
    }
  }
</style>
