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
  };

  const workHistory: Job[] = [
    {
      role: 'Engineer III',
      companies: [
        { name: 'American Express', url: 'https://www.americanexpress.com' }
      ],
      duration: 'February 2025 – Present',
      location: 'Bengaluru, India',
    },
    {
      role: 'Associate Software Engineer',
      companies: [
        { name: 'Fiery, previously EFI', url: 'https://www.fiery.com' },
        { name: '(an Epson company)', url: 'https://corporate.epson/en/news/2024/240919.html' }
      ],
      duration: 'July 2023 – February 2025',
      location: 'Bengaluru, India',
    },
    {
      role: 'Internship',
      companies: [
        { name: 'Fiery, previously EFI', url: 'https://www.fiery.com' },
        { name: '(an Epson company)', url: 'https://corporate.epson/en/news/2024/240919.html' }
      ],
      duration: 'January 2023 – July 2023',
      location: 'Bengaluru, India',
    },
    {
      role: 'Internship',
      companies: [
        { name: 'Corteva Agriscience', url: 'https://www.corteva.in' }
      ],
      duration: 'July 2022 – December 2022',
      location: 'Hyderabad, India',
    }
  ];

  // Tiny helper to italicize _like this_ within bullet points
  function em(text: string) {
    return text.replace(/_(.*?)_/g, '<em>$1</em>');
  }
</script>

<div class="min-h-screen bg-neutral-950 text-neutral-200 selection:bg-neutral-800 selection:text-white">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur supports-[backdrop-filter]:bg-neutral-950/60">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight text-neutral-100">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">Home</a>
        </nav>
      </div>
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
    </header>

    <!-- Title -->
    <section class="pt-14 sm:pt-20" aria-labelledby="work-title">
      <h1 id="work-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight text-neutral-100">Work</h1>
      <p class="mt-2 text-neutral-400 max-w-2xl">Applied ML systems, inference, and product‑grade tooling.</p>
    </section>

    <!-- Timeline -->
    <section class="mt-8 grid gap-4">
      {#each workHistory as work, i (i)}
        <article class="group rounded-2xl border border-neutral-800 bg-neutral-900/60 p-5 hover:border-neutral-700 hover:bg-neutral-900">
          <header class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h2 class="text-lg font-medium text-neutral-100">{work.role}</h2>
              <div class="mt-1 text-sm text-neutral-300">
                {#each work.companies as company, i (company.url)}
                  <a href={company.url} target="_blank" rel="noopener noreferrer" class="hover:underline">{company.name}</a>{i < work.companies.length - 1 ? ' ' : ''}
                {/each}
              </div>
            </div>
            <div class="shrink-0 text-right text-sm text-neutral-400">
              <div>{work.duration}</div>
              <div class="text-neutral-500">📍 {work.location}</div>
            </div>
          </header>
        </article>
      {/each}
    </section>

    <!-- Footer -->
    <footer class="mt-16">
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm text-neutral-500">
        <a class="hover:text-neutral-300" href="/">Back to home</a>
        <div class="text-neutral-600">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>
