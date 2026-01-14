<svelte:head>
  <title>Education — Kashif</title>
  <meta name="description" content="Academic background." />
</svelte:head>

<div class="page">
  <header class="page-header">
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
      {#each ['Machine Learning', 'Deep Learning', 'Statistics', 'Linear Algebra', 'Data Structures', 'Algorithms', 'Python', 'Distributed Systems'] as skill}
        <span class="skill">{skill}</span>
      {/each}
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
    border-bottom: 1px solid var(--gray-800);
  }
  
  .page-title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--white);
    margin-bottom: 0.5rem;
  }
  
  .page-desc {
    font-size: 1rem;
    color: var(--gray-500);
  }

  .edu-item {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
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
    gap: 0.375rem;
  }
  
  .edu-school {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--white);
  }
  
  .edu-degree {
    font-size: 0.9375rem;
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
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--gray-300);
    font-variant-numeric: tabular-nums;
  }
  
  .edu-location {
    font-size: 0.75rem;
    color: var(--gray-600);
  }

  .skills-section {
    border-top: 1px solid var(--gray-800);
    padding-top: 2rem;
  }
  
  .skills-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--gray-600);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 1rem;
  }
  
  .skills {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  
  .skill {
    padding: 0.5rem 0.875rem;
    font-size: 0.8125rem;
    color: var(--gray-400);
    background: var(--gray-900);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-sm);
    transition: all var(--duration-fast) var(--ease-out);
  }
  
  .skill:hover {
    color: var(--white);
    border-color: var(--gray-700);
  }
</style>
