<script lang="ts">
  const workHistory = [
    {
      role: "Engineer III",
      companies: [
        { name: "American Express", url: "https://www.americanexpress.com" },
        { name: "(via IntraEdge)", url: "https://intraedge.com" },
      ],
      duration: "February 2025 till present",
      location: "Bengaluru, IN",
      points: [
        "Built CC Listener module for Bridge Intelligence, capturing real-time CCs captions for thousands of Webex meetings, streaming them into Redis for downstream summarization using GPT-4o.",
        "Deployed CC Listener to Amex’s Hydra platform (RedHat OpenShift) across dev, QA, and pre-prod using Helm, Jenkins, and Docker; navigated complex internal infra and CI/CD pipelines.",
        "Built a hybrid semantic + fuzzy (trigram) retrieval system over 2M+ Confluence pages (runbooks); reduced latency from 30s (native search) to under 2s.",
        "Designed a multimodal runbook generation pipeline using GPT-4o, converting raw incident data (text + screenshots) into standardized Confluence pages.",
        "Tech stack: _Python, FastAPI, Langchain, PGVector, Streamlit, Node.js, Puppeteer, WebEx JS SDK, OpenShift_",
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
        "Fine-tuned Mistral-7B using QLoRA on 4070 Ti Super to incorporate internal documentation and company workflows.",
        "Deployed the fine-tuned model via vLLM on an NVIDIA T4 cluster; leveraged dynamic batching and quantized inference for sub-second 95th-percentile latency (TTFT).",
        "Led development of 'Beacon', a print-request automation tool powered by a fine-tuned ModernBERT NER model; parsed email and chat inputs to trigger document workflows.",
        "Built AskDB, an AI agent used by 300+ internal users; used llama 3.1 to translate business queries into SQL, fetch data and auto-generate charts and summaries.",
        "Tech stack: _vLLM, Python, PyTorch, QLoRA, peft, SFT, Mistral, Llama, SQL, FastAPI_",
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
        "Tech stack: _Node.js, ImageMagick, Angular_",
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
        "Migrated a Flask monolith to containerized microservices (Docker, Kubernetes), reducing AutoML job-launch latency by 40%.",
        "Tech stack: _Python, Flask, Docker, Kubernetes._",
      ],
    },
  ];
</script>

<svelte:head>
  <title>Work History • Portfolio</title>
</svelte:head>

<div
  class="min-h-screen text-gray-100 glegoo-regular px-4 sm:px-8 py-8 bg-neutral-900 flex justify-center"
>
  <div class="w-full max-w-2xl mx-auto">
    <!-- Header Section -->
    <section class="mb-8">
      <h2 class="text-2xl font-bold cormorant-garamond-700 flex items-center gap-2">
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
          <strong class="block my-2 glegoo-bold">
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
