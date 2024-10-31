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

<div id="container--main">
  <section class="section--page">
    <h2>
      <i class="fa-solid fa-microchip"></i> Tech •
      <a href="/">home</a>
    </h2>

    <div id="wrapper--techstack__items">
      {#each techStack as group}
        <div class="card--techstack">
          <small>
            {#each group as { href, text }, i}
              <a {href} target="_blank" rel="noopener noreferrer">{text}</a
              >{#if i < group.length - 1}
                {", "}
              {/if}
            {/each}
          </small>
        </div>
      {/each}
    </div>
  </section>
</div>
