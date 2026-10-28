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
  import { reveal, splitReveal } from '$lib/motion';
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

  function share(finished: number, total: number): number {
    return total === 0 ? 0 : Math.round((finished / total) * 100);
  }

  $: doneCount = done.size;
  $: percent = TOTAL_ITEMS === 0 ? 0 : Math.round((doneCount / TOTAL_ITEMS) * 100);
</script>

<header class="page-head">
  <p class="eyebrow">08 / Roadmap</p>
  <h1 class="page-title" use:splitReveal>Roadmap</h1>
  <p class="lead">
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

  <!-- Per-module overview: one short track per module, in course order. -->
  <ol class="meter" aria-hidden="true">
    {#each MODULES as mod}
      {@const finished = countDone(mod.items, done)}
      <li class:complete={finished === mod.items.length && mod.items.length > 0}>
        <span class="meter-track">
          <span class="meter-fill" style="width: {share(finished, mod.items.length)}%"></span>
        </span>
        <span class="meter-n">{mod.n}</span>
      </li>
    {/each}
  </ol>

  <p class="progress-note">
    Progress is stored in this browser. Work through the modules in order. Check an item when
    you can do it or answer it without notes, and uncheck it when you cannot.
  </p>
</section>

<section class="block" aria-labelledby="built-heading">
  <div class="block-heading">
    <h2 id="built-heading"><span class="block-n" aria-hidden="true">A</span>Built so far</h2>
    <p>The numbers to have ready. Every module that follows points back to these.</p>
  </div>

  <div class="artifact-list" use:reveal={{ children: '.artifact', gap: 70 }}>
    {#each ARTIFACTS as artifact}
      <details class="artifact">
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
          <div class="artifact-cols">
            <div>
              <p class="group-label">Talking points</p>
              <ul class="fact-list">
                {#each artifact.facts as fact}
                  <li>{fact}</li>
                {/each}
              </ul>
            </div>
            <div>
              <p class="group-label gap-label">Shore up</p>
              <ul class="fact-list gap-list">
                {#each artifact.gaps as gap}
                  <li>{gap}</li>
                {/each}
              </ul>
            </div>
          </div>
        </div>
      </details>
    {/each}
  </div>
</section>

<section class="block" aria-labelledby="course-heading">
  <div class="block-heading">
    <h2 id="course-heading"><span class="block-n" aria-hidden="true">B</span>Course</h2>
    <p>Twelve modules, in order. Each one lists what you already did, what to read, what to build, and the questions to answer cold.</p>
  </div>

  <ol class="module-list" use:reveal={{ children: '.module', gap: 45, y: 14 }}>
    {#each MODULES as mod}
      {@const total = mod.items.length}
      {@const finished = countDone(mod.items, done)}
      {@const isOpen = open.has(mod.id)}
      {@const complete = finished === total && total > 0}
      <li class="module" class:open={isOpen}>
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
          <span class="module-progress" class:complete>
            <span class="module-track" aria-hidden="true">
              <span class="module-fill" style="width: {share(finished, total)}%"></span>
            </span>
            {finished}/{total}
          </span>
          <span class="chev" aria-hidden="true">{isOpen ? '−' : '+'}</span>
        </button>

        {#if isOpen}
          <div class="module-body" id="module-{mod.id}">
            {#if mod.anchors.length}
              <div class="group">
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
              </div>
            {/if}

            {#each KIND_ORDER as kind}
              {@const items = ofKind(mod.items, kind)}
              {#if items.length}
                <div class="group">
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
                </div>
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
  <button type="button" class="btn btn--ghost btn-sm" on:click={resetProgress}>Reset progress</button>
</footer>

<style>
  /* ── Overall progress ─────────────────────────────────────── */

  .progress {
    display: grid;
    gap: var(--s-4);
    padding: var(--s-7) 0 0;
  }

  .progress-row {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-2) var(--s-4);
  }

  .progress-count {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }

  .progress-count strong {
    margin-right: var(--s-1);
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-2xl);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    line-height: 1;
    letter-spacing: -0.04em;
  }

  .progress-pct {
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }

  .progress-bar {
    position: relative;
    height: 2px;
    overflow: hidden;
    background: var(--line);
  }

  .progress-fill {
    display: block;
    height: 100%;
    background: var(--moss);
    transition: width var(--dur-slow) var(--ease-out);
  }

  .meter {
    display: grid;
    grid-template-columns: repeat(12, minmax(0, 1fr));
    gap: var(--s-2);
    margin: var(--s-2) 0 0;
    padding: 0;
    list-style: none;
  }

  .meter li {
    display: grid;
    gap: var(--s-2);
  }

  .meter-track {
    display: block;
    height: 2px;
    overflow: hidden;
    background: var(--line);
  }

  .meter-fill {
    display: block;
    height: 100%;
    background: var(--ink-3);
    transition: width var(--dur-slow) var(--ease-out);
  }

  .meter-n {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .meter li.complete .meter-fill {
    background: var(--moss);
  }
  .meter li.complete .meter-n {
    color: var(--moss);
  }

  .progress-note {
    max-width: var(--measure);
    margin: var(--s-2) 0 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  /* ── Section blocks ───────────────────────────────────────── */

  .block {
    padding-top: var(--s-9);
  }

  .block-heading {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 44ch);
    align-items: end;
    gap: var(--s-5);
    margin-bottom: var(--s-6);
  }

  .block-heading h2 {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    font-size: var(--text-xl);
  }

  .block-n {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    letter-spacing: 0;
  }

  .block-heading p {
    margin: 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.55;
  }

  .chev {
    justify-self: end;
    width: 20px;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-base);
    line-height: 1;
    text-align: center;
    transition: color var(--dur-fast) var(--ease);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    height: 24px;
    padding: 0 var(--s-2);
    border: 1px solid var(--line);
    border-radius: var(--radius-xs);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    text-decoration: none;
    transition:
      color var(--dur-fast) var(--ease),
      border-color var(--dur-fast) var(--ease);
  }

  .chip:hover {
    border-color: var(--fjord);
    color: var(--fjord);
  }

  .group + .group {
    margin-top: var(--s-6);
  }

  .group-label {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    margin: 0 0 var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .group-count {
    color: var(--ink-3);
    font-variant-numeric: tabular-nums;
  }
  .group-count::before {
    content: '·';
    margin-right: var(--s-3);
  }

  /* ── Built so far ─────────────────────────────────────────── */

  .artifact-list {
    border-top: 1px solid var(--line);
  }

  .artifact {
    border-bottom: 1px solid var(--line);
  }

  .artifact-summary {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto 20px;
    align-items: baseline;
    gap: var(--s-5);
    padding: var(--s-5) 0;
    color: var(--ink);
    list-style: none;
    cursor: pointer;
  }

  .artifact-summary::-webkit-details-marker {
    display: none;
  }

  .artifact-title {
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.25;
    letter-spacing: -0.015em;
    transition: color var(--dur-fast) var(--ease);
  }

  .artifact-summary:hover .artifact-title,
  .artifact-summary:hover .chev,
  .artifact[open] .chev {
    color: var(--fjord);
  }

  .artifact-meta {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .artifact-body {
    padding: 0 0 var(--s-6);
    animation: rise var(--dur) var(--ease-out) both;
  }

  .artifact-links {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-bottom: var(--s-5);
  }

  .artifact-cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: var(--s-6) var(--s-7);
  }

  .fact-list {
    display: grid;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .fact-list li {
    position: relative;
    padding-left: var(--s-5);
    color: var(--ink-2);
    font-size: 15px;
    line-height: 1.6;
  }

  .fact-list li::before {
    content: '–';
    position: absolute;
    left: 0;
    color: var(--ink-3);
  }

  .gap-label {
    color: var(--clay);
  }

  .gap-list li::before {
    content: '';
    top: 0.68em;
    left: 2px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    border: 1px solid var(--clay);
  }

  /* ── Course ───────────────────────────────────────────────── */

  .module-list {
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--line);
    list-style: none;
  }

  .module {
    border-bottom: 1px solid var(--line);
  }

  .module-head {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) auto 20px;
    align-items: baseline;
    column-gap: var(--s-5);
    width: 100%;
    padding: var(--s-5) 0;
    border: 0;
    background: transparent;
    color: var(--ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .module-n {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .module-main {
    display: grid;
    gap: var(--s-1);
    min-width: 0;
  }

  .module-title {
    color: var(--ink);
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.25;
    letter-spacing: -0.015em;
    transition: color var(--dur-fast) var(--ease);
  }

  .module-why {
    max-width: var(--measure);
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.55;
  }

  .module-head:hover .module-title,
  .module-head:hover .chev,
  .module.open .chev {
    color: var(--fjord);
  }

  .module-progress {
    display: inline-flex;
    align-items: center;
    gap: var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .module-track {
    display: block;
    width: 56px;
    height: 2px;
    overflow: hidden;
    background: var(--line);
  }

  .module-fill {
    display: block;
    height: 100%;
    background: var(--ink-3);
    transition: width var(--dur-slow) var(--ease-out);
  }

  .module-progress.complete {
    color: var(--moss);
  }
  .module-progress.complete .module-fill {
    background: var(--moss);
  }

  /* Body aligns with the title column: 48px index + 24px gap. */
  .module-body {
    padding: 0 0 var(--s-7) calc(48px + var(--s-5));
    animation: rise var(--dur) var(--ease-out) both;
  }

  .anchor-list {
    display: grid;
    gap: var(--s-2);
    max-width: 78ch;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .anchor-list li {
    color: var(--ink-2);
    font-size: 15px;
    line-height: 1.6;
  }

  .anchor-link {
    margin-right: var(--s-2);
    color: var(--ink);
    font-weight: 500;
    text-decoration: underline;
    text-decoration-color: var(--line-strong);
    text-decoration-thickness: 1px;
    text-underline-offset: 0.22em;
    transition:
      color var(--dur-fast) var(--ease),
      text-decoration-color var(--dur-fast) var(--ease);
  }

  .anchor-link:hover {
    color: var(--fjord);
    text-decoration-color: var(--fjord);
  }

  .anchor-note {
    color: var(--ink-2);
  }

  .item-list {
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--line);
    list-style: none;
  }

  .item {
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .item-row {
    display: grid;
    grid-template-columns: 16px minmax(0, 1fr);
    align-items: start;
    gap: var(--s-3);
    color: var(--ink);
    font-size: 15px;
    line-height: 1.55;
    cursor: pointer;
  }

  /* A quiet square checkbox: hairline when open, moss when done. */
  .item-row input {
    position: relative;
    width: 16px;
    height: 16px;
    margin: 4px 0 0;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-xs);
    background: var(--surface);
    appearance: none;
    -webkit-appearance: none;
    cursor: pointer;
    transition:
      background-color var(--dur-fast) var(--ease),
      border-color var(--dur-fast) var(--ease);
  }

  .item-row:hover input {
    border-color: var(--ink-3);
  }

  .item-row input:checked {
    border-color: var(--moss);
    background: var(--moss);
  }

  .item-row input:checked::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 4.5px;
    width: 4px;
    height: 8px;
    border: solid var(--ink-inverse);
    border-width: 0 1.5px 1.5px 0;
    transform: rotate(45deg);
  }

  .item-title {
    transition: color var(--dur-fast) var(--ease);
  }

  .item.done .item-title {
    color: var(--ink-3);
    text-decoration: line-through;
    text-decoration-color: var(--line-strong);
    text-decoration-thickness: 1px;
  }

  .item-links {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin: var(--s-2) 0 0 28px;
  }

  .detail {
    max-width: 76ch;
    margin: var(--s-2) 0 0 28px;
    color: var(--ink-2);
    font-size: 14px;
    line-height: 1.65;
    white-space: pre-line;
  }

  .answer {
    margin: var(--s-2) 0 0 28px;
  }

  .answer summary {
    display: inline-flex;
    gap: var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    list-style: none;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease);
  }

  .answer summary::-webkit-details-marker {
    display: none;
  }

  .answer summary::before {
    content: '+';
  }

  .answer[open] summary::before {
    content: '−';
  }

  .answer summary:hover,
  .answer[open] summary {
    color: var(--fjord);
  }

  .answer .detail {
    margin: var(--s-2) 0 var(--s-1);
    padding: var(--s-1) 0 var(--s-1) var(--s-4);
    border-left: 1px solid var(--line-strong);
  }

  /* ── Footer ───────────────────────────────────────────────── */

  .page-foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    margin-top: var(--s-8);
    padding-top: var(--s-5);
    border-top: 1px solid var(--line);
  }

  .page-foot p {
    margin: 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .page-foot code {
    overflow-wrap: anywhere;
  }

  @media (max-width: 900px) {
    .artifact-cols {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  @media (max-width: 720px) {
    .progress {
      padding-top: var(--s-6);
    }

    .meter {
      grid-template-columns: repeat(6, minmax(0, 1fr));
      row-gap: var(--s-3);
    }

    .block {
      padding-top: var(--s-8);
    }

    .block-heading {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--s-2);
    }

    .artifact-summary {
      grid-template-columns: minmax(0, 1fr) 20px;
      gap: var(--s-2) var(--s-4);
      padding: var(--s-4) 0;
    }

    .artifact-meta {
      grid-column: 1;
      grid-row: 2;
      white-space: normal;
    }

    .module-head {
      grid-template-columns: 32px minmax(0, 1fr) 20px;
      gap: var(--s-2) var(--s-3);
      padding: var(--s-4) 0;
    }

    .module-progress {
      grid-column: 2;
      grid-row: 2;
    }

    .module-head .chev {
      grid-column: 3;
      grid-row: 1;
    }

    .module-body {
      padding-left: 0;
    }

    .module-title,
    .artifact-title {
      font-size: var(--text-md);
    }
  }
</style>
