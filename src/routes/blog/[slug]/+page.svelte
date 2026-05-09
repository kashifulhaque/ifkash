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
    <a href="/blog" class="back-link">&larr; Back to Index</a>
  </nav>

  <header class="article-header">
    <div class="meta-row">
      <time class="article-date">
        {new Date(data.post.publishedAt).toLocaleDateString("en-US", {
          year: "numeric",
          month: "long",
          day: "numeric",
        })}
      </time>
      <span class="meta-figure">FIG_BLOG</span>
    </div>
    <h1 class="article-title">{data.post.title}</h1>
    {#if data.post.subtitle}
      <p class="article-subtitle">{data.post.subtitle}</p>
    {/if}
    <div class="ascii-rule"></div>
  </header>

  <div id="post-content" class="lesson-article">
    {@html data.post.content.html}
  </div>

  <div class="ascii-rule"></div>

  <footer class="article-footer">
    <a href="/blog" class="back-link">&larr; More posts</a>
    {#if data.post.url}
      <a
        href={data.post.url}
        target="_blank"
        rel="noopener noreferrer"
        class="hashnode-link"
      >
        Hashnode &nearr;
      </a>
    {/if}
  </footer>
</article>

<style>
  .article {
    max-width: 760px;
    margin: 0 auto;
    padding-top: 16px;
    animation: enter var(--dur-slow) var(--ease-out-expo);
  }

  .article-nav {
    margin-bottom: 32px;
  }

  .back-link,
  .hashnode-link {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-soft);
    border-bottom: none;
  }

  .back-link:hover,
  .hashnode-link:hover {
    color: var(--blueprint);
    border-bottom: none;
  }

  .article-header {
    margin-bottom: 40px;
  }

  .meta-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 24px;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--ink-mute);
  }

  .article-date {
    color: var(--ink-mute);
  }

  .meta-figure {
    color: var(--blueprint);
  }

  .article-title {
    font-family: var(--font-display);
    font-size: clamp(2.4rem, 6vw, 4rem);
    line-height: 0.92;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--blueprint);
    margin-bottom: 16px;
  }

  .article-subtitle {
    font-family: var(--font-body);
    font-size: clamp(1.05rem, 1.7vw, 1.2rem);
    line-height: 1.55;
    color: var(--ink-soft);
    font-style: italic;
  }

  /* ─── Lesson-style article prose ──────────────────────────── */

  .lesson-article {
    font-family: var(--font-body);
    font-size: 1.05rem;
    line-height: 1.7;
    color: var(--ink);
  }

  .lesson-article :global(p) {
    margin-bottom: 1.2em;
    text-align: justify;
    hyphens: auto;
  }

  .lesson-article :global(p:first-of-type::first-letter) {
    font-family: var(--font-display);
    float: left;
    font-size: 4.2rem;
    line-height: 0.85;
    padding: 0.06em 0.14em 0 0;
    color: var(--blueprint);
  }

  .lesson-article :global(h1),
  .lesson-article :global(h2),
  .lesson-article :global(h3),
  .lesson-article :global(h4),
  .lesson-article :global(h5),
  .lesson-article :global(h6) {
    font-family: var(--font-display);
    font-weight: 400;
    line-height: 1.05;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--blueprint);
    margin: 1.6em 0 0.6em;
  }

  .lesson-article :global(h2) {
    font-size: 1.9rem;
    border-top: 1px solid var(--rule-soft);
    padding-top: 1.2em;
  }

  .lesson-article :global(h3) {
    font-size: 1.4rem;
  }

  .lesson-article :global(h4) {
    font-size: 1.1rem;
  }

  .lesson-article :global(a) {
    color: var(--blueprint);
    border-bottom: 1px solid var(--blueprint-tint-strong);
    transition: border-color 0.15s, color 0.15s;
  }

  .lesson-article :global(a:hover) {
    border-bottom-color: var(--blueprint);
  }

  .lesson-article :global(strong) {
    color: var(--ink);
    font-weight: 700;
  }

  .lesson-article :global(em) {
    color: var(--ink);
  }

  .lesson-article :global(blockquote) {
    margin: 1.6em 0;
    padding: 1em 1.5em;
    border-left: 3px solid var(--blueprint);
    background: var(--blueprint-tint);
    color: var(--ink-soft);
    font-style: italic;
  }

  .lesson-article :global(code) {
    font-family: var(--font-mono);
    font-size: 0.92em;
    color: var(--ink);
    background: var(--code-bg);
    padding: 0.15em 0.4em;
    border: 1px solid var(--rule-soft);
  }

  .lesson-article :global(pre) {
    margin: 1.6em 0;
    padding: 16px 20px;
    background: var(--code-bg);
    border: 1px solid var(--rule-soft);
    overflow-x: auto;
    font-size: 0.88rem;
    line-height: 1.55;
  }

  .lesson-article :global(pre code) {
    background: transparent;
    border: none;
    padding: 0;
    font-size: inherit;
  }

  .lesson-article :global(hr) {
    border: none;
    height: 6px;
    margin: 32px 0;
    background-image:
      repeating-linear-gradient(
        to right,
        var(--blueprint) 0,
        var(--blueprint) 4px,
        transparent 4px,
        transparent 8px
      ),
      repeating-linear-gradient(
        to right,
        transparent 0,
        transparent 8px,
        var(--blueprint-tint-strong) 8px,
        var(--blueprint-tint-strong) 14px
      );
    background-size: 100% 3px, 100% 3px;
    background-position: 0 0, 0 3px;
    background-repeat: no-repeat;
  }

  .lesson-article :global(ul),
  .lesson-article :global(ol) {
    margin: 1.2em 0;
    padding-left: 1.5em;
  }

  .lesson-article :global(li) {
    margin-bottom: 0.4em;
  }

  .lesson-article :global(li::marker) {
    color: var(--blueprint);
  }

  .lesson-article :global(img) {
    max-width: 100%;
    height: auto;
    border: 1px solid var(--rule-soft);
    margin: 1.6em 0;
  }

  .lesson-article :global(table) {
    width: 100%;
    margin: 1.6em 0;
    border-collapse: collapse;
    font-family: var(--font-mono);
    font-size: 0.88rem;
  }

  .lesson-article :global(th),
  .lesson-article :global(td) {
    padding: 8px 12px;
    border: 1px solid var(--rule-soft);
    text-align: left;
  }

  .lesson-article :global(th) {
    background: var(--bg-surface);
    color: var(--blueprint);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 0.78rem;
  }

  /* ─── Footer ───────────────────────────────────────────────── */

  .article-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 16px;
  }

  /* ─── Katex / hljs overrides ──────────────────────────────── */

  :global(.katex-display) {
    overflow-x: auto;
    padding: 1rem 0;
  }

  @media (max-width: 768px) {
    .article-title {
      font-size: clamp(2rem, 9vw, 2.6rem);
    }

    .lesson-article {
      font-size: 1rem;
    }

    .lesson-article :global(p:first-of-type::first-letter) {
      font-size: 3.2rem;
    }
  }
</style>
