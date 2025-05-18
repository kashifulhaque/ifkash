<script lang="ts">
  const workHistory = [
    {
      role: "Engineer III",
      companies: [
        { name: "American Express", url: "https://www.americanexpress.com" },
        { name: "(IntraEdge)", url: "https://intraedge.com" },
      ],
      duration: "February 2025 till present",
      location: "Bengaluru, IN",
      points: [
        "Shipped a real-time NLP pipeline that live summarizes 1,000+ WebEx meetings weekly, cutting down manual meeting note taking.",
        "Built an NLP-powered Confluence search service (FastAPI, PGVector) indexing 2M+ pages, supporting sub-1500ms 95th-percentile search results latency.",
        "_Tech stack: Python, FastAPI, Langchain, PGVector, Streamlit, Node.js, Puppeteer, WebEx JS SDK, OpenShift._",
      ],
    },
    {
      role: "Associate Software Engineer",
      companies: [
        { name: "Fiery, formerly known as EFI", url: "https://www.fiery.com" },
        {
          name: "(an Epson company)",
          url: "https://corporate.epson/en/news/2024/240919.html",
        },
      ],
      duration: "Jul 2023 till February 2025",
      location: "Bengaluru, IN",
      points: [
        "Fine-tuned Mistral‑7B using LoRA adapters on 4070 Ti Super, baking company knowledge into model.",
        "Deployed the fine-tuned model on a 4x1080 Ti GPU cluster via vLLM; engineered dynamic batching and employed mixed-precision inference for sub-750ms 95th-percentile Time-To-First-Token.",
        `Led development of "Beacon", an NER-based print-request automation service integrating with emails and natural language interfaces, leveraging a custom fine-tuned ModernBERT model.`,
        "Built AskDB, a natural language to insights platform, used by 300+ internal users to query company DBs and auto-generate visual insights.",
        "_Tech stack: vLLM, Python, PyTorch, LoRA, SFT, Mistral, Llama, Ollama, SQL, FastAPI._",
      ],
    },
    {
      role: "Data Scientist, Intern",
      companies: [
        { name: "Fiery, formerly known as EFI", url: "https://www.fiery.com" },
        {
          name: "(an Epson company)",
          url: "https://corporate.epson/en/news/2024/240919.html",
        },
      ],
      duration: "Jan 2023 till Jul 2023",
      location: "Bengaluru, IN",
      points: [
        "Created a product mockup pipeline using ImageMagick and Node.js, achieving cost-effective design automation.",
        "_Tech stack: Node.js, ImageMagick, Angular._",
      ],
    },
    {
      role: "Fullstack Developer, Intern",
      companies: [
        { name: "Corteva Agriscience", url: "https://www.corteva.in" },
      ],
      duration: "Jul 2022 till Dec 2022",
      location: "Hyderabad, IN",
      points: [
        "Migrated Flask monolith to microservices (Python, Docker, Kubernetes), optimizing AutoML job triggering and reducing job-launch latency by 40%.",
        "_Tech stack: Python, Flask, Docker, Kubernetes._",
      ],
    },
  ];
</script>

<svelte:head>
  <title>Work History • Portfolio</title>
</svelte:head>

<div
  class="min-h-screen text-gray-100 space-grotesk-400 px-4 sm:px-8 py-8 bg-neutral-900 flex justify-center"
>
  <div class="w-full max-w-2xl mx-auto">
    <!-- Header Section -->
    <section class="mb-8">
      <h2 class="text-2xl font-bold space-grotesk-700 flex items-center gap-2">
        <i class="fa-solid fa-briefcase"></i>
        Work
        <span class="text-sm text-gray-400">•</span>
        <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
      </h2>
    </section>

    <!-- Work History Cards -->
    <!-- Work History Cards -->
    <section class="space-y-6">
      {#each workHistory as work}
        <div class="p-4">
          <p class="text-lg">{work.role}</p>
          <strong class="block my-2 space-grotesk-600">
            {#each work.companies as company, i (company.url)}
              <a
                href={company.url}
                target="_blank"
                rel="noopener noreferrer"
                class="text-base text-blue-300 hover:underline"
              >
                {company.name}
              </a>{i < work.companies.length - 1 ? " " : ""}
            {/each}
          </strong>
          <p class="text-base text-gray-400">
            <i>{work.duration}</i> • {work.location}
          </p>
          <!-- Experience Bullet Points -->
          <ul class="list-disc ml-6 mt-2 space-y-1 text-base text-gray-200">
            {#each work.points as point}
              <li>
                {@html point.replace(
                  // Allow _italic_ syntax for tech stacks
                  /^_(.*?)_$/g,
                  "<em>$1</em>"
                )}
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    </section>
  </div>
</div>
