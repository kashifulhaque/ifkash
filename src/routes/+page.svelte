<script>
  import { onMount } from "svelte";
  import { dev } from "$app/environment";
  import EdgeField from "$lib/components/EdgeField.svelte";
  import { loadProgress, WONDER_COUNT } from "$lib/game/progress";
  import { seedLink } from "$lib/game/seed";

  // Progress from the tiny-planet game lives in localStorage, so it is read
  // after mount; the server render never shows the badge.
  /** @type {{ text: string, href: string } | null} */
  let badge = null;
  onMount(() => {
    const progress = loadProgress();
    if (!progress.complete) return;
    badge = {
      text: `${progress.found.length} of ${WONDER_COUNT} small wonders`,
      href: progress.seed ? seedLink(progress.seed, "") : "/game",
    };
  });

  const resumeUrl = dev
    ? "http://localhost:8787/api/cv?format=view"
    : "/api/cv?format=view";

  const indexRows = [
    { n: "01", name: "Work", desc: "Roles and companies", href: "/work" },
    {
      n: "02",
      name: "Projects",
      desc: "Open source and independent work",
      href: "/projects",
    },
    {
      n: "03",
      name: "RL",
      desc: "Reinforcement learning experiments",
      href: "/rl",
    },
    {
      n: "04",
      name: "Tools",
      desc: "Useful in-browser utilities",
      href: "/tools",
    },
    {
      n: "05",
      name: "Education",
      desc: "Academic background",
      href: "/education",
    },
    {
      n: "06",
      name: "Blog",
      desc: "Notes on engineering and ML",
      href: "/blog",
    },
    { n: "07", name: "Game", desc: "A tiny planet to wander", href: "/game" },
    {
      n: "08",
      name: "Roadmap",
      desc: "Interview course anchored to shipped work",
      href: "/roadmap",
    },
  ];
</script>

<svelte:head>
  <title>Kashiful Haque</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, an ML engineer in Bangalore."
  />
</svelte:head>

<section class="intro">
  <p class="intro-label">ML Engineer · Bangalore, India</p>
  <h1 class="display">
    Kashiful Haque<a href="/fitness" class="period" aria-label="Fitness dashboard">.</a>
  </h1>
  <p class="intro-copy">
    I build pre-training and post-training systems for large language models,
    reinforcement learning pipelines, and high-performance inference software
    in C++ and Rust.
  </p>

  <div class="actions">
    <a href={resumeUrl} class="btn btn-primary">Read resume</a>
    <a href="/game" class="btn btn--outline">Enter game mode</a>
    {#if badge}
      <a href={badge.href} class="wonder-badge" title="Every small wonder on the tiny planet has been found">
        <span class="wonder-badge-spark" aria-hidden="true">✦</span>
        {badge.text}
      </a>
    {/if}
  </div>

  <dl class="profile-facts">
    <div>
      <dt>Focus</dt>
      <dd>LLM systems</dd>
    </div>
    <div>
      <dt>Experience</dt>
      <dd>4+ years</dd>
    </div>
    <div>
      <dt>Core stack</dt>
      <dd>C++, Rust, and Python</dd>
    </div>
  </dl>
</section>

<EdgeField />

<section class="index-section" aria-labelledby="explore-heading">
  <div class="section-heading">
    <h2 id="explore-heading" class="section-title">Explore</h2>
    <p>Selected work, experiments, and tools.</p>
  </div>

  <ol class="index-list">
    {#each indexRows as row}
      <li>
        <a href={row.href} class="index-row">
          <span class="index-n">{row.n}</span>
          <span class="index-name">{row.name}</span>
          <span class="index-desc">{row.desc}</span>
          <span class="index-arrow" aria-hidden="true">↗</span>
        </a>
      </li>
    {/each}
  </ol>
</section>

<style>
  .intro {
    max-width: 800px;
    padding: 72px 0 40px;
  }

  .intro-label {
    margin: 0 0 24px;
    color: var(--ink-3);
    font-family: var(--font-label);
    font-size: 13px;
    font-weight: 500;
  }

  h1 {
    max-width: 12ch;
  }

  .period {
    color: var(--ink-3);
  }

  .period:hover {
    color: var(--foreground);
  }

  .intro-copy {
    max-width: 62ch;
    margin: 28px 0 0;
    color: var(--ink-2);
    font-size: 18px;
    line-height: 1.6;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 32px;
  }

  .wonder-badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 40px;
    padding: 0 14px;
    border: 1px solid var(--border);
    /* sketched frame rather than a geometric pill, so the badge sits in the
       same hand as the buttons beside it */
    border-radius: var(--r-frame-1);
    animation: sketch-boil-radius 1200ms step-end infinite;
    color: var(--ink-2);
    font-family: var(--font-label);
    font-size: 13px;
    text-decoration: none;
    transition: border-color 160ms var(--ease-inout), color 160ms var(--ease-inout);
  }

  .wonder-badge:hover {
    border-color: var(--blueprint);
    color: var(--foreground);
  }

  .wonder-badge-spark {
    color: var(--blueprint);
  }

  .profile-facts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 24px;
    margin: 64px 0 0;
  }

  .profile-facts div {
    min-width: 0;
  }

  .profile-facts dt {
    margin-bottom: 4px;
    color: var(--ink-3);
    font-size: 13px;
    font-weight: 500;
  }

  .profile-facts dd {
    margin: 0;
    color: var(--foreground);
    font-size: 14px;
  }

  .index-section {
    padding: 96px 0 24px;
  }

  .section-heading {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
    gap: 24px;
    margin-bottom: 32px;
  }

  .section-heading h2 {
    font-size: 28px;
  }

  .section-heading p {
    margin: 0;
    color: var(--ink-3);
    font-size: 13px;
  }

  .index-list {
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--border);
    list-style: none;
  }

  .index-row {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) minmax(180px, 0.8fr) 20px;
    align-items: baseline;
    gap: 24px;
    padding: 20px 0;
    border-bottom: 1px solid var(--border);
    color: var(--foreground);
  }

  .index-row:hover .index-name,
  .index-row:hover .index-arrow {
    color: var(--foreground);
  }

  .index-n,
  .index-desc,
  .index-arrow {
    color: var(--ink-3);
    font-size: 13px;
  }

  .index-n {
    font-family: var(--font-label);
  }

  .index-name {
    font-size: 18px;
    font-weight: 500;
  }

  .index-arrow {
    justify-self: end;
    transition: transform 160ms var(--ease-inout);
  }

  .index-row:hover .index-arrow {
    transform: translate(2px, -2px);
  }

  @media (max-width: 720px) {
    .intro {
      padding-top: 32px;
    }

    .profile-facts {
      grid-template-columns: 1fr;
      gap: 20px;
      margin-top: 48px;
    }

    .index-section {
      padding-top: 72px;
    }

    .section-heading {
      grid-template-columns: 1fr;
      gap: 8px;
    }

    .index-row {
      grid-template-columns: 36px minmax(0, 1fr) 20px;
      gap: 12px;
      padding: 18px 0;
    }

    .index-desc {
      display: none;
    }
  }
</style>
