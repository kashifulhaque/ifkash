<script lang="ts">
  import { onMount } from "svelte";
  import { paperImageUrl } from "../../stores/paperStore";

  interface Tag {
    name: string;
    url: string;
  }

  interface Paper {
    stars: string;
    slug: string;
    paper_url: string;
    code_url: string;
    image_url: string;
    title: string;
    github_repo: string;
    framework: string;
    publish_date: string;
    description: string;
    tags: Tag[] | null;
  }

  let papers: Paper[] = [];
  let loading: boolean = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      const response = await fetch("/api/pwc");
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();

      if (!data || !Array.isArray(data)) {
        throw new Error("Invalid data received from API");
      }

      papers = data.filter((paper) => paper && paper.title && paper.paper_url);
    } catch (err) {
      console.error("Error fetching papers:", err);
      if (err instanceof Error) {
        error = err.message;
      } else {
        error = "An unknown error occurred while fetching papers.";
      }
    } finally {
      loading = false;
    }
  });

  const handleClick = (paper: string) => {
    paperImageUrl.set(paper);
  };
</script>

<svelte:head>
  <title>Papers with Code</title>
</svelte:head>

<main
  class="space-grotesk-400 min-h-screen bg-neutral-900 px-4 py-8 text-gray-100 sm:px-8"
>
  <h1 class="mb-8 flex items-center gap-2 text-2xl font-bold">
    <i class="fa-solid fa-file-contract"></i>
    Papers with Code
    <span class="text-sm text-gray-400">•</span>
    <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
  </h1>

  {#if loading}
    <p class="text-center">Loading papers...</p>
  {:else if error}
    <p class="mt-4 text-center font-bold text-red-500">{error}</p>
  {:else}
    <div class="grid grid-cols-1 gap-8 sm:grid-cols-2 lg:grid-cols-3">
      {#each papers as paper}
        <div
          class="overflow-hidden rounded-lg bg-gray-800 transition-transform duration-300 ease-in-out hover:scale-105"
        >
          <a
            href={"/papers" + paper.slug}
            on:click={() => handleClick(paper.image_url)}
          >
            <img
              src={paper.image_url}
              alt={paper.title}
              class="h-48 w-full object-cover"
            />
          </a>
          <div class="p-4">
            <h2 class="mb-2 text-lg font-bold">
              <a
                href={"/papers" + paper.slug}
                on:click={() => handleClick(paper.image_url)}
                class="text-gray-100 hover:underline"
              >
                {paper.title}
              </a>
            </h2>
            <div class="mb-2">
              <small class="text-sm italic opacity-70"
                >{paper.description}</small
              >
            </div>
            <div class="mb-3 flex">
              {#if paper.github_repo}
                <a
                  href={paper.github_repo}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="flex items-center gap-1 text-base text-blue-500 hover:underline"
                >
                  <i class="fa-brands fa-github"></i> code
                </a>
              {/if}
            </div>
            {#if paper.tags && paper.tags.length > 0}
              <div class="mt-2 flex flex-wrap gap-2">
                {#each paper.tags as tag}
                  {#if tag.name.length !== 0}
                    <a
                      href={tag.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="text-xs text-gray-300 lowercase hover:underline"
                    >
                      {tag.name}
                    </a>
                  {/if}
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</main>
