<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { dev } from '$app/environment';
  import { resumePath, resumeDevUrl } from '$lib/content';
  import type { Wonder } from '../wonders';
  import { biomeById } from '../biomes';

  export let wonder: Wonder;
  export let justFound = false;

  const dispatch = createEventDispatcher();

  $: biome = biomeById(wonder.biome);

  function linkUrl(url: string): string {
    if (url === resumePath && dev) return resumeDevUrl;
    return url;
  }

  function isExternal(url: string): boolean {
    return /^https?:/.test(url);
  }
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-label={wonder.title}>
  <button class="backdrop" on:click={() => dispatch('close')} aria-label="Close"></button>
  <div class="panel">
    <header>
      <p class="kicker">
        {justFound ? 'A small wonder found' : 'A small wonder'} · {biome.name}
      </p>
      <h2>{wonder.title}</h2>
      <p class="blurb">{wonder.blurb}</p>
      <button class="close" on:click={() => dispatch('close')} aria-label="Close">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
      </button>
    </header>

    <div class="items">
      {#each wonder.items as item}
        <article class="item">
          <div class="item-head">
            <h3>{item.title}</h3>
            {#if item.badge}
              <span class="badge">{item.badge}</span>
            {/if}
          </div>
          {#if item.subtitle}
            <p class="subtitle">{item.subtitle}</p>
          {/if}
          {#if item.body}
            <p class="body">{item.body}</p>
          {/if}
          {#if item.links.length}
            <div class="links">
              {#each item.links as link}
                <a
                  href={linkUrl(link.url)}
                  target={isExternal(link.url) || link.url === resumePath ? '_blank' : undefined}
                  rel={isExternal(link.url) ? 'noopener noreferrer' : undefined}
                >
                  {link.label}
                  {#if isExternal(link.url) || link.url === resumePath}<span aria-hidden="true">↗</span>{/if}
                </a>
              {/each}
            </div>
          {/if}
        </article>
      {/each}
    </div>

    <footer>
      <button class="continue" on:click={() => dispatch('close')}>Keep wandering</button>
      <span class="esc">Esc or tap outside to close</span>
    </footer>
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
    font-family: var(--planet-sans);
    color: #f2efe6;
  }
  .backdrop {
    position: absolute;
    inset: 0;
    border: none;
    background: rgba(6, 16, 24, 0.55);
    backdrop-filter: blur(4px);
  }
  .panel {
    position: relative;
    width: min(620px, 100%);
    max-height: min(82vh, 82dvh);
    display: flex;
    flex-direction: column;
    background: rgba(14, 30, 40, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 18px;
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5);
    animation: pop 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    overflow: hidden;
  }
  header {
    padding: 24px 28px 18px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    position: relative;
  }
  .kicker {
    margin: 0;
    font-size: 0.62rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: #e9c46a;
  }
  h2 {
    margin: 8px 0 6px;
    font-family: var(--planet-serif);
    font-weight: 400;
    font-size: 2.1rem;
    line-height: 1.05;
    color: #fbf8f0;
  }
  .blurb {
    margin: 0;
    font-size: 0.86rem;
    color: rgba(242, 239, 230, 0.66);
  }
  .close {
    position: absolute;
    top: 18px;
    right: 18px;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: transparent;
    color: rgba(242, 239, 230, 0.8);
    display: grid;
    place-items: center;
  }
  .close:hover {
    border-color: rgba(255, 255, 255, 0.4);
  }
  .close svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
  }
  .items {
    overflow-y: auto;
    padding: 6px 28px;
  }
  .item {
    padding: 16px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  }
  .item:last-child {
    border-bottom: none;
  }
  .item-head {
    display: flex;
    align-items: baseline;
    gap: 10px;
    flex-wrap: wrap;
  }
  h3 {
    margin: 0;
    font-family: var(--planet-serif);
    font-weight: 400;
    font-size: 1.25rem;
    color: #fbf8f0;
  }
  .badge {
    font-size: 0.58rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    padding: 3px 8px;
    border-radius: 999px;
    border: 1px solid rgba(233, 196, 106, 0.5);
    color: #e9c46a;
  }
  .subtitle {
    margin: 4px 0 0;
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: rgba(242, 239, 230, 0.5);
  }
  .body {
    margin: 8px 0 0;
    font-size: 0.9rem;
    line-height: 1.55;
    color: rgba(242, 239, 230, 0.82);
  }
  .links {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 12px;
  }
  .links a {
    font-size: 0.72rem;
    padding: 6px 12px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.18);
    color: rgba(242, 239, 230, 0.9);
    text-decoration: none;
    transition: border-color 0.15s, background 0.15s;
  }
  .links a:hover {
    border-color: rgba(233, 196, 106, 0.7);
    background: rgba(233, 196, 106, 0.08);
  }
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 28px 18px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }
  .continue {
    padding: 10px 18px;
    border-radius: 999px;
    border: 1px solid rgba(233, 196, 106, 0.6);
    background: rgba(233, 196, 106, 0.12);
    color: #fbf1d3;
    font-size: 0.8rem;
    font-weight: 500;
  }
  .continue:hover {
    background: rgba(233, 196, 106, 0.22);
  }
  .esc {
    font-size: 0.62rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: rgba(242, 239, 230, 0.4);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(14px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
  @media (max-width: 600px) {
    .overlay {
      padding: 12px;
    }
    header,
    .items,
    footer {
      padding-left: 18px;
      padding-right: 18px;
    }
    h2 {
      font-size: 1.6rem;
    }
    .esc {
      display: none;
    }
  }
</style>
