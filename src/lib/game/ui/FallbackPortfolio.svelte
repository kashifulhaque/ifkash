<script lang="ts">
  import { dev } from '$app/environment';
  import { sections, name, tagline, resumePath, resumeDevUrl } from '$lib/content';

  export let showGameLink = false;

  function linkUrl(url: string): string {
    if (url === resumePath && dev) return resumeDevUrl;
    return url;
  }
</script>

<div class="fallback">
  <header>
    <h1>{name}</h1>
    <p class="tagline">{tagline}</p>
    {#if showGameLink}
      <button class="game-link" on:click>↩ back to the game</button>
    {/if}
  </header>

  {#each sections as section}
    <section>
      <h2>{section.label}</h2>
      <ul>
        {#each section.items as item}
          <li>
            <strong>{item.title}</strong>
            {#if item.subtitle}<span class="sub"> — {item.subtitle}</span>{/if}
            {#if item.body}<p>{item.body}</p>{/if}
            {#each item.links as link}
              <a href={linkUrl(link.url)} target="_blank" rel="noopener noreferrer">{link.label}</a>
            {/each}
          </li>
        {/each}
      </ul>
    </section>
  {/each}
</div>

<style>
  .fallback {
    max-width: 760px;
    margin: 0 auto;
    padding: 48px 24px 96px;
    color: var(--ink, #eee);
    overflow-y: auto;
    height: 100%;
  }

  h1 {
    font-family: var(--font-display, monospace);
    font-size: 3rem;
    text-transform: uppercase;
    color: var(--blueprint, #ffd23f);
  }

  .tagline {
    font-family: var(--font-body, serif);
    margin-top: 12px;
    line-height: 1.55;
  }

  .game-link {
    margin-top: 16px;
    background: none;
    border: 1px solid currentColor;
    color: inherit;
    font-family: var(--font-mono, monospace);
    font-size: 0.75rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    padding: 6px 14px;
    cursor: pointer;
  }

  section {
    margin-top: 40px;
  }

  h2 {
    font-family: var(--font-display, monospace);
    font-size: 1.6rem;
    text-transform: uppercase;
    border-bottom: 1px solid var(--rule-soft, #444);
    padding-bottom: 6px;
  }

  ul {
    list-style: none;
    padding: 0;
  }

  li {
    padding: 14px 0;
    border-bottom: 1px solid var(--rule-soft, #333);
  }

  strong {
    font-family: var(--font-display, monospace);
    font-size: 1.1rem;
    letter-spacing: 0.02em;
  }

  .sub {
    font-family: var(--font-mono, monospace);
    font-size: 0.75rem;
    color: var(--ink-mute, #999);
  }

  li p {
    font-family: var(--font-body, serif);
    font-size: 0.92rem;
    line-height: 1.5;
    margin: 6px 0;
    color: var(--ink-soft, #ccc);
  }

  li a {
    font-family: var(--font-mono, monospace);
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-right: 14px;
    color: var(--blueprint, #ffd23f);
  }
</style>
