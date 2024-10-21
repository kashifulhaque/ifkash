<script lang="ts">
  import { onMount } from "svelte";

  let redocLoaded = false;
  let specUrl = "/api/docs";

  /// Function to load Redoc script dynamically
  const loadRedoc = () => {
    return new Promise<void>((resolve) => {
      const script = document.createElement("script");
      script.src =
        "https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js";
      script.onload = () => resolve();
      document.body.appendChild(script);
    });
  };

  /// On mount, load Redoc and render the documentation
  onMount(async () => {
    await loadRedoc();
    redocLoaded = true;

    if (redocLoaded) {
      Redoc.init(specUrl, {}, document.getElementById("redoc-container"));
    }
  });
</script>

<svelte:head>
  <title>API Documentation - ifkash.dev</title>
  <link
    href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700"
    rel="stylesheet"
  />
  <meta
    name="description"
    content="API documentation for interacting with ifkash.dev services"
  />
</svelte:head>

<div id="redoc-container">
  {#if !redocLoaded}
    <p>Loading API documentation ...</p>
  {/if}
</div>

<style>
  /* Dark theme for the sidebar */
  .menu-content,
  .api-content {
    background-color: #1a1a1a !important; /* Dark background */
    color: #ffffff !important; /* Light text color */
  }

  /* Ensure header text is readable (white on dark) */
  h1,
  h2,
  h3,
  h4,
  h5,
  h6,
  p,
  .api-info {
    color: #ffffff !important; /* Light text for headers and general content */
  }

  /* Adjust colors for response boxes and other UI elements */
  .response-col_status,
  .response-samples__content {
    background-color: #2b2b2b !important; /* Dark background for response content */
    color: #ffffff !important; /* Light text for response */
  }

  /* Fix buttons and links */
  .btn,
  a,
  .download-url {
    background-color: #3d3d3d !important; /* Dark background for buttons/links */
    color: #ffffff !important; /* Light text */
  }

  /* Adjust search bar colors */
  .search {
    background-color: #2b2b2b !important; /* Dark background for search */
    color: #ffffff !important; /* Light text */
  }

  /* Override Redoc-specific classes */
  #redoc-container .menu-content,
  #redoc-container .search {
    background-color: #2b2b2b !important; /* Consistent dark theme for search & sidebar */
  }

  #redoc-container {
    padding: 20px;
    font-family: "Roboto", sans-serif;
  }

  span {
    color: #000000;
  }
</style>
