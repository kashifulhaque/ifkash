<script lang="ts">
  // The landing card (docs/game/08-portfolio-integration.md, "Landing card"): what `/game`
  // shows first, as plain DOM with no Three.js or WASM loaded. Name, role, résumé and
  // contact are readable before anything heavy downloads.
  import { createEventDispatcher, onMount } from 'svelte';
  import { name, tagline } from '$lib/content';
  import FactsPanel from './FactsPanel.svelte';
  import { DOWNLOAD_MB, readPlainPreference, writePlainPreference } from './landing';
  import { contactItems, resumeHref } from './resume';

  const dispatch = createEventDispatcher<{ play: void }>();
  const contacts = contactItems();

  let remember = false;
  let showContact = false;
  let plainLink: HTMLAnchorElement;
  let playButton: HTMLButtonElement;

  onMount(() => {
    // A visitor who chose the plain site and ticked the box lands with that link focused.
    remember = readPlainPreference();
    (remember ? plainLink : playButton)?.focus();
  });

  function leavePlain(): void {
    writePlainPreference(remember);
  }

  function toggleRemember(): void {
    // Unticking forgets a saved choice straight away; ticking saves when they pick the plain site.
    if (!remember) writePlainPreference(false);
  }
</script>

<main class="card" aria-labelledby="landing-name">
  <p class="eyebrow">Portfolio · and a small action RPG</p>
  <h1 id="landing-name">{name}</h1>
  <p class="role">{tagline}</p>

  <div class="actions">
    <button type="button" class="btn primary" bind:this={playButton} on:click={() => dispatch('play')}>Play</button>
    <a class="btn" href={resumeHref()} target="_blank" rel="noopener noreferrer">Résumé</a>
    <button type="button" class="btn" aria-expanded={showContact} aria-controls="landing-contact" on:click={() => (showContact = !showContact)}
      >Contact</button
    >
    <a class="btn" href="/" bind:this={plainLink} on:click={leavePlain}>Read the plain site</a>
  </div>

  {#if showContact}
    <ul id="landing-contact" class="contact">
      {#each contacts as item}
        <li><FactsPanel {item} level={2} /></li>
      {/each}
    </ul>
  {/if}

  <p class="size">Downloads about {DOWNLOAD_MB} MB.</p>
  <label class="remember">
    <input type="checkbox" bind:checked={remember} on:change={toggleRemember} />
    Remember my choice
  </label>
</main>

<style>
  .card {
    box-sizing: border-box;
    width: min(640px, calc(100% - 2 * var(--gutter)));
    margin: var(--s-8) auto;
    padding: var(--s-6);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    color: var(--ink);
    font-family: var(--font-sans);
  }

  .eyebrow {
    margin: 0 0 var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    font-weight: 600;
    line-height: 1.1;
  }

  .role {
    margin: var(--s-3) 0 var(--s-5);
    color: var(--ink-2);
    font-size: var(--text-md);
    line-height: 1.5;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }

  .btn {
    padding: var(--s-2) var(--s-4);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    text-decoration: none;
    cursor: pointer;
  }

  .btn:hover {
    border-color: var(--ink-3);
  }

  .btn.primary {
    background: var(--fjord);
    border-color: var(--fjord);
    color: var(--ink-inverse);
  }

  .btn:focus-visible {
    outline: 2px solid var(--fjord);
    outline-offset: 2px;
  }

  .contact {
    display: grid;
    gap: var(--s-2);
    margin: var(--s-4) 0 0;
    padding: 0;
    list-style: none;
  }

  .size {
    margin: var(--s-5) 0 var(--s-2);
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .remember {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    color: var(--ink-2);
    font-size: var(--text-sm);
    cursor: pointer;
  }
</style>
