<svelte:head>
  <title>Education — Kashif</title>
  <meta name="description" content="Academic background and formal studies of Kashif." />
</svelte:head>

<div class="min-h-screen selection:text-white" style="background-color: var(--color-background); color: var(--color-paragraph);">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur" style="backdrop-filter: blur(12px); background-color: rgba(22, 22, 26, 0.6);">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight" style="color: var(--color-headline);">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm focus:outline-none transition-colors" style="color: var(--color-paragraph);" onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.backgroundColor='transparent'">Home</a>
        </nav>
      </div>
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
    </header>

    <!-- Title -->
    <section class="pt-14 sm:pt-20" aria-labelledby="edu-title">
      <h1 id="edu-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight" style="color: var(--color-headline);">Education</h1>
      <p class="mt-3 max-w-2xl" style="color: var(--color-paragraph);">Formal studies and academic foundation.</p>
    </section>

    <!-- Education entries -->
    <section class="mt-10 grid gap-6">
      <div class="rounded-2xl border p-5" style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1);">
        <h2 class="text-lg font-medium" style="color: var(--color-headline);">Indian Institute of Technology Madras</h2>
        <p class="mt-1 text-sm" style="color: var(--color-paragraph);">Bachelor of Science, Data Science and Applications</p>
        <p class="mt-1 text-xs" style="color: var(--color-secondary);">2020 – 2024</p>
      </div>
    </section>

    <!-- Footer -->
    <footer class="mt-16">
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm" style="color: var(--color-secondary);">
        <a class="transition-colors" onmouseover="this.style.color='var(--color-paragraph)'" onmouseout="this.style.color='var(--color-secondary)'" href="/">Back to home</a>
        <div style="color: var(--color-secondary);">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>
