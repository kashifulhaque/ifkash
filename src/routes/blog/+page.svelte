<script lang="ts">
  import { onMount } from "svelte";

  interface BlogPost {
    id: string;
    title: string;
    brief: string;
    url: string;
    coverImage?: { url: string }; // Optional as it might not always be present
    publishedAt: string; // This is a string date
  }

  let posts: BlogPost[] = [];
  let loading = true;
  let error = "";
  const options: Intl.DateTimeFormatOptions = {
    month: "long",
    day: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: true,
    timeZone: "UTC",
  };

  const query = `
    query {
      publication(host: "blog.ifkash.dev") {
        posts(first: 10) {
          edges {
            node {
              id
              title
              brief
              url
              coverImage { url }
              publishedAt
            }
          }
        }
      }
    }
  `;

  onMount(async () => {
    try {
      const res = await fetch("https://gql.hashnode.com/", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query }),
      });
      const result = await res.json();
      // Assuming result.data.publication.posts.edges is an array of { node: BlogPost }
      posts = result.data.publication.posts.edges.map(
        (edge: { node: BlogPost }) => edge.node,
      );
      console.log(posts);
    } catch (e) {
      if (e instanceof Error) {
        error = e.message;
      } else {
        error = "An unknown error occurred while fetching blog posts.";
      }
    } finally {
      loading = false;
    }
  });
</script>

<svelte:head>
  <title>Blog • Portfolio</title>
</svelte:head>

<div class="min-h-screen bg-neutral-900 px-4 py-8">
  <div class="mx-auto max-w-3xl">
    <!-- Header -->
    <section class="mb-8">
      <h2 class="flex items-center gap-2 text-2xl font-bold text-gray-100">
        <i class="fa-solid fa-blog"></i>
        Blog
        <span class="text-sm text-gray-400">•</span>
        <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
      </h2>
    </section>

    <!-- Blog Posts -->
    <section class="space-y-6">
      {#if loading}
        <div class="p-4 text-gray-300">Loading posts...</div>
      {:else if error}
        <div class="p-4 text-red-400">{error}</div>
      {:else if posts.length === 0}
        <div class="p-4 text-gray-300">No blog posts found.</div>
      {:else}
        {#each posts as post}
          <div
            class="flex flex-col overflow-hidden rounded-lg bg-neutral-800 shadow md:grid md:grid-cols-4 md:gap-4"
          >
            <div class="flex flex-col justify-between p-4 md:col-span-3">
              <a
                href={`${post.url}`}
                target="_blank"
                rel="noopener noreferrer"
                class="mb-2 block text-xl font-bold text-blue-300 hover:underline"
              >
                {post.title}
              </a>
              <p class="mb-2 line-clamp-3 text-base text-gray-300">
                {post.brief}
              </p>
              <p class="mt-auto text-xs text-gray-500">
                {new Date(post.publishedAt)
                  .toLocaleString("en-US", options)
                  .replace(",", "")
                  .replace(", ", " at ")
                  .replace(" ", " ")
                  .replace(" PM", "PM")
                  .replace(" AM", "AM")}
              </p>
            </div>
          </div>
        {/each}
      {/if}
    </section>
  </div>
</div>
