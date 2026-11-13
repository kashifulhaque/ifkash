<script lang="ts">
  // The phone codex view (docs/game/07-presentation.md, "Mobile"): what `/game` opens on a
  // phone-sized screen. The plain portfolio through the codex pages, a short trailer clip, and
  // a button for touch mode. No Three.js or WASM loads here.
  import { createEventDispatcher, onMount } from 'svelte';
  import { name, tagline } from '$lib/content';
  import { GROUPS, entriesIn, entryHref } from '../codex/codexPages';
  import { TRAILER_SOURCES, trailerPresent } from './trailer';

  const dispatch = createEventDispatcher<{ touch: void }>();
  const groups = GROUPS.map((g) => ({ ...g, entries: entriesIn(g.section) }));

  // The trailer file is not in the repository until it is recorded, so the page asks whether it
  // exists and shows the poster alone when it does not.
  let hasClip = false;
  let playing = false;
  let video: HTMLVideoElement;

  onMount(async () => {
    hasClip = await trailerPresent();
  });

  function playClip(): void {
    playing = true;
    void Promise.resolve().then(() => video?.play().catch(() => (playing = false)));
  }
</script>

<main class="phone" aria-labelledby="phone-name">
  <p class="eyebrow">Portfolio · and a small action RPG</p>
  <h1 id="phone-name">{name}</h1>
  <p class="role">{tagline}</p>

  <div class="trailer" class:playing>
    {#if hasClip}
      <video bind:this={video} playsinline muted loop controls={playing} preload="none" aria-label="Trailer for Cinderheart">
        {#each TRAILER_SOURCES as s}<source src={s.src} type={s.type} />{/each}
      </video>
    {/if}
    {#if !playing}
      <div class="poster">
        <span class="poster-mark" aria-hidden="true">&#9670;</span>
        <span class="poster-title">Cinderheart</span>
        <span class="poster-sub">A small action RPG</span>
        {#if hasClip}
          <button type="button" class="play" on:click={playClip}>Play trailer</button>
        {:else}
          <span class="poster-note">Trailer coming soon</span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="actions">
    <button type="button" class="btn primary" on:click={() => dispatch('touch')}>Try touch mode</button>
    <a class="btn" href="/codex">Open the codex</a>
  </div>
  <p class="note">Touch mode lowers quality to keep the game smooth. A keyboard or gamepad plays best.</p>

  {#each groups as group (group.section)}
    <section aria-labelledby={`phone-${group.section}`}>
      <h2 id={`phone-${group.section}`}>{group.label}</h2>
      <ul>
        {#each group.entries as entry (entry.key)}
          <li>
            <a href={entryHref(entry)}>
              <span class="n">{entry.item.title}</span>
              {#if entry.item.subtitle}<span class="s">{entry.item.subtitle}</span>{/if}
            </a>
          </li>
        {/each}
      </ul>
    </section>
  {/each}
</main>

<style>
  .phone {
    box-sizing: border-box;
    width: 100%;
    max-width: 560px;
    margin: 0 auto;
    padding: var(--s-5) var(--gutter) var(--s-7);
    color: var(--ink);
    font-family: var(--font-sans);
  }

  .eyebrow {
    margin: 0 0 var(--s-2);
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
    margin: var(--s-2) 0 var(--s-4);
    color: var(--ink-2);
    font-size: var(--text-md);
    line-height: 1.5;
  }

  .trailer {
    position: relative;
    aspect-ratio: 16 / 9;
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .trailer video {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .poster {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-1);
    background: radial-gradient(ellipse at 50% 70%, color-mix(in srgb, var(--falu) 35%, var(--surface)), var(--surface) 70%);
    text-align: center;
  }

  .poster-mark {
    color: var(--falu);
    font-size: var(--text-xl);
  }

  .poster-title {
    font-family: var(--font-display);
    font-size: var(--text-xl);
    font-weight: 600;
  }

  .poster-sub,
  .poster-note {
    color: var(--ink-2);
    font-size: var(--text-sm);
  }

  .play {
    margin-top: var(--s-2);
    padding: var(--s-2) var(--s-4);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--ink);
    font: inherit;
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-4);
  }

  .btn {
    box-sizing: border-box;
    min-height: 44px;
    padding: var(--s-2) var(--s-4);
    display: inline-flex;
    align-items: center;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    text-decoration: none;
    cursor: pointer;
  }

  .btn.primary {
    background: var(--fjord);
    border-color: var(--fjord);
    color: var(--ink-inverse);
  }

  .note {
    margin: var(--s-3) 0 var(--s-5);
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  h2 {
    margin: var(--s-5) 0 var(--s-2);
    font-family: var(--font-display);
    font-size: var(--text-lg);
    font-weight: 600;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  li {
    border-bottom: 1px solid var(--line);
  }

  li a {
    display: flex;
    flex-direction: column;
    min-height: 44px;
    justify-content: center;
    padding: var(--s-2) 0;
    color: var(--ink);
    text-decoration: none;
  }

  .s {
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  a:focus-visible,
  button:focus-visible {
    outline: 2px solid var(--fjord);
    outline-offset: 2px;
  }
</style>
