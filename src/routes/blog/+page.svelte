<script lang="ts">
  import { onMount } from "svelte";
  let posts = [];
  let loading = true;
  let error = "";
  const options = {
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
      posts = result.data.publication.posts.edges.map((edge: any) => edge.node);
      console.log(posts);
    } catch (e) {
      error = "Failed to load blog posts.";
    } finally {
      loading = false;
    }
  });
</script>

<svelte:head>
  <title>Blog • Portfolio</title>
</svelte:head>

<div class="min-h-screen bg-neutral-900 py-8 px-4">
  <div class="max-w-3xl mx-auto">
    <!-- Header -->
    <section class="mb-8">
      <h2 class="text-2xl font-bold flex items-center gap-2 text-gray-100">
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
            class="bg-neutral-800 rounded-lg shadow flex flex-col md:grid md:grid-cols-4 md:gap-4 overflow-hidden"
          >
            <div class="md:col-span-3 flex flex-col justify-between p-4">
              <a
                href={`${post.url}`}
                target="_blank"
                rel="noopener noreferrer"
                class="text-xl font-bold text-blue-300 hover:underline block mb-2"
              >
                {post.title}
              </a>
              <p class="text-gray-300 text-base mb-2 line-clamp-3">
                {post.brief}
              </p>
              <p class="text-xs text-gray-500 mt-auto">
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
