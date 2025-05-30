<script lang="ts">
  import axios from "axios";
  import { onMount } from "svelte";

  interface Post {
    title: string;
    link: string;
    date: string;
    author: string;
    source: string;
  }

  interface BlogsResponse {
    posts: Post[];
    blogs: string[];
  }

  let posts: Post[] = [];
  let blogs: string[] = [];
  let isLoading = true;
  let error: string = "";
  let selectedBlog: string = "All";

  onMount(async () => {
    try {
      const { data } = await axios.get<BlogsResponse>("/api/blogs");
      posts = data.posts;
      blogs = ["All", ...data.blogs];
    } catch (err) {
      console.error(err);
      error = "Failed to fetch blog posts. Please try again later.";
    } finally {
      isLoading = false;
    }
  });

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleString();
  }

  $: filteredPosts =
    selectedBlog === "All"
      ? posts
      : posts.filter((post) => post.source === selectedBlog);
</script>

<svelte:head>
  <title>Tech Blogs | Latest Engineering Posts</title>
</svelte:head>

<main
  class="space-grotesk-400 mx-auto max-w-3xl bg-neutral-900 p-5 text-gray-100"
>
  <h1 class="mb-5 text-2xl font-bold text-orange-400">
    Latest Engineering Blog Posts
    <span class="mx-2 text-sm text-gray-400">•</span>
    <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
  </h1>

  {#if isLoading}
    <p class="text-center text-lg">Loading blog posts...</p>
  {:else if error}
    <p class="mt-4 text-center font-bold text-red-500">{error}</p>
  {:else}
    <div class="mb-6">
      <div class="flex flex-wrap gap-2">
        {#each blogs as blog}
          <button
            class="cursor-pointer rounded-full px-3 py-1 text-sm {selectedBlog ===
            blog
              ? 'bg-blue-600'
              : 'bg-neutral-700 hover:bg-neutral-600'}"
            on:click={() => (selectedBlog = blog)}
          >
            {blog}
          </button>
        {/each}
      </div>
    </div>

    <div class="space-y-6">
      {#each filteredPosts as post}
        <article
          class="rounded-lg border border-neutral-700 p-4 transition-colors hover:bg-neutral-800"
        >
          <h2 class="mb-1 text-lg text-blue-200">
            <a
              href={post.link}
              target="_blank"
              rel="noopener noreferrer"
              class="hover:underline"
            >
              {post.title}
            </a>
          </h2>
          <div class="mt-2 flex items-center text-sm text-gray-200">
            <span
              class="rounded bg-neutral-700 px-2 py-0.5 text-xs font-semibold"
            >
              {post.source}
            </span>
            {#if post.author}
              <span class="ml-2">by {post.author}</span>
            {/if}
            <span class="ml-auto text-gray-400">{formatDate(post.date)}</span>
          </div>
        </article>
      {/each}

      {#if filteredPosts.length === 0}
        <p class="py-10 text-center text-gray-400">
          No posts found for the selected blog.
        </p>
      {/if}
    </div>
  {/if}
</main>
