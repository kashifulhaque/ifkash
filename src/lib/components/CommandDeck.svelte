<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { tick } from "svelte";
  import CitedAnswer from "$lib/components/CitedAnswer.svelte";
  import { exaApi, type Answer, type SearchHit } from "$lib/exaApi";

  type Category = "Page" | "Project" | "Experiment" | "Tool";

  type Destination = {
    label: string;
    description: string;
    href: string;
    category: Category;
    keywords?: string;
    search: string;
  };

  const entries: Omit<Destination, "search">[] = [
    { label: "Home", description: "The front page and portfolio index", href: "/", category: "Page" },
    { label: "Work", description: "Roles, teams, and production systems", href: "/work", category: "Page", keywords: "rust c++ python llm" },
    { label: "Projects", description: "Open-source and independent builds", href: "/projects", category: "Page", keywords: "rust c++ python llm models" },
    { label: "Reinforcement learning", description: "Agents, environments, and training runs", href: "/rl", category: "Experiment" },
    { label: "Research radar", description: "New papers on post-training, RL, and inference", href: "/radar", category: "Page", keywords: "papers arxiv research exa news" },
    { label: "Interview roadmap", description: "A practical ML systems study path", href: "/roadmap", category: "Page" },
    { label: "Tools", description: "Useful things that run in the browser", href: "/tools", category: "Page" },
    { label: "Education", description: "Academic background and foundations", href: "/education", category: "Page" },
    { label: "Blog", description: "Engineering and machine-learning notes", href: "/blog", category: "Page" },
    { label: "Tiny planet", description: "Wander a low-poly world and find its wonders", href: "/game", category: "Experiment" },
    { label: "Fitness", description: "Workout, cardio, and progress tracker", href: "/fitness", category: "Tool" },
    { label: "PDF annotator", description: "Draw, sign, and write on PDFs locally", href: "/tools/pdf-annotator", category: "Tool" },
    { label: "Expense splitter", description: "Settle shared expenses with fewer payments", href: "/tools/splitter", category: "Tool" },
    { label: "banana.cpp", description: "A pure C++ inference engine for small LLMs", href: "/projects/banana-cpp", category: "Project" },
    { label: "smol-llama", description: "A 360M parameter LLaMA trained from scratch", href: "/projects/smol-llama", category: "Project" },
    { label: "Micro-Llama", description: "Bare-metal inference on 264 KB of SRAM", href: "/projects/vicharak", category: "Project" },
    { label: "smoltorch", description: "Autograd and neural networks in NumPy", href: "/projects/smoltorch", category: "Project" },
    { label: "NoPokeDB", description: "A compact vector database with crash recovery", href: "/projects/nopokedb", category: "Project" },
    { label: "PoliteLlama", description: "An ORPO-trained model with impeccable manners", href: "/projects/polite-orpo", category: "Project" }
  ];

  const destinations: Destination[] = entries.map((entry) => ({
    ...entry,
    search: `${entry.label} ${entry.description} ${entry.category} ${entry.keywords ?? ""}`.toLowerCase()
  }));

  let dialog: HTMLDialogElement;
  let searchInput: HTMLInputElement;
  let previousFocus: HTMLElement | null = null;
  let isOpen = false;
  let query = "";
  let activeIndex = 0;

  $: terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  $: filtered = terms.length
    ? destinations
        .map((destination) => ({ destination, score: score(destination, terms) }))
        .filter((match) => match.score >= 0)
        .sort((a, b) => b.score - a.score || a.destination.label.localeCompare(b.destination.label))
        .map((match) => match.destination)
    : destinations;

  // ── Site search and ask (Exa, through the Worker) ─────────────
  // Local destinations come first. With 3+ characters the deck also offers two
  // actions: search the site, blog, and GitHub, or ask a question about Kashif.
  // A query ending in "?" puts Ask first. When nothing local matches, the site
  // search runs on its own after a pause.

  type Row =
    | { kind: "dest"; d: Destination }
    | { kind: "ask" }
    | { kind: "search" }
    | { kind: "web"; hit: SearchHit };

  let webHits: SearchHit[] = [];
  let webQuery = "";
  let webLoading = false;
  let webError = "";
  let answer: Answer | null = null;
  let answerQuery = "";
  let askLoading = false;
  let askError = "";
  let autoTimer: ReturnType<typeof setTimeout> | undefined;

  $: q = query.trim();
  $: rows = buildRows(filtered, q, webQuery === q ? webHits : []);

  function buildRows(local: Destination[], text: string, hits: SearchHit[]): Row[] {
    const out: Row[] = local.map((d) => ({ kind: "dest", d }));
    if (text.length >= 3) {
      const actions: Row[] = text.endsWith("?")
        ? [{ kind: "ask" }, { kind: "search" }]
        : [{ kind: "search" }, { kind: "ask" }];
      if (text.endsWith("?")) out.unshift(...actions);
      else out.push(...actions);
    }
    for (const hit of hits) out.push({ kind: "web", hit });
    return out;
  }

  function onQueryInput(): void {
    activeIndex = 0;
    webError = "";
    askError = "";
    if (answerQuery !== query.trim()) answer = null;
    clearTimeout(autoTimer);
    const text = query.trim();
    if (text.length >= 3 && !text.endsWith("?") && filtered.length === 0) {
      autoTimer = setTimeout(() => void runSearch(text), 700);
    }
  }

  async function runSearch(text: string): Promise<void> {
    if (webQuery === text || webLoading) return;
    webLoading = true;
    webError = "";
    try {
      const res = await exaApi.search(text);
      webHits = res.results;
      webQuery = text;
      if (!res.results.length) webError = "Nothing on the site or blog matches that.";
    } catch (e) {
      webError = e instanceof Error ? e.message : "Search failed.";
    } finally {
      webLoading = false;
    }
  }

  async function runAsk(text: string): Promise<void> {
    if (askLoading) return;
    askLoading = true;
    askError = "";
    answer = null;
    try {
      answer = await exaApi.ask(text);
      answerQuery = text;
    } catch (e) {
      askError = e instanceof Error ? e.message : "Couldn't answer that.";
    } finally {
      askLoading = false;
    }
  }

  /** Where a search hit lives, for its category column. */
  function hitPlace(url: string): string {
    try {
      const host = new URL(url).hostname;
      if (host === "blog.ifkash.dev") return "Blog";
      if (host === "github.com") return "GitHub";
      if (host.endsWith("ifkash.dev")) return "Site";
    } catch {
      /* fall through */
    }
    return "Web";
  }

  /** Same-site hits navigate in the app; everything else opens normally. */
  function hitHref(url: string): string {
    try {
      const u = new URL(url);
      if (u.hostname === "ifkash.dev" && !u.pathname.startsWith("/api/")) return u.pathname + u.search;
    } catch {
      /* fall through */
    }
    return url;
  }

  function activate(row: Row): void {
    if (row.kind === "dest") visit(row.d);
    else if (row.kind === "ask") void runAsk(q);
    else if (row.kind === "search") void runSearch(q);
    else {
      const href = hitHref(row.hit.url);
      closeDeck();
      if (href.startsWith("/")) void goto(href);
      else window.open(href, "_blank", "noopener");
    }
  }

  function score(destination: Destination, searchTerms: string[]): number {
    const label = destination.label.toLowerCase();
    let total = 0;

    for (const term of searchTerms) {
      if (!destination.search.includes(term)) return -1;
      if (label === term) total += 20;
      else if (label.startsWith(term)) total += 12;
      else if (label.includes(term)) total += 7;
      else total += 2;
    }

    return total;
  }

  function isEditable(target: EventTarget | null): boolean {
    return target instanceof HTMLElement &&
      (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable);
  }

  async function openDeck(): Promise<void> {
    if (dialog.open) return;
    previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    query = "";
    activeIndex = 0;
    answer = null;
    askError = "";
    webError = "";
    dialog.showModal();
    isOpen = true;
    await tick();
    searchInput.focus();
  }

  function closeDeck(): void {
    if (dialog.open) dialog.close();
  }

  function handleDialogClose(): void {
    isOpen = false;
    previousFocus?.focus();
  }

  function toggleDeck(): void {
    if (dialog.open) closeDeck();
    else void openDeck();
  }

  function revealActive(): void {
    void tick().then(() => {
      dialog.querySelector<HTMLElement>('[data-active="true"]')?.scrollIntoView({ block: "nearest" });
    });
  }

  function moveActive(delta: number): void {
    if (!rows.length) return;
    activeIndex = (activeIndex + delta + rows.length) % rows.length;
    revealActive();
  }

  function visit(destination: Destination): void {
    closeDeck();
    void goto(destination.href);
  }

  function surpriseMe(): void {
    const choices = destinations.filter((destination) => destination.href !== $page.url.pathname);
    visit(choices[Math.floor(Math.random() * choices.length)]);
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    const commandKey = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k";
    const slashKey = event.key === "/" && !event.metaKey && !event.ctrlKey && !event.altKey;

    if (commandKey || (slashKey && !isEditable(event.target))) {
      event.preventDefault();
      toggleDeck();
      return;
    }

    if (!isOpen) return;
    if (event.key === "Escape") {
      event.preventDefault();
      closeDeck();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      moveActive(1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      moveActive(-1);
    } else if (event.key === "Enter" && event.target === searchInput && rows[activeIndex]) {
      event.preventDefault();
      activate(rows[activeIndex]);
    }
  }
</script>

<svelte:window on:keydown={handleWindowKeydown} />

<button class="deck-trigger" type="button" on:click={openDeck} aria-haspopup="dialog">
  <svg viewBox="0 0 24 24" aria-hidden="true">
    <circle cx="11" cy="11" r="6.5"></circle>
    <path d="m16 16 4.5 4.5"></path>
  </svg>
  <span class="trigger-label">Search</span>
  <kbd>⌘ K</kbd>
</button>

<dialog
  bind:this={dialog}
  class="deck-dialog"
  aria-labelledby="deck-title"
  on:close={handleDialogClose}
>
  <button class="deck-backdrop" type="button" on:click={closeDeck} tabindex="-1" aria-hidden="true"></button>
  <section class="deck" aria-describedby="deck-description">
    <header class="deck-header">
      <div>
        <p class="eyebrow">Portfolio signal deck</p>
        <h2 id="deck-title">Where should we jump?</h2>
        <p id="deck-description">Jump to a page, search the site and blog, or ask about my work.</p>
      </div>
      <button class="close" type="button" on:click={closeDeck} aria-label="Close jump deck">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M18 6 6 18M6 6l12 12"></path>
        </svg>
      </button>
    </header>

    <label class="search-shell">
      <span aria-hidden="true">/</span>
      <input
        bind:this={searchInput}
        bind:value={query}
        type="search"
        aria-label="Search portfolio"
        placeholder="Try ‘rust’, or ask ‘what has Kashif built with RL?’"
        autocomplete="off"
        spellcheck="false"
        on:input={onQueryInput}
      />
      <kbd>esc</kbd>
      {#if webLoading || askLoading}
        <span class="search-progress" aria-hidden="true"></span>
      {/if}
    </label>

    {#if askLoading || answer || askError}
      <div class="answer-panel" aria-live="polite">
        {#if askLoading}
          <p class="answer-status working">Reading ifkash.dev and the blog</p>
          <div class="skeleton" aria-hidden="true">
            <span></span><span></span><span class="short"></span>
          </div>
        {:else if askError}
          <p class="answer-status error">{askError}</p>
        {:else if answer}
          <p class="answer-label">Answer · from my own pages</p>
          <CitedAnswer answer={answer.answer} sources={answer.sources} />
        {/if}
      </div>
    {/if}

    <div class="result-meta" aria-live="polite">
      <span>
        {filtered.length} {filtered.length === 1 ? "signal" : "signals"}
        {#if webLoading}· <span class="working">searching the site and blog</span>{:else if webQuery === q && webHits.length}· {webHits.length} from the web index{:else if webError}· {webError}{/if}
      </span>
      <button type="button" on:click={surpriseMe}>Surprise me ↗</button>
    </div>

    <nav class="results" aria-label="Jump destinations">
      {#each rows as row, index}
        {#if row.kind === "dest"}
          <a
            href={row.d.href}
            class:active={index === activeIndex}
            data-active={index === activeIndex}
            on:click={closeDeck}
            on:focus={() => (activeIndex = index)}
          >
            <span class="result-category">{row.d.category}</span>
            <span class="result-copy">
              <strong>{row.d.label}</strong>
              <small>{row.d.description}</small>
            </span>
            <span class="result-arrow" aria-hidden="true">↗</span>
          </a>
        {:else if row.kind === "web"}
          <a
            href={hitHref(row.hit.url)}
            target={hitHref(row.hit.url).startsWith("/") ? undefined : "_blank"}
            rel="noopener"
            class:active={index === activeIndex}
            data-active={index === activeIndex}
            on:click={closeDeck}
            on:focus={() => (activeIndex = index)}
          >
            <span class="result-category">{hitPlace(row.hit.url)}</span>
            <span class="result-copy">
              <strong>{row.hit.title}</strong>
              <small>{row.hit.snippet || row.hit.url}</small>
            </span>
            <span class="result-arrow" aria-hidden="true">↗</span>
          </a>
        {:else}
          <button
            type="button"
            class="row-action"
            class:active={index === activeIndex}
            data-active={index === activeIndex}
            disabled={row.kind === "ask" ? askLoading : webLoading}
            on:click={() => activate(row)}
            on:focus={() => (activeIndex = index)}
          >
            <span class="result-category">{row.kind === "ask" ? "Ask" : "Search"}</span>
            <span class="result-copy">
              <strong>
                {#if row.kind === "ask"}
                  {askLoading ? "Asking" : `Ask: “${q}”`}
                {:else}
                  {webLoading ? "Searching the site and blog" : `Search the site and blog for “${q}”`}
                {/if}
              </strong>
              <small>
                {row.kind === "ask"
                  ? "A short answer from my own pages, with sources"
                  : "Pages, posts, and repos, through Exa"}
              </small>
            </span>
            <span class="result-arrow" aria-hidden="true">↵</span>
          </button>
        {/if}
      {:else}
        <div class="empty">
          <span class="empty-mark" aria-hidden="true">?</span>
          <p>No signal found.</p>
          <button type="button" on:click={surpriseMe}>Take me somewhere unexpected</button>
        </div>
      {/each}
      {#if webLoading}
        {#each [0, 1, 2] as i}
          <div class="row-skeleton" aria-hidden="true" style:--delay="{i * 120}ms">
            <span class="sk-cat"></span>
            <span class="sk-copy"><span></span><span class="short"></span></span>
          </div>
        {/each}
      {/if}
    </nav>

    <footer class="deck-footer" aria-hidden="true">
      <span><kbd>↑</kbd><kbd>↓</kbd> move</span>
      <span><kbd>enter</kbd> jump or ask</span>
      <span><kbd>/</kbd> open anywhere</span>
    </footer>
  </section>
</dialog>

<style>
  /* ── Trigger (sits in the top bar) ───────────────────────── */

  .deck-trigger {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    height: 32px;
    padding: 0 var(--s-1) 0 var(--s-3);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    line-height: 1;
    white-space: nowrap;
    cursor: pointer;
    transition:
      border-color var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease),
      background-color var(--dur-fast) var(--ease);
  }

  .deck-trigger:hover {
    border-color: var(--line-strong);
    background: var(--hover-wash);
    color: var(--ink);
  }

  .deck-trigger svg,
  .close svg {
    width: 14px;
    height: 14px;
    flex: 0 0 auto;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .deck-trigger svg {
    color: var(--ink-3);
  }

  .trigger-label {
    padding-right: var(--s-4);
  }

  kbd {
    display: inline-grid;
    place-items: center;
    min-width: 20px;
    height: 20px;
    padding: 0 var(--s-1);
    border: 1px solid var(--line);
    border-radius: var(--radius-xs);
    background: var(--surface-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 400;
    line-height: 1;
    letter-spacing: 0;
  }

  /* ── Dialog shell ────────────────────────────────────────── */

  .deck-dialog {
    width: 100vw;
    max-width: none;
    height: 100dvh;
    max-height: none;
    margin: 0;
    padding: 0;
    border: 0;
    overflow: hidden;
    background: transparent;
    color: var(--ink);
  }

  .deck-dialog::backdrop {
    background: var(--scrim);
  }

  .deck-backdrop {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    border: 0;
    background: transparent;
    cursor: default;
  }

  .deck {
    position: relative;
    display: flex;
    flex-direction: column;
    width: min(640px, calc(100vw - 2 * var(--gutter)));
    max-height: min(680px, calc(100dvh - 2 * var(--s-8)));
    margin: clamp(var(--s-6), 12vh, var(--s-9)) auto 0;
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
    box-shadow: 0 12px 32px -12px var(--shadow-color);
    animation: deck-arrive var(--dur) var(--ease-out) both;
  }

  .deck-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-5);
    padding: var(--s-5) var(--s-5) var(--s-4);
  }

  .deck-header .eyebrow {
    margin: 0 0 var(--s-2);
  }

  .deck-header h2 {
    margin: 0;
    color: var(--ink);
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.2;
    letter-spacing: -0.02em;
  }

  .deck-header p:last-child {
    margin: var(--s-1) 0 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .close {
    display: inline-grid;
    place-items: center;
    flex: 0 0 auto;
    width: 32px;
    height: 32px;
    margin: calc(-1 * var(--s-1)) calc(-1 * var(--s-2)) 0 0;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-3);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background-color var(--dur-fast) var(--ease);
  }

  .close:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }

  /* ── Search field ────────────────────────────────────────── */

  .search-shell {
    position: relative;
    display: grid;
    /* overflow: hidden (for the progress bar) would let the deck's flex column
       squeeze the field; keep it at full height. */
    flex-shrink: 0;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--s-3);
    margin: 0 var(--s-5);
    padding: 0 var(--s-2) 0 var(--s-3);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--bg);
    transition: border-color var(--dur-fast) var(--ease), box-shadow var(--dur-fast) var(--ease);
  }

  .search-shell:focus-within {
    border-color: var(--fjord);
    box-shadow: 0 0 0 3px var(--fjord-wash);
  }

  .search-shell > span {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .search-shell input {
    width: 100%;
    height: 44px;
    padding: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-base);
  }

  .search-shell input:focus-visible {
    outline: 0;
  }

  .search-shell input::placeholder {
    color: var(--ink-3);
  }

  .search-shell input::-webkit-search-cancel-button {
    display: none;
  }

  /* ── Result count and results ────────────────────────────── */

  .result-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-4) var(--s-5) var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .result-meta button,
  .empty button {
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--ink-2);
    font: inherit;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease);
  }

  .result-meta button:hover,
  .empty button:hover {
    color: var(--fjord);
  }

  .results {
    min-height: 120px;
    overflow-y: auto;
    overscroll-behavior: contain;
    border-top: 1px solid var(--line);
  }

  .results a,
  .results .row-action {
    display: grid;
    grid-template-columns: 88px minmax(0, 1fr) 16px;
    align-items: center;
    gap: var(--s-4);
    min-height: 56px;
    padding: var(--s-3) var(--s-5);
    border-bottom: 1px solid var(--line);
    color: var(--ink);
    text-decoration: none;
    transition: background-color var(--dur-fast) var(--ease);
  }

  .results a:last-child,
  .results .row-action:last-child {
    border-bottom: 0;
  }

  .results a:hover,
  .results .row-action:hover {
    background: var(--hover-wash);
  }

  .results .row-action {
    width: 100%;
    border-top: 0;
    border-right: 0;
    border-left: 0;
    background: transparent;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .results .row-action:disabled {
    cursor: default;
  }

  /* ── Working state: search or ask in flight ──────────────── */

  /* An indeterminate bar along the bottom of the field. */
  .search-progress {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    height: 2px;
    overflow: hidden;
    background: var(--fjord-wash);
  }

  .search-progress::after {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    width: 35%;
    background: var(--fjord);
    animation: deck-progress 1.1s var(--ease) infinite;
  }

  /* Animated ellipsis after a working label. */
  .working::after {
    content: "…";
    display: inline-block;
    width: 1.2em;
    overflow: hidden;
    vertical-align: bottom;
    animation: deck-dots 1.2s steps(4, end) infinite;
  }

  .answer-status.working::after {
    content: "...";
  }

  .skeleton {
    display: grid;
    gap: var(--s-2);
    margin-top: var(--s-3);
  }

  .skeleton span,
  .row-skeleton .sk-cat,
  .row-skeleton .sk-copy span {
    display: block;
    height: 10px;
    border-radius: var(--radius-xs);
    background: linear-gradient(90deg, var(--line) 0%, var(--hover-wash) 50%, var(--line) 100%);
    background-size: 200% 100%;
    animation: deck-shimmer 1.4s linear infinite;
    animation-delay: var(--delay, 0ms);
  }

  .skeleton .short,
  .row-skeleton .short {
    width: 60%;
  }

  .row-skeleton {
    display: grid;
    grid-template-columns: 88px minmax(0, 1fr) 16px;
    align-items: center;
    gap: var(--s-4);
    min-height: 56px;
    padding: var(--s-3) var(--s-5);
    border-bottom: 1px solid var(--line);
  }

  .row-skeleton .sk-cat {
    width: 44px;
  }

  .row-skeleton .sk-copy {
    display: grid;
    gap: var(--s-2);
  }

  @keyframes deck-progress {
    from {
      left: -35%;
    }
    to {
      left: 100%;
    }
  }

  @keyframes deck-dots {
    from {
      width: 0;
    }
    to {
      width: 1.2em;
    }
  }

  @keyframes deck-shimmer {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }

  .results .row-action.active {
    background: var(--fjord-wash);
  }

  .results .row-action.active strong {
    color: var(--fjord);
  }

  .answer-panel {
    flex-shrink: 0;
    max-height: 260px;
    margin: var(--s-4) var(--s-5) 0;
    padding: var(--s-4);
    overflow-y: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--bg);
  }

  .answer-label {
    margin: 0 0 var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .answer-status {
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-sm);
  }

  .answer-status.error {
    color: var(--falu);
  }

  .results a:focus-visible {
    outline-offset: -2px;
  }

  .results a.active {
    background: var(--fjord-wash);
  }

  .results a.active strong {
    color: var(--fjord);
  }

  .results a.active .result-arrow,
  .results a:hover .result-arrow {
    color: var(--fjord);
    transform: translate(2px, -2px);
  }

  .result-category {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .result-copy {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .result-copy strong {
    color: var(--ink);
    font-size: 15px;
    font-weight: 500;
    line-height: 1.3;
    letter-spacing: -0.005em;
    transition: color var(--dur-fast) var(--ease);
  }

  .result-copy small {
    overflow: hidden;
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.4;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-arrow {
    justify-self: end;
    color: var(--ink-3);
    font-size: var(--text-sm);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .empty {
    display: grid;
    place-items: center;
    gap: var(--s-2);
    min-height: 180px;
    padding: var(--s-6);
    text-align: center;
  }

  .empty-mark {
    display: grid;
    width: 32px;
    height: 32px;
    place-items: center;
    border: 1px solid var(--line);
    border-radius: var(--radius-pill);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .empty p {
    margin: 0;
    color: var(--ink-2);
  }

  .empty button {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  /* ── Footer hints ────────────────────────────────────────── */

  .deck-footer {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2) var(--s-5);
    padding: var(--s-3) var(--s-5);
    border-top: 1px solid var(--line);
    background: var(--surface-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .deck-footer span {
    display: inline-flex;
    align-items: center;
    gap: var(--s-1);
  }

  .deck-footer kbd {
    background: var(--surface);
  }

  @keyframes deck-arrive {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  @media (max-width: 640px) {
    /* Icon-only 32×32 trigger */
    .deck-trigger {
      justify-content: center;
      width: 32px;
      padding: 0;
    }

    .deck-trigger kbd {
      display: none;
    }

    .trigger-label {
      position: absolute;
      width: 1px;
      height: 1px;
      padding: 0;
      overflow: hidden;
      clip: rect(0 0 0 0);
      white-space: nowrap;
    }

    .deck {
      width: calc(100vw - 2 * var(--s-3));
      max-height: calc(100dvh - 2 * var(--s-3));
      margin-top: var(--s-3);
    }

    .deck-header {
      padding: var(--s-4) var(--s-4) var(--s-3);
    }

    .search-shell {
      margin: 0 var(--s-4);
    }

    .result-meta {
      padding-inline: var(--s-4);
    }

    .answer-panel {
      margin-inline: var(--s-4);
    }

    .row-skeleton {
      grid-template-columns: minmax(0, 1fr);
      padding-inline: var(--s-4);
    }

    .row-skeleton .sk-cat {
      display: none;
    }

    .results a,
    .results .row-action {
      grid-template-columns: minmax(0, 1fr) 16px;
      gap: var(--s-3);
      padding-inline: var(--s-4);
    }

    .result-category {
      display: none;
    }

    .deck-footer {
      display: none;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .deck,
    .working::after {
      animation: none;
    }

    .search-progress::after {
      left: 0;
      width: 100%;
      animation: none;
      opacity: 0.6;
    }

    .skeleton span,
    .row-skeleton .sk-cat,
    .row-skeleton .sk-copy span {
      animation: none;
    }
  }
</style>
