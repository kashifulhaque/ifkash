<script lang="ts">
  import { page } from '$app/stores';
  import OldGame from './OldGame.svelte';
  import LatticefallGame from './LatticefallGame.svelte';

  // `/game?engine=lf` shows the Latticefall engine; plain `/game` keeps the tiny-planet
  // game until WP-086 makes Latticefall the default.
  $: useLatticefall = $page.url.searchParams.get('engine') === 'lf';
  $: showDev = $page.url.searchParams.get('dev') === '1';
</script>

{#if useLatticefall}
  <LatticefallGame {showDev} />
{:else}
  <OldGame />
{/if}
