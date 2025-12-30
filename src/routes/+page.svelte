<svelte:head>
  <title>Kashif — Applied ML Engineer</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, Applied ML Engineer."
  />
</svelte:head>

<script>
  import { browser } from '$app/environment';

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

<section class="mb-12">
  <p class="text-[var(--color-paragraph)] leading-relaxed mb-8">
    Systems-focused ML engineer with 3.5 YOE building numerical computing
    components, inference-optimized pipelines and agentic coding environments. I
    work at the intersection of ML systems, low-level frameworks and RL for
    code, designing execution sandboxes, structured task environments and
    deep-dive into unfamiliar codebases to build testable evals. Previously
    fine-tuned and deployed models and optimized inference systems org-wide.
  </p>
</section>

<!-- Featured Section (Example) -->
<section class="mb-16">
  <h3
    class="text-sm font-bold uppercase tracking-widest text-[var(--color-headline)] mb-6"
  >
    Featured
  </h3>
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
</section>

<style>
  /* Local overrides if needed */
</style>
