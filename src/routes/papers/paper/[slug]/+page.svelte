<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { browser } from "$app/environment";

  interface Author {
    name: string;
    url: string;
  }

  interface Paper {
    title: string;
    authors: Author[];
    dataset_url: string;
    arxiv_page_url: string;
    arxiv_pdf_url: string;
    description: string;
  }

  let error = {
    isError: false,
    message: "",
  };
  let loading: boolean = true;
  let paper: Paper | undefined;

  const slug = $page.params.slug;
  if (!slug) {
    (error.isError = true), (error.message = "slug not found");
  }

  onMount(async () => {
    const paperUrl = `https://paperswithcode.com/paper/${slug}`;
    try {
      const response = await fetch(`/api/pwc/paper?paper_url=${paperUrl}`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();

      if (!data) {
        throw new Error("Invalid data received from API");
      }

      paper = data;
    } catch (err) {
      error.isError = true;
      error.message = err as string;
      console.error("Error fetching papers", error);
    } finally {
      loading = false;
    }
  });

  function goBack() {
    if (browser) window.history.back();
  }
</script>

<main id="container--main">
  {#if loading}
    <p>Loading...</p>
  {:else if error.isError}
    <p>{error.message}</p>
  {:else if paper}
    <h1>{paper.title} • <a on:click={goBack}>back</a></h1>

    <ul class="authors-list">
      {#each paper.authors as author, index}
        <li style="display: inline; margin-right: 0.5em;">
          <a href={author.url} target="_blank"><small>{author.name}</small></a>
        </li>
      {/each}
    </ul>

    <p class="description">{paper.description}</p>

    <ul class="link-list">
      <li class="link-item">
        <a href={paper.dataset_url} target="_blank">dataset</a> •
      </li>
      <li class="link-item">
        <a href={paper.arxiv_page_url} target="_blank">ArXiv page</a> •
      </li>
      <li class="link-item">
        <a href={paper.arxiv_pdf_url} target="_blank">pdf</a>
      </li>
      <li class="link-item last-item">
        <a href="https://example.com" target="_blank">teach me</a>
      </li>
    </ul>
  {:else}
    <p>No paper found.</p>
  {/if}
</main>

<style>
  @import url("https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap");

  .description {
    font-family: "Inter", sans-serif;
    font-optical-sizing: auto;
    font-weight: 400;
    font-style: normal;
    font-size: 0.95rem;
    line-height: 1.75rem;
    margin: 1em 0;
  }

  ul {
    padding-left: 1.2em;
    margin: 0.5em 0;
  }

  li {
    list-style-type: none;
    margin-bottom: 0.5em;
  }

  a {
    cursor: pointer;
    text-decoration: none;
    color: var(--mainLinkColor);
    font-weight: 500;
  }

  a:hover {
    text-decoration: underline;
  }

  small {
    font-size: 0.85rem;
    font-family: "Inter", sans-serif;
    font-weight: 100;
    font-style: italic;
  }

  .authors-list {
    margin: 0;
    padding-left: 0;
    display: inline;
  }

  .link-list {
    display: flex;
    justify-content: space-between; /* Distributes space between items */
    padding-left: 0;
    margin: 0;
  }

  .link-item {
    margin-right: 0.5em;
  }

  .last-item {
    margin-left: auto; /* Pushes the last item to the right */
  }
</style>
