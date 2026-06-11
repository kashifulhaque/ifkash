<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { dev } from '$app/environment';
  import type { Section } from '$lib/content';
  import { resumePath, resumeDevUrl } from '$lib/content';

  export let section: Section;

  const dispatch = createEventDispatcher();

  function linkUrl(url: string): string {
    if (url === resumePath && dev) return resumeDevUrl;
    return url;
  }

  function accentHex(color: number): string {
    return '#' + color.toString(16).padStart(6, '0');
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') dispatch('close');
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="overlay" role="dialog" aria-label="{section.label} loot">
  <button class="backdrop" on:click={() => dispatch('close')} aria-label="Close"></button>
  <div class="panel" style="--accent: {accentHex(section.color)}">
    <header class="panel-header">
      <span class="panel-tag">LOOT ACQUIRED</span>
      <h2 class="panel-title">{section.label}</h2>
      <button class="close-btn" on:click={() => dispatch('close')} aria-label="Close">×</button>
    </header>

    <div class="items">
      {#each section.items as item}
        <article class="item">
          <div class="item-head">
            <h3 class="item-title">{item.title}</h3>
            {#if item.badge}
              <span class="badge">{item.badge}</span>
            {/if}
          </div>
          {#if item.subtitle}
            <p class="item-subtitle">{item.subtitle}</p>
          {/if}
          {#if item.body}
            <p class="item-body">{item.body}</p>
          {/if}
          {#if item.links.length}
            <div class="item-links">
              {#each item.links as link}
                <a href={linkUrl(link.url)} target="_blank" rel="noopener noreferrer">
                  {link.label} ↗
                </a>
              {/each}
            </div>
          {/if}
        </article>
      {/each}
    </div>

    <footer class="panel-footer">ESC OR TAP OUTSIDE TO CLOSE</footer>
  </div>
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .backdrop {
    position: absolute;
    inset: 0;
    background: rgba(10, 10, 14, 0.6);
    border: none;
    cursor: pointer;
    backdrop-filter: blur(3px);
  }

  .panel {
    position: relative;
    width: min(680px, 100%);
    max-height: min(80vh, 80dvh);
    display: flex;
    flex-direction: column;
    background: rgba(18, 16, 22, 0.96);
    border: 2px solid var(--accent);
    box-shadow: 0 0 40px rgba(0, 0, 0, 0.6), 0 0 24px color-mix(in srgb, var(--accent) 35%, transparent);
    animation: pop 0.18s ease-out;
  }

  @keyframes pop {
    from { transform: scale(0.94); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }

  .panel-header {
    display: flex;
    align-items: baseline;
    gap: 16px;
    padding: 18px 22px 12px;
    border-bottom: 1px solid color-mix(in srgb, var(--accent) 45%, transparent);
  }

  .panel-tag {
    font-family: var(--font-mono, monospace);
    font-size: 0.65rem;
    letter-spacing: 0.18em;
    color: var(--accent);
    text-transform: uppercase;
  }

  .panel-title {
    font-family: var(--font-display, monospace);
    font-size: 2rem;
    letter-spacing: 0.04em;
    color: #fff;
    margin-right: auto;
  }

  .close-btn {
    background: none;
    border: 1px solid rgba(255, 255, 255, 0.3);
    color: #fff;
    font-size: 1.3rem;
    width: 34px;
    height: 34px;
    line-height: 1;
    cursor: pointer;
  }

  .close-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .items {
    overflow-y: auto;
    padding: 8px 22px;
  }

  .item {
    padding: 14px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .item:last-child {
    border-bottom: none;
  }

  .item-head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-wrap: wrap;
  }

  .item-title {
    font-family: var(--font-display, monospace);
    font-size: 1.3rem;
    letter-spacing: 0.03em;
    color: #fff;
  }

  .badge {
    font-family: var(--font-mono, monospace);
    font-size: 0.6rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    padding: 2px 8px;
    border: 1px solid var(--accent);
    color: var(--accent);
  }

  .item-subtitle {
    font-family: var(--font-mono, monospace);
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    color: rgba(255, 255, 255, 0.55);
    margin-top: 4px;
    text-transform: uppercase;
  }

  .item-body {
    font-family: var(--font-body, serif);
    font-size: 0.92rem;
    line-height: 1.55;
    color: rgba(255, 255, 255, 0.8);
    margin-top: 6px;
  }

  .item-links {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-top: 10px;
  }

  .item-links a {
    font-family: var(--font-mono, monospace);
    font-size: 0.68rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    padding: 5px 12px;
    border: 1px solid rgba(255, 255, 255, 0.25);
    color: rgba(255, 255, 255, 0.85);
    text-decoration: none;
    transition: border-color 0.15s, color 0.15s;
  }

  .item-links a:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .panel-footer {
    padding: 10px 22px 14px;
    font-family: var(--font-mono, monospace);
    font-size: 0.6rem;
    letter-spacing: 0.16em;
    color: rgba(255, 255, 255, 0.35);
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  @media (max-width: 600px) {
    .overlay { padding: 12px; }
    .panel-title { font-size: 1.5rem; }
  }
</style>
