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
  <span>Jump</span>
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
      <button class="close" type="button" on:click={closeDeck} aria-label="Close jump deck">×</button>
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
  .deck-trigger {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 28px;
    padding: 3px 4px 3px 9px;
    border: 1px solid var(--border);
    border-radius: var(--r-frame-2);
    background: transparent;
    color: var(--ink-2);
    font: 500 13px/1 var(--font-sans);
    transition: border-color 160ms var(--ease-inout), color 160ms var(--ease-inout), background 160ms var(--ease-inout);
  }

  .deck-trigger:hover {
    border-color: var(--border-strong);
    background: var(--hover);
    color: var(--foreground);
  }

  kbd {
    font: 500 10px/1 var(--font-mono);
    letter-spacing: -0.02em;
  }

  .deck-trigger kbd {
    padding: 4px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--ink-3);
    background: var(--popover);
  }

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
    color: var(--foreground);
  }

  .deck-dialog::backdrop {
    background: color-mix(in srgb, var(--background) 72%, transparent);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
  }

  .deck-backdrop {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    border: 0;
    background: transparent;
  }

  .deck {
    position: relative;
    display: flex;
    flex-direction: column;
    width: min(680px, calc(100vw - 32px));
    max-height: min(720px, calc(100dvh - 64px));
    margin: clamp(32px, 10vh, 96px) auto 0;
    border: 1px solid var(--border-strong);
    border-radius: var(--r-frame-1);
    overflow: hidden;
    background: var(--background);
    box-shadow: 0 24px 80px var(--shadow-color);
    animation: deck-arrive 220ms var(--ease-out-quart) both;
  }

  .deck::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    background-image: radial-gradient(circle, var(--dots) 1px, transparent 1px);
    background-size: 18px 18px;
    mask-image: linear-gradient(to bottom, black, transparent 42%);
  }

  .deck-header {
    position: relative;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    padding: 24px 24px 20px;
  }

  .eyebrow {
    margin: 0 0 5px;
    color: var(--accent);
    font: 500 10px/1.4 var(--font-mono);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .deck-header h2 {
    margin: 0;
    color: var(--foreground);
    font-size: clamp(26px, 5vw, 36px);
    font-weight: 400;
    line-height: 1.1;
    letter-spacing: -0.025em;
  }

  .deck-header p:last-child {
    margin: 7px 0 0;
    color: var(--ink-3);
    font-size: 13px;
  }

  .close {
    width: 32px;
    height: 32px;
    flex: 0 0 auto;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--r-frame-3);
    background: var(--background);
    color: var(--ink-2);
    font: 400 22px/1 var(--font-sans);
  }

  .close:hover {
    border-color: var(--border-strong);
    color: var(--foreground);
  }

  .search-shell {
    position: relative;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    margin: 0 16px;
    padding: 0 12px;
    border: 1px solid var(--border-strong);
    border-radius: var(--r-frame-2);
    background: var(--popover);
  }

  .search-shell > span {
    color: var(--accent);
    font: 500 18px/1 var(--font-mono);
  }

  .search-shell input {
    width: 100%;
    height: 48px;
    padding: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: var(--foreground);
    font: 400 16px/1 var(--font-sans);
  }

  .search-shell input::placeholder {
    color: var(--ink-3);
  }

  .search-shell kbd {
    padding: 4px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--ink-3);
    background: var(--background);
  }

  .result-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 39px;
    padding: 8px 20px 6px;
    color: var(--ink-3);
    font: 500 10px/1.4 var(--font-mono);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .result-meta button,
  .empty button {
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--accent);
    font: inherit;
    text-transform: inherit;
  }

  .result-meta button:hover,
  .empty button:hover {
    color: var(--accent-2);
  }

  .results {
    min-height: 126px;
    overflow-y: auto;
    overscroll-behavior: contain;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    scrollbar-color: var(--border-strong) transparent;
  }

  .results a {
    display: grid;
    grid-template-columns: 88px minmax(0, 1fr) 20px;
    align-items: center;
    gap: 16px;
    min-height: 68px;
    padding: 11px 20px;
    border-bottom: 1px solid var(--border);
    color: var(--foreground);
    text-decoration: none;
  }

  .results a:last-child {
    border-bottom: 0;
  }

  .results a:hover,
  .results a:focus,
  .results a.active {
    background: var(--hover);
  }

  .results a.active .result-arrow,
  .results a:hover .result-arrow {
    color: var(--accent);
    transform: translate(2px, -2px);
  }

  .result-category {
    color: var(--ink-3);
    font: 500 10px/1.4 var(--font-mono);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .result-copy {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .result-copy strong {
    color: var(--foreground);
    font-size: 16px;
    font-weight: 500;
    line-height: 1.25;
  }

  .result-copy small {
    margin-top: 2px;
    overflow: hidden;
    color: var(--ink-3);
    font-size: 12px;
    line-height: 1.35;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-arrow {
    justify-self: end;
    color: var(--ink-3);
    font-size: 13px;
    transition: color 140ms var(--ease-inout), transform 140ms var(--ease-inout);
  }

  .empty {
    display: grid;
    place-items: center;
    min-height: 190px;
    padding: 32px;
    text-align: center;
  }

  .empty-mark {
    display: grid;
    width: 40px;
    height: 40px;
    place-items: center;
    border: 1px dashed var(--border-strong);
    border-radius: var(--r-frame-1);
    color: var(--ink-3);
    font: 500 16px/1 var(--font-mono);
  }

  .empty p {
    margin: 8px 0 2px;
    color: var(--foreground);
  }

  .empty button {
    font: 500 11px/1.5 var(--font-mono);
    text-transform: uppercase;
  }

  .deck-footer {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    padding: 11px 20px 13px;
    color: var(--ink-3);
    font: 500 10px/1.3 var(--font-mono);
  }

  .deck-footer span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .deck-footer kbd {
    min-width: 18px;
    padding: 3px 4px;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--ink-2);
    text-align: center;
  }

  @keyframes deck-arrive {
    from { opacity: 0; transform: translateY(10px) scale(0.985); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  @media (max-width: 560px) {
    .deck-trigger kbd {
      display: none;
    }

    .deck {
      width: calc(100vw - 20px);
      max-height: calc(100dvh - 20px);
      margin-top: 10px;
    }

    .deck-header {
      padding: 20px 18px 16px;
    }

    .search-shell {
      margin: 0 10px;
    }

    .results a {
      grid-template-columns: minmax(0, 1fr) 20px;
      gap: 10px;
      padding-inline: 16px;
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
