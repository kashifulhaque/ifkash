<script lang="ts">
  import axios from "axios";
  import { onMount } from "svelte";

  interface Story {
    by: string;
    id: number;
    score: number;
    time: number;
    title: string;
    type: string;
    url: string;
  }

  let stories: Story[] = [];
  let isLoading = true;
  let error: string = "";

  onMount(async () => {
    try {
      const { data } = await axios.get<Story[]>("/api/hn");
      stories = data;
    } catch (err) {
      console.error(err);
      error = "Failed to fetch Hacker News stories. Please try again later.";
    } finally {
      isLoading = false;
    }
  });

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }
</script>

<svelte:head>
  <title>Tech News | Top {stories.length} Hacker News Stories</title>
</svelte:head>

<main
  class="max-w-3xl mx-auto p-5 bg-neutral-900 text-gray-100 space-grotesk-400"
>
  <h1 class="text-2xl font-bold text-orange-400 mb-5">
    Top {stories.length} Hacker News Stories of the Week
    <span class="text-sm text-gray-400 mx-2">•</span>
    <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
  </h1>

  {#if isLoading}
    <p class="text-lg text-center">Loading stories...</p>
  {:else if error}
    <p class="text-center text-red-500 font-bold mt-4">{error}</p>
  {:else}
    <ol class="list-decimal pl-5">
      {#each stories as story (story.id)}
        <li class="mb-5">
          <article>
            <h2 class="text-lg mb-1 text-blue-200">
              <a
                href={story.url}
                target="_blank"
                rel="noopener noreferrer"
                class="hover:underline"
              >
                {story.title}
              </a>
            </h2>
            <p class="text-sm text-gray-200">
              {story.score} points by {story.by} |
              <a
                href={`https://news.ycombinator.com/item?id=${story.id}`}
                target="_blank"
                rel="noopener noreferrer"
                class="hover:underline text-blue-300"
              >
                HN Discussion
              </a>
              | {formatDate(story.time)}
            </p>
          </article>
        </li>
      {/each}
    </ol>
  {/if}
</main>
