<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { browser } from "$app/environment";
  import { paperImageUrl } from "../../../../stores/paperStore";

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

  let error = { isError: false, message: "" };
  let loading: boolean = true;
  let paper: Paper | undefined;
  let paperImage: string | null;

  paperImageUrl.subscribe((value) => {
    paperImage = value;
  });

  const slug = $page.params.slug;
  if (!slug) {
    error.isError = true;
    error.message = "slug not found";
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

<svelte:head>
  <title>Papers with Code</title>
</svelte:head>

<main
  class="min-h-screen text-gray-100 space-grotesk-400 px-4 sm:px-8 py-8"
>
  {#if loading}
    <p class="text-center text-lg">Loading...</p>
  {:else if error.isError}
    <p class="text-center text-red-500 font-bold mt-4">{error.message}</p>
  {:else if paper}
    <h1 class="text-2xl font-bold mb-4">
      {paper.title}
      <span class="text-sm text-gray-400 mx-2">•</span>
      <a on:click={goBack} class="text-base text-blue-300 hover:underline cursor-pointer"
        >back</a
      >
    </h1>

    {#if paperImage}
      <img
        src={paperImage}
        alt={paper.title}
        class="block mx-auto my-4 rounded-lg"
      />
    {/if}

    <ul class="flex flex-wrap gap-2 mb-4">
      {#each paper.authors as author}
        <li class="list-none">
          <a
            href={author.url}
            target="_blank"
            class="text-gray-300 hover:underline"
          >
            <small class="text-xs italic">{author.name}</small>
          </a>
        </li>
      {/each}
    </ul>

    <p class="text-base text-blue-100 leading-relaxed my-4">{paper.description}</p>

    <ul class="flex flex-wrap items-center gap-2">
      <li>
        <a
          href={paper.dataset_url}
          target="_blank"
          class="text-blue-300 hover:underline"
        >
          dataset
        </a>
        <span> •</span>
      </li>
      <li>
        <a
          href={paper.arxiv_page_url}
          target="_blank"
          class="text-blue-300 hover:underline"
        >
          ArXiv page
        </a>
        <span> •</span>
      </li>
      <li>
        <a
          href={paper.arxiv_pdf_url}
          target="_blank"
          class="text-blue-300 hover:underline"
        >
          pdf
        </a>
      </li>
      <li class="ml-auto">
        <a
          href="https://example.com"
          target="_blank"
          class="text-blue-200 hover:underline"
        >
          teach me
        </a>
      </li>
    </ul>
  {:else}
    <p class="text-center">No paper found.</p>
  {/if}
</main>
