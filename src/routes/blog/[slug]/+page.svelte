<script lang="ts">
    import { onMount, tick } from "svelte";
    import { browser } from "$app/environment";

    // Import katex and its auto-render function specifically
    import renderMathInElement from "katex/contrib/auto-render";
    import "katex/dist/katex.min.css";

    // Import highlight.js
    import hljs from "highlight.js";
    import "highlight.js/styles/github-dark-dimmed.css";

    export let data;

    const renderContent = async () => {
        if (!browser) return;
        await tick();

        // 1. Highlight Code
        try {
            hljs.highlightAll();
        } catch (e) {
            console.error("Highlighting failed", e);
        }

        // 2. Render LaTeX
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
                    output: "html", // Use HTML output by default to avoid MathML issues in some browsers
                });
            } catch (e) {
                console.error("Katex rendering failed", e);
            }
        }
    };

    onMount(() => {
        renderContent();
    });

    // Re-run if post changes
    $: if (browser && data.post) {
        renderContent();
    }
</script>

<svelte:head>
    <title>{data.post.title} — Kashif</title>
    <meta name="description" content={data.post.subtitle || data.post.title} />
</svelte:head>

<div class="max-w-3xl mx-auto px-6 py-12 md:py-20 select-text">
    <!-- Navigation -->
    <nav class="mb-12">
        <a
            href="/blog"
            class="inline-flex items-center text-sm font-medium text-[var(--color-paragraph)] hover:text-[var(--color-headline)] transition-colors group"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="mr-2 group-hover:-translate-x-1 transition-transform"
            >
                <path d="m15 18-6-6 6-6" />
            </svg>
            Back to Blog
        </a>
    </nav>

    <!-- Article Header -->
    <header class="mb-12 space-y-6">
        {#if data.post.coverImage?.url}
            <figure
                class="w-full aspect-[2/1] rounded-2xl overflow-hidden bg-[var(--color-surface-hover)] border border-[var(--color-border)] shadow-sm"
            >
                <img
                    src={data.post.coverImage.url}
                    alt={data.post.title}
                    class="w-full h-full object-cover"
                />
            </figure>
        {/if}

        <div class="space-y-4">
            <div
                class="hidden md:flex flex-wrap items-center gap-3 text-sm text-[var(--color-paragraph)] font-medium"
            >
                <time datetime={data.post.publishedAt}>
                    {new Date(data.post.publishedAt).toLocaleDateString(
                        "en-US",
                        {
                            year: "numeric",
                            month: "long",
                            day: "numeric",
                        },
                    )}
                </time>
                {#if data.post.tags?.length}
                    <span aria-hidden="true" class="text-[var(--color-border)]"
                        >•</span
                    >
                    <div class="flex flex-wrap gap-2">
                        {#each data.post.tags as tag}
                            <span
                                class="px-2 py-0.5 rounded-full bg-[var(--color-surface-hover)] text-[var(--color-headline)] text-xs border border-[var(--color-border)]"
                            >
                                {tag.name}
                            </span>
                        {/each}
                    </div>
                {/if}
            </div>

            <h1
                class="font-serif text-3xl md:text-5xl font-extrabold tracking-tight text-[var(--color-headline)] leading-[1.1]"
            >
                {data.post.title}
            </h1>

            {#if data.post.subtitle}
                <p
                    class="font-serif text-xl md:text-2xl text-[var(--color-paragraph)] leading-relaxed font-light"
                >
                    {data.post.subtitle}
                </p>
            {/if}

            {#if data.post.url}
                <div>
                    <a
                        href={data.post.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-border)] text-sm font-medium text-[var(--color-headline)] transition-colors"
                    >
                        Read on Hashnode
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="14"
                            height="14"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path
                                d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                            />
                            <polyline points="15 3 21 3 21 9" />
                            <line x1="10" y1="14" x2="21" y2="3" />
                        </svg>
                    </a>
                </div>
            {/if}
        </div>
    </header>

    <!-- Article Content -->
    <article
        id="post-content"
        class="font-serif prose prose-invert prose-lg max-w-none
    prose-headings:text-[var(--color-headline)] prose-headings:font-bold prose-headings:tracking-tight
    prose-p:text-[var(--color-paragraph)] prose-p:leading-relaxed

    prose-code:font-mono prose-code:text-[var(--color-highlight)] prose-code:bg-[var(--color-surface-hover)] prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded-md prose-code:before:content-none prose-code:after:content-none
    prose-pre:font-mono prose-pre:bg-[#1e1e1e] prose-pre:border prose-pre:border-[var(--color-border)] prose-pre:rounded-xl prose-pre:shadow-sm
    prose-blockquote:border-l-[var(--color-highlight)] prose-blockquote:bg-[var(--color-surface-hover)] prose-blockquote:py-2 prose-blockquote:px-4 prose-blockquote:rounded-r-lg prose-blockquote:not-italic
    prose-img:rounded-xl prose-img:shadow-md prose-img:border prose-img:border-[var(--color-border)]
    prose-hr:border-[var(--color-border)]
    marker:text-[var(--color-highlight)]"
    >
        {@html data.post.content.html}
    </article>

    <!-- Footer / Post-Article -->
    <hr class="my-12 border-[var(--color-border)]" />

    <div
        class="flex justify-between items-center text-[var(--color-paragraph)]"
    >
        <a
            href="/blog"
            class="hover:text-[var(--color-headline)] transition-colors text-sm font-medium"
            >← More Posts</a
        >

        {#if data.post.url}
            <a
                href={data.post.url}
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-border)] text-sm font-medium text-[var(--color-headline)] transition-colors"
            >
                Read on Hashnode
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path
                        d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                    />
                    <polyline points="15 3 21 3 21 9" />
                    <line x1="10" y1="14" x2="21" y2="3" />
                </svg>
            </a>
        {/if}
    </div>
</div>

<style>
    /* Ensure KaTeX display equations scroll if too wide */
    :global(.katex-display) {
        overflow-x: auto;
        overflow-y: hidden;
        padding: 1rem 0;
    }

    /* Force link color to override prose/prose-invert specificities */
    #post-content :global(a) {
        color: var(--color-accent-primary);
        font-weight: 600;
        text-decoration: none;
        transition: all 0.2s ease-in-out;
    }

    #post-content :global(a:hover) {
        text-decoration: underline;
        opacity: 0.8;
    }
</style>
