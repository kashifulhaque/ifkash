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
  class="max-w-3xl mx-auto p-5 bg-neutral-900 text-gray-100 space-grotesk-400"
>
  <h1 class="text-2xl font-bold text-orange-400 mb-5">
    Latest Engineering Blog Posts
    <span class="text-sm text-gray-400 mx-2">•</span>
    <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
  </h1>

  {#if isLoading}
    <p class="text-lg text-center">Loading blog posts...</p>
  {:else if error}
    <p class="text-center text-red-500 font-bold mt-4">{error}</p>
  {:else}
    <div class="mb-6">
      <div class="flex gap-2 flex-wrap">
        {#each blogs as blog}
          <button
            class="px-3 py-1 rounded-full text-sm cursor-pointer {selectedBlog ===
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
          class="border border-neutral-700 rounded-lg p-4 hover:bg-neutral-800 transition-colors"
        >
          <h2 class="text-lg mb-1 text-blue-200">
            <a
              href={post.link}
              target="_blank"
              rel="noopener noreferrer"
              class="hover:underline"
            >
              {post.title}
            </a>
          </h2>
          <div class="flex items-center text-sm text-gray-200 mt-2">
            <span
              class="bg-neutral-700 px-2 py-0.5 rounded text-xs font-semibold"
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
        <p class="text-center text-gray-400 py-10">
          No posts found for the selected blog.
        </p>
      {/if}
    </div>
  {/if}
</main>
