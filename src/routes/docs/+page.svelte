<script lang="ts">
  import { onMount } from 'svelte';

  let redocLoaded = false;
  let specUrl = "/api/docs";

  /// Function to load Redoc script dynamically
  const loadRedoc = () => {
    return new Promise<void>((resolve) => {
      const script = document.createElement('script');
      script.src = "https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js";
      script.onload = () => resolve();
      document.body.appendChild(script);
    });
  };

  /// On mount, load Redoc and render the documentation
  onMount(async () => {
    await loadRedoc();
    redocLoaded = true;

    if (redocLoaded) {
      Redoc.init(specUrl, {}, document.getElementById('redoc-container'));
    }
  });
</script>

<svelte:head>
  <title>API Documentation - ifkash.dev</title>
  <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
  <meta name="description" content="API documentation for interacting with ifkash.dev services">
</svelte:head>

<style>
  #redoc-container {
    padding: 20px;
    font-family: 'Roboto', sans-serif;
  }
</style>

<div id="redoc-container">
  {#if !redocLoaded}
    <p>Loading API documentation ...</p>
  {/if}
</div>
