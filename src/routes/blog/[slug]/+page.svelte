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

<div class="min-h-screen py-8 px-4" style="background-color: var(--color-background);">
  <div class="max-w-3xl mx-auto prose space-y-6" style="color: var(--color-paragraph);">
    {#if loading}
      <p style="color: var(--color-paragraph);">Loading post...</p>
    {:else if error}
      <p style="color: #fca5a5;">{error}</p>
    {:else}
      <div class="flex justify-between items-center">
        <h1 class="text-3xl font-bold" style="color: var(--color-headline);">{post.title}</h1>
        <a
          href={`https://blog.ifkash.dev/${post.slug}`}
          target="_blank"
          rel="noopener noreferrer"
          class="px-4 py-2 rounded-lg transition-colors" style="background-color: var(--color-button); color: var(--color-button-text);" onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.8)'" onmouseout="this.style.backgroundColor='var(--color-button)'"
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
    color: var(--color-paragraph);
  }
  .prose h1, .prose h2, .prose h3, .prose h4, .prose h5, .prose h6 {
    color: var(--color-headline);
  }
  .prose a {
    color: var(--color-highlight);
  }
  .prose a:hover {
    color: var(--color-button);
  }
  .prose code {
    background-color: rgba(114, 117, 126, 0.1);
    color: var(--color-paragraph);
  }
  .prose pre {
    background-color: rgba(114, 117, 126, 0.1);
  }
  .prose blockquote {
    border-color: var(--color-secondary);
    color: var(--color-paragraph);
  }
  /* KaTeX overrides if needed */
  .katex {
    font-size: 1em;
  }
</style>
