<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { tick } from "svelte";

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
    if (!filtered.length) return;
    activeIndex = (activeIndex + delta + filtered.length) % filtered.length;
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
    } else if (event.key === "Enter" && event.target === searchInput && filtered[activeIndex]) {
      event.preventDefault();
      visit(filtered[activeIndex]);
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
        <p id="deck-description">Search every corner, or let the site choose.</p>
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
        placeholder="Try ‘rust’, ‘LLM’, or ‘tool’"
        autocomplete="off"
        spellcheck="false"
        on:input={() => (activeIndex = 0)}
      />
      <kbd>esc</kbd>
    </label>

    <div class="result-meta" aria-live="polite">
      <span>{filtered.length} {filtered.length === 1 ? "signal" : "signals"}</span>
      <button type="button" on:click={surpriseMe}>Surprise me ↗</button>
    </div>

    <nav class="results" aria-label="Jump destinations">
      {#each filtered as destination, index}
        <a
          href={destination.href}
          class:active={index === activeIndex}
          data-active={index === activeIndex}
          on:click={closeDeck}
          on:focus={() => (activeIndex = index)}
        >
          <span class="result-category">{destination.category}</span>
          <span class="result-copy">
            <strong>{destination.label}</strong>
            <small>{destination.description}</small>
          </span>
          <span class="result-arrow" aria-hidden="true">↗</span>
        </a>
      {:else}
        <div class="empty">
          <span class="empty-mark" aria-hidden="true">?</span>
          <p>No signal found.</p>
          <button type="button" on:click={surpriseMe}>Take me somewhere unexpected</button>
        </div>
      {/each}
    </nav>

    <footer class="deck-footer" aria-hidden="true">
      <span><kbd>↑</kbd><kbd>↓</kbd> move</span>
      <span><kbd>enter</kbd> jump</span>
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
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--s-3);
    margin: 0 var(--s-5);
    padding: 0 var(--s-2) 0 var(--s-3);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
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

  .results a {
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

  .results a:last-child {
    border-bottom: 0;
  }

  .results a:hover {
    background: var(--hover-wash);
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

    .results a {
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
    .deck {
      animation: none;
    }
  }
</style>
