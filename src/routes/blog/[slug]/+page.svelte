<script lang="ts">
  import { onMount, tick } from "svelte";
  import { browser } from "$app/environment";

  import renderMathInElement from "katex/contrib/auto-render";
  import "katex/dist/katex.min.css";

  import hljs from "highlight.js";
  import "highlight.js/styles/github-dark-dimmed.css";

  export let data;

  const renderContent = async () => {
    if (!browser) return;
    await tick();

    try {
      hljs.highlightAll();
    } catch (e) {
      console.error("Highlighting failed", e);
    }

    const element = document.getElementById("post-content");
    if (element) {
      try {
        renderMathInElement(element, {
          delimiters: [
            { left: "$$", right: "$$", display: true },
            { left: "$", right: "$", display: false },
            { left: "\\(", right: "\\)", display: false },
            { left: "\\[", right: "\\]", display: true },
          ],
          throwOnError: false,
          output: "html",
        });
      } catch (e) {
        console.error("Katex rendering failed", e);
      }
    }
  };

  onMount(() => {
    renderContent();
  });

  $: if (browser && data.post) {
    renderContent();
  }
</script>

<svelte:head>
  <title>{data.post.title} — Kashif</title>
  <meta name="description" content={data.post.subtitle || data.post.title} />
</svelte:head>

<article class="article">
  <nav class="article-nav">
    <a href="/blog" class="back-link">← Blog</a>
  </nav>

  <header class="article-header">
    <time class="article-date">
      {new Date(data.post.publishedAt).toLocaleDateString("en-US", {
        year: "numeric",
        month: "long",
        day: "numeric",
      })}
    </time>
    <h1 class="article-title">{data.post.title}</h1>
    {#if data.post.subtitle}
      <p class="article-subtitle">{data.post.subtitle}</p>
    {/if}
  </header>

  <div
    id="post-content"
    class="prose prose-invert prose-lg max-w-none
      prose-headings:text-white prose-headings:font-semibold
      prose-p:text-[var(--gray-300)] prose-p:leading-relaxed
      prose-code:text-white prose-code:bg-[var(--gray-900)] prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:before:content-none prose-code:after:content-none
      prose-pre:bg-[var(--gray-950)] prose-pre:border prose-pre:border-[var(--gray-800)] prose-pre:rounded-lg
      prose-blockquote:border-l-[var(--gray-600)] prose-blockquote:bg-[var(--gray-950)] prose-blockquote:py-1 prose-blockquote:px-4 prose-blockquote:rounded-r prose-blockquote:not-italic
      prose-img:rounded-lg prose-img:border prose-img:border-[var(--gray-800)]
      prose-hr:border-[var(--gray-800)]
      prose-a:text-white prose-a:underline prose-a:decoration-[var(--gray-600)] prose-a:underline-offset-2 hover:prose-a:decoration-white
      marker:text-[var(--gray-600)]"
  >
    {@html data.post.content.html}
  </div>

  <footer class="article-footer">
    <a href="/blog" class="back-link">← More posts</a>
    {#if data.post.url}
      <a href={data.post.url} target="_blank" rel="noopener noreferrer" class="hashnode-link">
        Hashnode ↗
      </a>
    {/if}
  </footer>
</article>

<style>
  .article {
    animation: fade-up var(--duration-slow) var(--ease-out);
  }
  
  .article-nav {
    margin-bottom: 3rem;
  }
  
  .back-link {
    font-size: 0.875rem;
    color: var(--gray-500);
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .back-link:hover {
    color: var(--white);
  }
  
  .article-header {
    margin-bottom: 3rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--gray-800);
  }
  
  .article-date {
    display: block;
    font-size: 0.8125rem;
    color: var(--gray-600);
    margin-bottom: 1rem;
  }
  
  .article-title {
    font-size: clamp(2rem, 5vw, 2.75rem);
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1.15;
    color: var(--white);
    margin-bottom: 0.75rem;
  }
  
  .article-subtitle {
    font-size: 1.125rem;
    color: var(--gray-400);
    line-height: 1.5;
  }
  
  .article-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 4rem;
    padding-top: 2rem;
    border-top: 1px solid var(--gray-800);
  }
  
  .hashnode-link {
    font-size: 0.8125rem;
    color: var(--gray-500);
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .hashnode-link:hover {
    color: var(--white);
  }

  :global(.katex-display) {
    overflow-x: auto;
    padding: 1rem 0;
  }
  
  @keyframes fade-up {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
