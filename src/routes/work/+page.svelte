<svelte:head>
  <title>Work — Kashif</title>
  <meta name="description" content="Roles, teams, and impact to date." />
</svelte:head>

<script lang="ts">
  type Company = { name: string; url: string };
  type Job = {
    role: string;
    companies: Company[];
    duration: string;
    location: string;
    startYear: number;
    endYear: number;
    isFreelance?: boolean;
  };

  const workHistory: Job[] = [
    {
      role: 'Engineer III',
      companies: [
        { name: 'American Express', url: 'https://www.americanexpress.com' }
      ],
      duration: 'February 2025 – Present',
      location: 'Bangalore, India',
      startYear: 2025,
      endYear: 2025,
    },
    {
      role: 'Associate Software Engineer',
      companies: [
        { name: 'Fiery, previously EFI', url: 'https://www.fiery.com' },
        { name: '(an Epson company)', url: 'https://corporate.epson/en/news/2024/240919.html' }
      ],
      duration: 'July 2023 – February 2025',
      location: 'Bangalore, India',
      startYear: 2023,
      endYear: 2025,
    },
    {
      role: 'Internship',
      companies: [
        { name: 'Fiery, previously EFI', url: 'https://www.fiery.com' },
        { name: '(an Epson company)', url: 'https://corporate.epson/en/news/2024/240919.html' }
      ],
      duration: 'January 2023 – July 2023',
      location: 'Bangalore, India',
      startYear: 2023,
      endYear: 2023,
    },
    {
      role: 'Internship',
      companies: [
        { name: 'Corteva Agriscience', url: 'https://www.corteva.in' }
      ],
      duration: 'July 2022 – December 2022',
      location: 'Hyderabad, India',
      startYear: 2022,
      endYear: 2022,
    }
  ];

  // Calculate the year range
  const allYears = workHistory.flatMap(w => [w.startYear, w.endYear]);
  const minYear = Math.min(...allYears);
  const maxYear = Math.max(...allYears);
  const years = Array.from({ length: maxYear - minYear + 1 }, (_, i) => maxYear - i);

  // Helper function to calculate position percentage for a year
  function getYearPosition(year: number): number {
    return ((year - minYear) / (maxYear - minYear)) * 100;
  }

  // Helper to italicize _like this_ within bullet points
  function em(text: string) {
    return text.replace(/_(.*?)_/g, '<em>$1</em>');
  }
</script>

<div class="min-h-screen selection:text-white" style="background-color: var(--color-background); color: var(--color-paragraph);">
  <div class="mx-auto max-w-4xl px-5 sm:px-6 pb-24">
    <!-- Header -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur" style="backdrop-filter: blur(12px); background-color: rgba(22, 22, 26, 0.6);">
      <div class="mx-auto max-w-4xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight" style="color: var(--color-headline);">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm focus:outline-none transition-colors" style="color: var(--color-paragraph);" onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.backgroundColor='transparent'">Home</a>
        </nav>
      </div>
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
    </header>

    <!-- Title -->
    <section class="pt-14 sm:pt-20" aria-labelledby="work-title">
      <h1 id="work-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight" style="color: var(--color-headline);">Work</h1>
      <p class="mt-2 max-w-2xl" style="color: var(--color-paragraph);">Applied ML systems, inference, and product‑grade tooling.</p>
    </section>

    <!-- Timeline Section -->
    <section class="mt-12">
      <div class="relative">
        <!-- Years and Timeline -->
        <div class="flex gap-8">
          <!-- Years Column -->
          <div class="w-16 flex-shrink-0">
            <div class="space-y-0">
              {#each years as year}
                <div
                  class="text-sm font-mono h-32 flex items-start"
                  style="color: var(--color-secondary);"
                >
                  {year}
                </div>
              {/each}
            </div>
          </div>

          <!-- Timeline and Jobs -->
          <div class="flex-1 relative">
            <!-- Jobs -->
            <div class="space-y-6 relative z-10">
              {#each workHistory as work, i (i)}
                <div 
                  class="group"
                  style="margin-top: {i === 0 ? '0' : 'calc(var(--job-spacing, 0px))'}"
                >
                  <!-- Job Card -->
                  <div class="ml-8 rounded-2xl border p-5 transition-colors" style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1);" onmouseover="this.style.borderColor='var(--color-highlight)'; this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.borderColor='var(--color-secondary)'; this.style.backgroundColor='rgba(114, 117, 126, 0.1)'">
                    <header class="flex flex-wrap items-start justify-between gap-3">
                      <div class="flex-1">
                        <h2 class="text-lg font-medium" style="color: var(--color-headline);">{work.role}</h2>
                        <div class="mt-1 text-sm" style="color: var(--color-paragraph);">
                          {#each work.companies as company, j (company.url)}
                            <a href={company.url} target="_blank" rel="noopener noreferrer" class="transition-colors" style="color: inherit;" onmouseover="this.style.color='var(--color-highlight)'; this.style.textDecoration='underline'" onmouseout="this.style.color='inherit'; this.style.textDecoration='none'">{company.name}</a>{j < work.companies.length - 1 ? ' ' : ''}
                          {/each}
                        </div>
                      </div>
                      <div class="shrink-0 text-right text-sm" style="color: var(--color-paragraph);">
                        <div style="color: var(--color-paragraph);">{work.duration}</div>
                        <div class="mt-1" style="color: var(--color-secondary);">📍 {work.location}</div>
                      </div>
                    </header>
                  </div>

                  <!-- Timeline connector dot -->
                  <div
                    class="absolute left-2 mt-8 w-4 h-4 rounded-full transform -translate-x-1/2 transition-colors"
                    style="top: 2.5rem; background-color: rgba(114, 117, 126, 0.1); border: 1px solid rgba(44, 182, 125, 0.5);"
                    onmouseover="this.style.borderColor='rgb(44, 182, 125)'"
                    onmouseout="this.style.borderColor='rgba(44, 182, 125, 0.5)'"
                  ></div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Footer -->
    <footer class="mt-24">
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm" style="color: var(--color-secondary);">
        <a class="transition-colors" onmouseover="this.style.color='var(--color-paragraph)'" onmouseout="this.style.color='var(--color-secondary)'" href="/">Back to home</a>
        <div style="color: var(--color-secondary);">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>

<style>
  :global(em) {
    font-style: italic;
  }
</style>
