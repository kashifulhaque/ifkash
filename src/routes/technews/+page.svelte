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
  let error = null;

  onMount(async () => {
    try {
      const { data } = await axios.get<Story[]>("/hn");
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
  <title>Tech News | Top 15 Hacker News Stories</title>
</svelte:head>

<main>
  <h1>Top 15 Hacker News Stories of the Week</h1>

  {#if isLoading}
    <p>Loading stories...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <ol>
      {#each stories as story (story.id)}
        <li>
          <article>
            <h2>
              <a href={story.url} target="_blank" rel="noopener noreferrer">
                {story.title}
              </a>
            </h2>
            <p>
              {story.score} points by {story.by} |
              <a
                href={`https://news.ycombinator.com/item?id=${story.id}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                HN Discussion
              </a>
              |
              {formatDate(story.time)}
            </p>
          </article>
        </li>
      {/each}
    </ol>
  {/if}
</main>

<style>
  main {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }

  h1 {
    color: #ff6600;
    margin-bottom: 20px;
  }

  ol {
    list-style-type: decimal;
    padding-left: 20px;
  }

  li {
    margin-bottom: 20px;
  }

  h2 {
    font-size: 1.2em;
    margin-bottom: 5px;
  }

  a {
    color: #2e2e2e;
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }

  p {
    font-size: 0.9em;
    color: #828282;
  }

  .error {
    color: red;
  }
</style>
