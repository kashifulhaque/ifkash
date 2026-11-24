<script lang="ts">
  import { registry } from "$lib/designs/registry";
  import { activeDesign } from "$lib/designs/active";

  $: Home = registry[$activeDesign].Home;
</script>

<svelte:head>
  <title>Kashiful Haque</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, an ML engineer in Bangalore."
  />
</svelte:head>

<svelte:component this={Home} />
