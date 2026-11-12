<script lang="ts">
  import { page } from '$app/stores';
  import { name } from '$lib/content';
  import LandingCard from '$lib/latticefall/ui/portfolio/LandingCard.svelte';
  import { skipsLanding } from '$lib/latticefall/ui/portfolio/landing';

  // `/game?engine=lf` shows the Latticefall engine; plain `/game` keeps the tiny-planet
  // game until WP-086 makes Latticefall the default.
  $: useLatticefall = $page.url.searchParams.get('engine') === 'lf';
  $: showDev = $page.url.searchParams.get('dev') === '1';

  // `/game?engine=lf` first shows the landing card, as plain DOM. Neither game's code
  // (Three.js, the WASM core) is fetched until the visitor presses Play; `&dev=1` and
  // `&play=1` skip the card. Plain `/game` skips it too, so the live site keeps opening the
  // tiny-planet game directly until WP-086.
  let started =
    $page.url.searchParams.get('engine') !== 'lf' || skipsLanding($page.url.searchParams);

  const loadLatticefall = () => import('./LatticefallGame.svelte');
  const loadOld = () => import('./OldGame.svelte');
</script>

<svelte:head>
  {#if !started}
    <title>{name}: ML engineer. Portfolio, and a small action RPG.</title>
    <meta
      name="description"
      content="{name} is an ML engineer. This page holds the portfolio, work, projects, résumé and contact, and a small action RPG."
    />
  {/if}
</svelte:head>

{#if !started}
  <LandingCard on:play={() => (started = true)} />
{:else if useLatticefall}
  {#await loadLatticefall() then mod}
    <svelte:component this={mod.default} {showDev} />
  {/await}
{:else}
  {#await loadOld() then mod}
    <svelte:component this={mod.default} />
  {/await}
{/if}
