<script lang="ts">
  const projects = [
    {
      title: "Boo",
      links: [{ text: "git repo", url: "https://github.com/VVIP-Kitchen/boo" }],
      description:
        "Innovative AI-Powered Discord Bot who engages in natural conversations, interprets and responds to image inputs, and generates high-quality images on demand. Equipped with advanced retrieval-augmented generation (RAG) capabilities, it seamlessly navigates and provides insights from IITM's BS in Data Science Discourse, enhancing community interactions and access to knowledge.",
      tech: [
        "Go",
        "Docker",
        "Python",
        "Discord.py",
        "Large Language Models (LLM)",
        "Meta's LLaMa 3.1 8B",
        "Cloudflare Workers AI",
        "GitHub Models",
      ],
    },
    {
      title: "Personal LLM",
      links: [
        { text: "git repo", url: "https://github.com/kashifulhaque/ifkash" },
        { text: "demo", url: "/chat" },
      ],
      description:
        "My personal assistant who knows about me and can provide general info too!",
      tech: [
        "Go",
        "Large Language Models (LLM)",
        "Meta's LLaMa 3.2 3B",
        "Cloudflare Workers AI",
      ],
    },
    {
      title: "Odeer",
      links: [
        { text: "git repo", url: "https://github.com/kashifulhaque/odeer" },
      ],
      htmlDescription:
        "It's a CLI tool like <a href='https://ollama.com/' target='_blank' rel='noopener noreferrer' class='text-blue-400 hover:underline'>ollama</a> to interact with LLMs on your terminal",
      tech: ["Go", "Large Language Models (LLM)", "Cloudflare Workers AI"],
    },
    {
      title: "berzifetch-rs",
      links: [
        {
          text: "git repo",
          url: "https://github.com/kashifulhaque/berzifetch-rs",
        },
      ],
      description: "A system fetch for Linux written in Rust.",
      tech: ["Rust"],
    },
    {
      title: "Vuether",
      links: [
        { text: "git repo", url: "https://github.com/kashifulhaque/vuether" },
        { text: "demo", url: "https://vuether.now.sh" },
      ],
      description: "A plain, simple weather web-app written in Vue.js",
      tech: ["Vue.js", "OpenWeatherMap API"],
    },
  ];
</script>

<svelte:head>
  <title>Some of my side projects</title>
</svelte:head>

<div
  class="min-h-screen text-gray-100 space-grotesk-400 px-4 sm:px-8 py-8"
>
  <!-- Header Section -->
  <section class="mb-8">
    <h2 class="text-2xl font-bold space-grotesk-700 flex items-center gap-2">
      <i class="fa-solid fa-diagram-project"></i>
      Projects
      <span class="text-sm text-gray-400">•</span>
      <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
    </h2>
  </section>

  <!-- Projects List -->
  <section class="space-y-6">
    {#each projects as project}
      <div class="p-4">
        <!-- Project Title -->
        <strong class="block text-xl space-grotesk-600">{project.title}</strong>
        
        <!-- Project Links, if any -->
        <div class="my-2">
          {#each project.links as link, j}
            <a
              href={link.url}
              target="_blank"
              rel="noopener noreferrer"
              class="text-blue-400 hover:underline"
            >
              {link.text}
            </a>{#if j < project.links.length - 1}
              <span class="mx-1">•</span>
            {/if}
          {/each}
        </div>
        
        <!-- Project description -->
        <p class="mb-2 text-base text-gray-300">
          {#if project.htmlDescription}
            {@html project.htmlDescription}
          {:else}
            {project.description}
          {/if}
        </p>

        <p class="text-sm text-gray-400 mb-1">Tech used:</p>
        <ul class="list-disc list-inside text-sm text-gray-400 space-y-1">
          {#each project.tech as techItem}
            <li>{techItem}</li>
          {/each}
        </ul>
      </div>
    {/each}
  </section>
</div>
