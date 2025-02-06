<script lang="ts">
  const workHistory = [
    {
      role: "Associate Software Engineer",
      companies: [
        { name: "Fiery", url: "https://www.efi.com" },
        {
          name: "(an Epson company)",
          url: "https://corporate.epson/en/news/2024/240919.html",
        },
      ],
      duration: "Jul 2023 till now",
      location: "Bengaluru",
    },
    {
      role: "Data Scientist, Intern",
      companies: [
        { name: "Electronics for Imaging", url: "https://www.efi.com" },
      ],
      duration: "Jan 2023 till Jul 2023",
      location: "Bengaluru",
    },
    {
      role: "Fullstack Developer, Intern",
      companies: [
        { name: "Corteva Agriscience", url: "https://www.corteva.in" },
      ],
      duration: "Jul 2022 till Dec 2022",
      location: "Hyderabad",
    },
  ];
</script>

<svelte:head>
  <title>Work History • Portfolio</title>
</svelte:head>

<div
  class="min-h-screen bg-gray-900 text-gray-100 space-grotesk-400 px-4 sm:px-8 py-8"
>
  <!-- Header Section -->
  <section class="mb-8">
    <h2 class="text-3xl font-bold space-grotesk-700 flex items-center gap-2">
      <i class="fa-solid fa-briefcase"></i>
      Work
      <span class="text-sm text-gray-400">•</span>
      <a href="/" class="text-blue-500 hover:underline">home</a>
    </h2>
  </section>

  <!-- Work History Cards -->
  <section class="space-y-6">
    {#each workHistory as work}
      <div
        class="p-4 border border-gray-700 rounded hover:bg-gray-800 transition-colors"
      >
        <p class="text-lg">{work.role}</p>
        <strong class="block my-2 space-grotesk-600">
          {#each work.companies as company, i (company.url)}
            <a
              href={company.url}
              target="_blank"
              rel="noopener noreferrer"
              class="text-blue-400 hover:underline"
            >
              {company.name}
            </a>{i < work.companies.length - 1 ? " " : ""}
          {/each}
        </strong>
        <p class="text-sm text-gray-400">
          <i>{work.duration}</i> • {work.location}
        </p>
      </div>
    {/each}
  </section>
</div>
