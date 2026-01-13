<script>
  import { auth } from "$lib/stores/auth";
  import { dev } from "$app/environment";

  const resumeUrl = dev
    ? "http://localhost:8787/api/resume?format=view"
    : "/api/resume?format=view";
</script>

<svelte:head>
  <title>Kashif — Applied ML Engineer</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, ML Engineer."
  />
</svelte:head>

<!-- Featured Section (Example) -->
<section class="mb-16">
  <div class="space-y-4">
    <!-- Resume Link (Open PDF directly) -->
    <a
      href={resumeUrl}
      class="group block w-full bg-[var(--color-surface)] border border-[var(--color-border)] p-6 rounded-lg hover:border-[var(--color-secondary)] transition-all cursor-pointer text-left"
    >
      <div class="flex items-center justify-between">
        <div>
          <h4
            class="text-lg font-semibold text-[var(--color-headline)] group-hover:text-[var(--color-highlight)] transition-colors"
          >
            Resume
          </h4>
          <p class="text-sm text-[var(--color-paragraph)] mt-1">
            Check out my full professional history.
          </p>
        </div>
        <span
          class="text-[var(--color-secondary)] group-hover:translate-x-1 group-hover:text-[var(--color-headline)] transition-all"
          >→</span
        >
      </div>
    </a>

    <!-- Resume Editor (only visible when authenticated) -->
    {#if $auth.isAuthenticated}
      <a
        href="/editor"
        class="group block bg-[var(--color-surface)] border border-[var(--color-border)] p-6 rounded-lg hover:border-[var(--color-secondary)] transition-all"
      >
        <div class="flex items-center justify-between">
          <div>
            <h4
              class="text-lg font-semibold text-[var(--color-headline)] group-hover:text-[var(--color-highlight)] transition-colors"
            >
              Edit Resume
            </h4>
            <p class="text-sm text-[var(--color-paragraph)] mt-1">
              View and edit your resume source files.
            </p>
          </div>
          <span
            class="text-[var(--color-secondary)] group-hover:translate-x-1 group-hover:text-[var(--color-headline)] transition-all"
            >✏️</span
          >
        </div>
      </a>
    {/if}
  </div>
</section>

<style>
  /* Local overrides if needed */
</style>
