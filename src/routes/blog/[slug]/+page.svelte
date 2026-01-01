<script lang="ts">
    import { onMount, tick } from "svelte";
    import SvelteMarkdown from "svelte-markdown";
    import remarkMath from "remark-math";
    import rehypeKatex from "rehype-katex";

    export let data;

    const markdownOptions: any = {
        remarkPlugins: [remarkMath],
        rehypePlugins: [rehypeKatex],
    };

    onMount(async () => {
        // Dynamically highlight code blocks after rendering
        // We wait for tick to ensure DOM is updated by SvelteMarkdown
        await tick();
        if ((window as any).hljs) {
            (window as any).hljs.highlightAll();
        }
    });

    // Re-run highlighting if post changes (unlikely in this flow but good practice)
    $: if (data.post) {
        (async () => {
            await tick();
            if ((window as any).hljs) {
                (window as any).hljs.highlightAll();
            }
        })();
    }
</script>

<svelte:head>
    <title>{data.post.title} — Kashif</title>
    <meta name="description" content={data.post.subtitle || data.post.title} />

    <!-- KaTeX for Math -->
    <link
        rel="stylesheet"
        href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css"
        integrity="sha384-n8MVd4RsNIU0tAv4ct0nTaAbDJwPJzDEaqSD1odI+WdtXRGWt2kTvGFasHpSy3SV"
        crossorigin="anonymous"
    />

    <!-- Highlight.js for Syntax Highlighting -->
    <link
        rel="stylesheet"
        href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark-dimmed.min.css"
    />
    <script
        src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"
    ></script>
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
                class="flex flex-wrap items-center gap-3 text-sm text-[var(--color-paragraph)] font-medium"
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
                class="text-3xl md:text-5xl font-extrabold tracking-tight text-[var(--color-headline)] leading-[1.1]"
            >
                {data.post.title}
            </h1>

            {#if data.post.subtitle}
                <p
                    class="text-xl md:text-2xl text-[var(--color-paragraph)] leading-relaxed font-light"
                >
                    {data.post.subtitle}
                </p>
            {/if}
        </div>
    </header>

    <!-- Article Content -->
    <article
        class="prose prose-invert prose-lg max-w-none
    prose-headings:text-[var(--color-headline)] prose-headings:font-bold prose-headings:tracking-tight
    prose-p:text-[var(--color-paragraph)] prose-p:leading-relaxed
    prose-a:text-[var(--color-highlight)] prose-a:no-underline hover:prose-a:underline
    prose-code:text-[var(--color-highlight)] prose-code:bg-[var(--color-surface-hover)] prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded-md prose-code:before:content-none prose-code:after:content-none
    prose-pre:bg-[#1e1e1e] prose-pre:border prose-pre:border-[var(--color-border)] prose-pre:rounded-xl prose-pre:shadow-sm
    prose-blockquote:border-l-[var(--color-highlight)] prose-blockquote:bg-[var(--color-surface-hover)] prose-blockquote:py-2 prose-blockquote:px-4 prose-blockquote:rounded-r-lg prose-blockquote:not-italic
    prose-img:rounded-xl prose-img:shadow-md prose-img:border prose-img:border-[var(--color-border)]
    prose-hr:border-[var(--color-border)]
    marker:text-[var(--color-highlight)]"
    >
        <SvelteMarkdown
            source={data.post.content.markdown}
            {...markdownOptions}
        />
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
        <!-- Potential Share Buttons could go here -->
    </div>
</div>

<style>
    /* Custom overrides for specific markdown elements if needed */
    :global(.katex-display) {
        overflow-x: auto;
        overflow-y: hidden;
        padding: 1rem 0;
    }
</style>
