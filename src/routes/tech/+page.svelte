<script lang="ts">
  const techStack = [
    [
      { href: "https://www.python.org/", text: "Python" },
      {
        href: "https://developer.mozilla.org/en-US/docs/Web/JavaScript",
        text: "JavaScript",
      },
      { href: "https://www.typescriptlang.org/", text: "TypeScript" },
      { href: "https://cplusplus.com/doc/tutorial/", text: "C++" },
      { href: "https://www.rust-lang.org/", text: "Rust" },
      { href: "https://go.dev/", text: "Go" },
      { href: "https://en.wikipedia.org/wiki/SQL", text: "SQL" },
    ],
    [
      { href: "https://nodejs.org/en/", text: "Node.js" },
      { href: "https://expressjs.com/", text: "Express.js" },
      { href: "https://vuejs.org/", text: "Vue" },
      { href: "https://angular.io/", text: "Angular" },
      { href: "https://fastapi.tiangolo.com/", text: "FastAPI" },
      { href: "https://flask.palletsprojects.com/", text: "Flask" },
      { href: "https://tailwindcss.com/", text: "Tailwind" },
    ],
    [
      { href: "https://numpy.org/", text: "NumPy" },
      { href: "https://pandas.pydata.org/", text: "Pandas" },
      { href: "https://scikit-learn.org/", text: "scikit-learn" },
      { href: "https://pytorch.org/", text: "PyTorch" },
      { href: "https://huggingface.co/", text: "huggingface" },
      { href: "https://jax.readthedocs.io/en/latest/", text: "JAX" },
      { href: "https://www.nltk.org/", text: "NLTK" },
      { href: "https://spacy.io/", text: "spaCy" },
      { href: "https://developer.nvidia.com/cuda-toolkit", text: "CUDA" },
    ],
    [
      {
        href: "https://www.pinecone.io/learn/vector-database/",
        text: "Vector Database",
      },
      { href: "https://qdrant.tech/", text: "Qdrant" },
      { href: "https://www.mysql.com/", text: "MySQL" },
      { href: "https://www.sqlite.org/", text: "SQLite" },
      { href: "https://redis.io/", text: "Redis" },
      { href: "https://www.docker.com/", text: "Docker" },
      { href: "https://git-scm.com/", text: "Git" },
      { href: "https://www.kernel.org/", text: "Linux" },
    ],
  ];
</script>

<svelte:head>
  <title>Tech Stack • Portfolio</title>
</svelte:head>

<div
  class="min-h-screen bg-neutral-900 text-gray-100 space-grotesk-400 px-4 sm:px-8 py-8"
>
  <!-- Header Section -->
  <section class="mb-8">
    <h2 class="text-2xl font-bold space-grotesk-700 flex items-center gap-2">
      <i class="fa-solid fa-microchip"></i>
      Tech
      <span class="text-sm text-gray-400">•</span>
      <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
    </h2>
  </section>

  <!-- Tech Stack Cards -->
  <section class="space-y-4">
    {#each techStack as group}
      <div>
        <small class="text-base">
          {#each group as { href, text }, i (href)}
            <a
              {href}
              target="_blank"
              rel="noopener noreferrer"
              class="text-blue-300 hover:underline"
            >
              {text}
            </a>{#if i < group.length - 1}{" • "}
            {/if}
          {/each}
        </small>
      </div>
    {/each}
  </section>
</div>
