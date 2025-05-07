<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import Markdown from "svelte-markdown";
  import remarkMath from "remark-math";
  import rehypeKatex from "rehype-katex";

  let post: any = null;
  let loading = true;
  let error = "";

  async function fetchPost(slug: string) {
    const query = `
      query PostBySlug($slug: String!) {
        publication(host: \"blog.ifkash.dev\") {
          post(slug: $slug) {
            title
            content { markdown }
            brief
            coverImage { url }
            publishedAt
            slug
          }
        }
      }
    `;
    const res = await fetch("https://gql.hashnode.com/", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query, variables: { slug } }),
    });
    const { data, errors } = await res.json();
    if (errors) throw new Error(errors.map((e) => e.message).join(", "));
    return data.publication.post;
  }

  onMount(async () => {
    const slug = get(page).params.slug;
    try {
      post = await fetchPost(slug);
    } catch {
      error = "Failed to load post.";
    } finally {
      loading = false;
    }
  });
</script>

<svelte:head>
  {#if post}
    <title>{post.title} • Portfolio</title>
  {:else}
    <title>Loading… • Portfolio</title>
  {/if}
  <!-- Load KaTeX CSS from CDN -->
  <link
    rel="stylesheet"
    href="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.css"
    integrity="sha384-MlJ6fQHG8t0p+7mXzM8DlaS3xj6XK2+2G6ZV5y5gvpZCxf5w+6mT6vVQ7LZnPdrG"
    crossorigin="anonymous"
  />
</svelte:head>

<div class="min-h-screen bg-neutral-900 py-8 px-4">
  <div class="max-w-3xl mx-auto prose space-y-6">
    {#if loading}
      <p>Loading post...</p>
    {:else if error}
      <p class="text-red-400">{error}</p>
    {:else}
      <div class="flex justify-between items-center">
        <h1 class="text-3xl font-bold text-gray-100">{post.title}</h1>
        <a
          href={`https://blog.ifkash.dev/${post.slug}`}
          target="_blank"
          rel="noopener noreferrer"
          class="px-4 py-2 bg-blue-500 rounded-lg text-white hover:bg-blue-600"
        >
          Read on Hashnode
        </a>
      </div>
      {#if post.coverImage?.url}
        <!-- svelte-ignore a11y-img-redundant-alt -->
        <img
          src={post.coverImage.url}
          alt="Cover image"
          class="rounded-lg w-full"
        />
      {/if}
      <!-- render markdown with math support -->
      <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]} source={post.content.markdown} />
    {/if}
  </div>
</div>

<style>
  .prose {
    color: #e5e7eb;
  }
  /* KaTeX overrides if needed */
  .katex {
    font-size: 1em;
  }
</style>
