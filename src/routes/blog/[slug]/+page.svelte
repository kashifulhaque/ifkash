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
    <a href="/blog" class="back-link">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
      <span>All writing</span>
    </a>
  </nav>

  <header class="article-header">
    <span class="eyebrow">writing · entry</span>
    <h1 class="article-title">{data.post.title}</h1>
    {#if data.post.subtitle}
      <p class="article-subtitle">{data.post.subtitle}</p>
    {/if}
    <time class="article-date" datetime={data.post.publishedAt}>
      {new Date(data.post.publishedAt).toLocaleDateString("en-US", {
        year: "numeric",
        month: "long",
        day: "numeric",
      })}
    </time>
  </header>

  <div
    id="post-content"
    class="prose prose-lg max-w-none
      prose-headings:text-[var(--text-primary)] prose-headings:font-semibold
      prose-p:text-[var(--text-secondary)] prose-p:leading-relaxed
      prose-code:text-[var(--text-primary)] prose-code:bg-[var(--surface-raised)] prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:before:content-none prose-code:after:content-none
      prose-pre:bg-[var(--surface-raised)] prose-pre:border prose-pre:border-[var(--border)] prose-pre:rounded-lg
      prose-blockquote:border-l-[var(--text-faint)] prose-blockquote:bg-[var(--surface-raised)] prose-blockquote:py-1 prose-blockquote:px-4 prose-blockquote:rounded-r prose-blockquote:not-italic
      prose-img:rounded-lg prose-img:border prose-img:border-[var(--border)]
      prose-hr:border-[var(--border)]
      prose-a:text-[var(--text-primary)] prose-a:underline prose-a:decoration-[var(--text-faint)] prose-a:underline-offset-2 hover:prose-a:decoration-[var(--text-primary)]
      marker:text-[var(--text-faint)]"
  >
    {@html data.post.content.html}
  </div>

  <footer class="article-footer">
    <a href="/blog" class="back-link">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
      <span>More writing</span>
    </a>
    {#if data.post.url}
      <a href={data.post.url} target="_blank" rel="noopener noreferrer" class="hashnode-link">
        <span>Open on Hashnode</span>
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M7 17L17 7M9 7h8v8"/></svg>
      </a>
    {/if}
  </footer>
</article>

<style>
  .article {
    max-width: var(--canvas-narrow);
    margin: 0 auto;
    animation: fade-up var(--dur-base) var(--ease-out-quart);
  }

  .article-nav {
    margin-bottom: var(--space-2xl);
  }

  .back-link {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.85rem;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text-secondary);
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: 999px;
    transition: all var(--dur-fast) var(--ease-out-quart);
  }

  .back-link:hover {
    color: var(--accent);
    border-color: var(--accent);
  }

  .back-link svg {
    transition: transform var(--dur-fast) var(--ease-out-quart);
  }

  .back-link:hover svg {
    transform: translateX(-2px);
  }

  .article-header {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    margin-bottom: var(--space-2xl);
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--border);
  }

  .article-date {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
    margin-top: 0.5rem;
  }

  .article-date::before {
    content: "";
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent);
  }

  .article-title {
    font-size: clamp(2rem, 4vw + 0.5rem, 3rem);
    font-weight: 400;
    letter-spacing: -0.035em;
    line-height: 1.05;
    color: var(--text-primary);
  }

  .article-subtitle {
    font-family: var(--font-serif);
    font-size: 1.25rem;
    font-weight: 300;
    font-style: italic;
    color: var(--text-secondary);
    line-height: 1.4;
    letter-spacing: -0.01em;
  }

  .article-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    margin-top: var(--space-3xl);
    padding-top: var(--space-xl);
    border-top: 1px solid var(--border);
  }

  .hashnode-link {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text-tertiary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .hashnode-link:hover {
    color: var(--accent);
  }

  .hashnode-link svg {
    color: var(--text-faint);
    transition: transform var(--dur-fast), color var(--dur-instant);
  }

  .hashnode-link:hover svg {
    color: var(--accent);
    transform: translate(2px, -2px);
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
