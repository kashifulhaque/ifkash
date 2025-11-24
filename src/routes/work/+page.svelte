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
      role: 'ML Engineer',
      companies: [
        { name: 'Wand AI', url: 'https://wand.ai' }
      ],
      duration: 'Nov 2025 - Present',
      location: 'Palo Alto, CA, USA',
      startYear: 2025,
      endYear: 2026
    },
    {
      role: 'Engineer III',
      companies: [
        { name: 'American Express', url: 'https://www.americanexpress.com' }
      ],
      duration: 'Feb 2025 – Nov 2025',
      location: 'Bangalore, KA, India',
      startYear: 2025,
      endYear: 2025,
    },
    {
      role: 'Associate Software Engineer',
      companies: [
        { name: 'Fiery, previously EFI', url: 'https://www.fiery.com' },
        { name: '(an Epson company)', url: 'https://corporate.epson/en/news/2024/240919.html' }
      ],
      duration: 'Jan 2023 – Feb 2025',
      location: 'Bangalore, KA, India',
      startYear: 2023,
      endYear: 2025,
    },
    {
      role: 'Internship',
      companies: [
        { name: 'Corteva Agriscience', url: 'https://www.corteva.in' }
      ],
      duration: 'Jul 2022 – Dec 2022',
      location: 'Hyderabad, TG, India',
      startYear: 2022,
      endYear: 2022,
    }
  ];

  // Helper to italicize _like this_ within bullet points
  function em(text: string) {
    return text.replace(/_(.*?)_/g, '<em>$1</em>');
  }
</script>

<div class="max-w-3xl mx-auto">
  <!-- Title -->
  <section class="mb-12" aria-labelledby="work-title">
    <h1 id="work-title" class="text-3xl font-bold tracking-tight text-[var(--color-headline)]">Work</h1>
    <p class="mt-2 text-lg text-[var(--color-paragraph)]">Applied ML systems, inference, and product‑grade tooling.</p>
  </section>

  <!-- Timeline Section -->
  <section class="space-y-8">
    {#each workHistory as work, i (i)}
      <div class="group relative pl-8 border-l border-[var(--color-border)] last:border-0 pb-8 last:pb-0">
        <!-- Timeline dot -->
        <div class="absolute -left-[5px] top-2 w-[9px] h-[9px] rounded-full bg-[var(--color-border)] group-hover:bg-[var(--color-highlight)] transition-colors"></div>

        <div class="flex flex-col sm:flex-row sm:justify-between sm:items-baseline gap-1 mb-2">
          <h2 class="text-xl font-semibold text-[var(--color-headline)]">{work.role}</h2>
          <span class="text-sm font-mono text-[var(--color-secondary)] whitespace-nowrap">{work.duration}</span>
        </div>

        <div class="text-[var(--color-paragraph)] mb-1">
          {#each work.companies as company, j (company.url)}
            <a href={company.url} target="_blank" rel="noopener noreferrer" class="hover:text-[var(--color-highlight)] transition-colors inline-block border-b border-transparent hover:border-[var(--color-highlight)]">
              {company.name}
            </a>{j < work.companies.length - 1 ? ' ' : ''}
          {/each}
        </div>

        <div class="text-sm text-[var(--color-secondary)] flex items-center gap-1">
          <span>📍 {work.location}</span>
        </div>
      </div>
    {/each}
  </section>
</div>

<style>
  :global(em) {
    font-style: italic;
    color: var(--color-headline);
  }
</style>
