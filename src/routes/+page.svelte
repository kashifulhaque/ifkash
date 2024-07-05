<script lang="ts">
  let title: string = "Kashiful Haque";

  const socialLinks = [
    {
      href: "https://ifkash.hashnode.dev",
      icon: "fa-brands fa-hashnode",
      text: "Blog",
    },
    {
      href: "https://www.linkedin.com/in/kashifulhaque",
      icon: "fa-brands fa-linkedin",
      text: "Linkedin",
    },
    {
      href: "https://github.com/kashifulhaque",
      icon: "fa-brands fa-github",
      text: "Github",
    },
    {
      href: "https://leetcode.com/u/ifkash/",
      icon: "fa-solid fa-code",
      text: "Leetcode",
    },
    {
      href: "https://www.instagram.com/enderboi25/",
      icon: "fa-brands fa-instagram",
      text: "Instagram",
    },
  ];

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
  <section id="wrapper--hero" class="section--page">
    <a
      href="https://twitter.com/Deltanpopo/status/1701793604967231699"
      target="_blank"
      rel="noopener noreferrer"
    >
      <img
        id="profile-pic"
        src="images/gojo.webp"
        alt="Gojo Satoru in Shibuya"
        title="Gojo Satoru in Shibuya • Source: @Deltanpopo"
      />
    </a>

    <div>
      <h1 id="user-name">{title}</h1>
      <p>
        <a
          href="mailto:haque.kashiful7@gmail.com"
          target="_blank"
          rel="noopener noreferrer"
          class="email"
        >
          <i class="fa-solid fa-envelope"></i> haque.kashiful7@gmail.com</a
        >
      </p>
      <p>
        <a class="resume" href="./assets/Kashiful_Haque.pdf" target="_blank"
          ><i class="fa-solid fa-download"></i> Resume
        </a>
      </p>
    </div>
  </section>

  <section class="section--page">
    <div id="socials--list">
      {#each socialLinks as { href, icon, text }}
        <a {href} target="_blank" rel="noopener noreferrer" class="profiles">
          <i class={icon}></i>
          {text}
        </a>
      {/each}
    </div>
  </section>

  <section class="section--page">
    <h2><i class="fa-solid fa-bolt"></i> Summary</h2>
    <p>
      I work at the intersection of Software Engineering, Data Science & Machine
      Learning focussing on Natural Language Processing (NLP), Generative AI
      (GenAI) and Large Language Models (LLMs). I also have a keen interest in
      cloud, linux and other tools surrounding tech.
      <br />
      <br />
      <small
        >Born and brought up in Kolkata, I lived most of my life there. Since
        2022, I have travelled to cities such as Hyderabad, Chennai and
        Bengaluru (current) for work/studies.</small
      >
    </p>
  </section>

  <section id="work-history-wrapper" class="section--page">
    <h2><i class="fa-solid fa-briefcase"></i> Work</h2>

    <div class="card--work-history">
      👨‍💻 Associate Software Engineer @
      <strong class="company"
        ><a href="https://www.efi.com" target="_blank" rel="noopener noreferrer"
          >Electronics for Imaging</a
        ></strong
      >
      🏢
      <p>
        📅 07/2023 - <span class="present">Present</span> • Bengaluru, India 📍
      </p>
    </div>

    <div class="card--work-history">
      👨‍🔬 Data Scientist, Intern @
      <strong class="company"
        ><a href="https://www.efi.com" target="_blank" rel="noopener noreferrer"
          >Electronics for Imaging</a
        ></strong
      >
      🏢
      <p>🗓️ 01/2023 - 07/2023 • Bengaluru, India 📍</p>
    </div>

    <div class="card--work-history">
      👨‍💻 Fullstack Developer, Intern @
      <strong class="company"
        ><a
          href="https://www.corteva.in"
          target="_blank"
          rel="noopener noreferrer">Corteva Agriscience</a
        ></strong
      >
      🏢
      <p>📆 07/2022 - 12/2022 • Hyderabad, India 📍</p>
    </div>
  </section>

  <section id="work-history-wrapper" class="section--page">
    <h2><i class="fa-solid fa-graduation-cap"></i> Education</h2>
    <div class="card--work-history">
      <strong>Indian Institute of Technology Madras</strong>
      <p>Bachelor of Science, Data Science and Applications</p>
      <p>12/2020 - 09/2024</p>
    </div>

    <div class="card--work-history">
      <strong>University of Calcutta</strong>
      <p>Bachelor of Technology, Electronics & Communications Engineering</p>
      <p>08/2016 - 03/2019</p>
    </div>
  </section>

  <section class="section--page">
    <h2><i class="fa-solid fa-microchip"></i> Tech</h2>

    <div id="wrapper--techstack__items">
      {#each techStack as group}
        <div class="card--techstack">
          <span>
            {#each group as { href, text }, i}
              <a {href} target="_blank" rel="noopener noreferrer">{text}</a
              >{#if i < group.length - 1}
                {", "}
              {/if}
            {/each}
          </span>
        </div>
      {/each}
    </div>
  </section>

  <a href="./assets/Kashiful_Haque-dark.pdf" target="_blank"
    ><hr class="center-ball" /></a
  >
  <section class="section--page section--page-text-center footer">
    <a target="_blank" rel="noopener noreferrer" href="https://kit.svelte.dev"
      >Made with <i class="fa-solid fa-heart"></i> using SvelteKit</a
    >
    🚧
    <a
      target="_blank"
      rel="noopener noreferrer"
      href="https://github.com/kashifulhaque/ifkash">Fork me on GitHub</a
    >
  </section>
</div>

<svelte:head>
  <title>{title} • Portfolio</title>
</svelte:head>
