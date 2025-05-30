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
  class="space-grotesk-400 min-h-screen bg-neutral-900 px-4 py-8 text-gray-100 sm:px-8"
>
  {#if loading}
    <p class="text-center text-lg">Loading...</p>
  {:else if error.isError}
    <p class="mt-4 text-center font-bold text-red-500">{error.message}</p>
  {:else if paper}
    <h1 class="mb-4 text-2xl font-bold">
      {paper.title}
      <span class="mx-2 text-sm text-gray-400">•</span>
      <button
        type="button"
        on:click={goBack}
        class="cursor-pointer text-base text-blue-300 hover:underline"
        aria-label="Go back to previous page"
      >
        back
      </button>
    </h1>

    {#if paperImage}
      <img
        src={paperImage}
        alt={paper.title}
        class="mx-auto my-4 block rounded-lg"
      />
    {/if}

    <ul class="mb-4 flex flex-wrap gap-2">
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

    <p class="my-4 text-base leading-relaxed text-blue-100">
      {paper.description}
    </p>

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
