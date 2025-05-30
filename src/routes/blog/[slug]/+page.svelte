<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import Markdown from "svelte-markdown";
  import remarkMath from "remark-math";
  import rehypeKatex from "rehype-katex";

  interface PostContent {
    markdown: string;
  }

  interface Post {
    title: string;
    content: PostContent;
    brief?: string; // brief might not be used on this page but good to have
    coverImage?: { url: string };
    publishedAt: string;
    slug: string;
  }

  interface GQLError {
    message: string;
  }

  let post: Post | null = null;
  let loading = true;
  let error = "";

  async function fetchPost(slug: string): Promise<Post> {
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
    if (errors && errors.length > 0) {
      throw new Error(errors.map((e: GQLError) => e.message).join(", "));
    }
    if (!data || !data.publication || !data.publication.post) {
      throw new Error("Post data is not in the expected format.");
    }
    return data.publication.post;
  }

  onMount(async () => {
    const slug = get(page).params.slug;
    try {
      post = await fetchPost(slug);
    } catch (err) {
      if (err instanceof Error) {
        error = err.message;
      } else {
        error = "An unknown error occurred while fetching the post.";
      }
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

<div class="min-h-screen bg-neutral-900 px-4 py-8">
  <div class="prose mx-auto max-w-3xl space-y-6">
    {#if loading}
      <p>Loading post...</p>
    {:else if error}
      <p class="text-red-400">{error}</p>
    {:else if post}
      <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold text-gray-100">{post.title}</h1>
        <a
          href={`https://blog.ifkash.dev/${post.slug}`}
          target="_blank"
          rel="noopener noreferrer"
          class="rounded-lg bg-blue-500 px-4 py-2 text-white hover:bg-blue-600"
        >
          Read on Hashnode
        </a>
      </div>
      {#if post.coverImage?.url}
        <!-- svelte-ignore a11y-img-redundant-alt -->
        <img
          src={post.coverImage.url}
          alt="Cover image"
          class="w-full rounded-lg"
        />
      {/if}
      <!-- render markdown with math support -->
      <Markdown
        remarkPlugins={[remarkMath]}
        rehypePlugins={[rehypeKatex]}
        source={post.content.markdown}
      />
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
