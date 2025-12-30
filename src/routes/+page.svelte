<svelte:head>
  <title>Kashif — Applied ML Engineer</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, Applied ML Engineer."
  />
</svelte:head>

<script>
  import { browser } from '$app/environment';
  import { auth } from '$lib/stores/auth';

  async function handleResumeDownload() {
    if (!browser) return;

    // Determine API URL based on environment
    const apiUrl = window.location.hostname === 'localhost'
      ? 'http://localhost:8787/api/resume'
      : 'https://ifkash.dev/api/resume';

    const fallbackUrl = '/assets/Kashiful_Haque.pdf';

    try {
      // Try to fetch from API
      const response = await fetch(apiUrl);
      
      if (!response.ok) {
        throw new Error('API request failed');
      }

      // Get the PDF blob
      const blob = await response.blob();
      
      // Create download link
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'Kashiful_Haque.pdf';
      document.body.appendChild(a);
      a.click();
      
      // Cleanup
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (error) {
      console.warn('Failed to fetch resume from API, falling back to static asset:', error);
      // Fallback to static asset
      window.location.href = fallbackUrl;
    }
  }
</script>

<!-- About Section -->
<section class="mb-16">
  <p class="text-[var(--color-paragraph)] leading-relaxed mb-4">
    I'm an Applied ML Engineer passionate about building intelligent systems
    that scale. Currently focused on agentic coding environments and
    inference-optimized ML systems.
  </p>
</section>

<!-- Featured Section (Example) -->
<section class="mb-16">
  <h3
    class="text-sm font-bold uppercase tracking-widest text-[var(--color-headline)] mb-6"
  >
    Featured
  </h3>
  
  <div class="space-y-4">
    <!-- Resume Download -->
    <button
      on:click={handleResumeDownload}
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
    </button>

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
