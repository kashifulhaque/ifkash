<svelte:head>
  <title>Education — Kashif</title>
  <meta name="description" content="Academic background." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <span class="category-badge">Background</span>
    <h1 class="page-title">Education</h1>
    <p class="page-desc">Formal studies and academic foundation.</p>
  </header>

  <section class="education">
    <article class="edu-item">
      <div class="edu-main">
        <h2 class="edu-school">Indian Institute of Technology Madras</h2>
        <p class="edu-degree">B.Sc. Data Science and Applications</p>
      </div>
      <div class="edu-meta">
        <span class="edu-year">2020 — 2024</span>
        <span class="edu-location">Chennai, India</span>
      </div>
    </article>
  </section>

  <section class="skills-section">
    <h3 class="skills-title">Areas of Focus</h3>
    <div class="skills">
      {#each ["Machine Learning", "Deep Learning", "Statistics", "Linear Algebra", "Data Structures", "Algorithms", "Python", "Distributed Systems"] as skill}
        <span class="skill">{skill}</span>
      {/each}
    </div>
  </section>

  <section class="coursework-section">
    <h3 class="skills-title">Notable Coursework</h3>
    <div class="coursework-grid">
      <div class="course-item">
        <span class="course-code">CS6910</span>
        <span class="course-name">Deep Learning</span>
      </div>
      <div class="course-item">
        <span class="course-code">CS5691</span>
        <span class="course-name">Pattern Recognition & ML</span>
      </div>
      <div class="course-item">
        <span class="course-code">MA5890</span>
        <span class="course-name">Statistical Methods</span>
      </div>
      <div class="course-item">
        <span class="course-code">CS3110</span>
        <span class="course-name">Data Structures & Algorithms</span>
      </div>
    </div>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 3rem;
  }

  .page-header {
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--color-border);
  }

  .category-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.15em;
    color: var(--gray-400);
    margin-bottom: 1rem;
  }

  .category-badge::before {
    content: "";
    width: 6px;
    height: 6px;
    background: var(--accent);
    border-radius: 50%;
  }

  .page-title {
    font-family: var(--font-serif);
    font-size: clamp(2.5rem, 6vw, 3.5rem);
    font-weight: 400;
    letter-spacing: -0.02em;
    color: var(--white);
    margin-bottom: 0.75rem;
  }

  .page-desc {
    font-size: 1rem;
    color: var(--gray-500);
  }

  .edu-item {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1.5rem;
    padding: 2rem;
    background: var(--gray-950);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    transition: all var(--duration-fast) var(--ease-out);
  }

  .edu-item:hover {
    border-color: var(--color-border-hover);
  }

  @media (min-width: 640px) {
    .edu-item {
      grid-template-columns: 1fr auto;
      gap: 2rem;
      align-items: start;
    }
  }

  .edu-main {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .edu-school {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--white);
  }

  .edu-degree {
    font-size: 1rem;
    color: var(--gray-400);
  }

  .edu-meta {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    text-align: left;
  }

  @media (min-width: 640px) {
    .edu-meta {
      text-align: right;
    }
  }

  .edu-year {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--gray-300);
    font-variant-numeric: tabular-nums;
  }

  .edu-location {
    font-size: 0.8125rem;
    color: var(--gray-600);
  }

  .skills-section,
  .coursework-section {
    padding-top: 2rem;
  }

  .skills-title {
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--gray-500);
    text-transform: uppercase;
    letter-spacing: 0.15em;
    margin-bottom: 1.25rem;
  }

  .skills {
    display: flex;
    flex-wrap: wrap;
    gap: 0.625rem;
  }

  .skill {
    padding: 0.5rem 1rem;
    font-size: 0.8125rem;
    color: var(--gray-400);
    background: var(--gray-950);
    border: 1px solid var(--color-border);
    border-radius: 9999px;
    transition: all var(--duration-fast) var(--ease-out);
  }

  .skill:hover {
    color: var(--white);
    border-color: var(--color-border-hover);
    transform: translateY(-1px);
  }

  .coursework-grid {
    display: grid;
    grid-template-columns: repeat(1, 1fr);
    gap: 0.75rem;
  }

  @media (min-width: 480px) {
    .coursework-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  .course-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 1rem;
    background: var(--gray-950);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    transition: all var(--duration-fast) var(--ease-out);
  }

  .course-item:hover {
    border-color: var(--color-border-hover);
  }

  .course-code {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--gray-500);
    font-family: var(--font-mono);
  }

  .course-name {
    font-size: 0.9375rem;
    color: var(--white);
  }
</style>
