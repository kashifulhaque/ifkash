<svelte:head>
  <title>Token Lens — Tools</title>
  <meta
    name="description"
    content="Watch a language model choose its next token, live, with its real probabilities. Runs on-device in your browser."
  />
</svelte:head>

<script lang="ts">
  import { splitReveal } from '$lib/motion';
  import TokenLens from '$lib/components/TokenLens.svelte';
</script>

<div class="lens-page">
  <header class="page-head">
    <p class="eyebrow">04 / Tools / Token Lens</p>
    <h1 class="page-title" use:splitReveal>Token Lens</h1>
    <p class="lead">
      Watch a small language model pick its next token, one step at a time, with the probabilities it
      really computed.
    </p>
  </header>

  <div class="body">
    <TokenLens />
  </div>
</div>

<style>
  .lens-page {
    max-width: calc(var(--content-max) + 2 * var(--gutter));
    margin: 0 auto;
    padding: 0 var(--gutter) var(--s-9);
  }
  .body {
    padding-top: var(--s-7);
  }
</style>
