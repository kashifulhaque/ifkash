<script>
  import { auth } from "$lib/stores/auth";
  import { dev } from "$app/environment";

  const resumeUrl = dev
    ? "http://localhost:8787/api/resume?format=view"
    : "/api/resume?format=view";
</script>

<svelte:head>
  <title>Kashiful Haque</title>
  <meta name="description" content="Personal website of Kashiful Haque, ML Engineer." />
</svelte:head>

<div class="home">
  <!-- Hero -->
  <section class="hero">
    <h1 class="hero-title">
      Kashiful Haque
    </h1>
    <p class="hero-subtitle">
      Building intelligent systems at the intersection of research and production. 
      Focused on inference optimization, model training, and scalable AI infrastructure.
    </p>
  </section>

  <!-- Links -->
  <section class="links-section">
    <a href={resumeUrl} class="link-card">
      <span class="link-label">Resume</span>
      <span class="link-arrow">↗</span>
    </a>
    
    {#if $auth.isAuthenticated}
      <a href="/editor" class="link-card">
        <span class="link-label">Editor</span>
        <span class="link-arrow">→</span>
      </a>
    {/if}
  </section>

  <!-- Currently -->
  <section class="status-section">
    <div class="status-row">
      <span class="status-label">Currently</span>
      <span class="status-value">ML Engineer @ Wand AI</span>
    </div>
    <div class="status-row">
      <span class="status-label">Location</span>
      <span class="status-value">Palo Alto, CA</span>
    </div>
    <div class="status-row">
      <span class="status-label">Focus</span>
      <span class="status-value">LLMs, Inference, Production ML</span>
    </div>
  </section>
</div>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: 4rem;
  }

  /* Hero */
  .hero {
    padding-top: 2rem;
  }
  
  .hero-title {
    font-size: clamp(2.5rem, 8vw, 4rem);
    font-weight: 700;
    letter-spacing: -0.03em;
    line-height: 1;
    color: var(--white);
    margin-bottom: 1.5rem;
  }
  
  .hero-subtitle {
    font-size: 1.125rem;
    line-height: 1.7;
    color: var(--gray-400);
    max-width: 540px;
  }

  /* Links */
  .links-section {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--gray-800);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  
  .link-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem;
    background: var(--gray-950);
    transition: background var(--duration-fast) var(--ease-out);
  }
  
  .link-card:hover {
    background: var(--gray-900);
  }
  
  .link-label {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--white);
  }
  
  .link-arrow {
    font-size: 1rem;
    color: var(--gray-600);
    transition: all var(--duration-fast) var(--ease-out);
  }
  
  .link-card:hover .link-arrow {
    color: var(--white);
    transform: translateX(4px);
  }

  /* Status */
  .status-section {
    display: flex;
    flex-direction: column;
    gap: 0;
    border-top: 1px solid var(--gray-800);
    padding-top: 2rem;
  }
  
  .status-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 0.875rem 0;
    border-bottom: 1px solid var(--gray-900);
  }
  
  .status-row:last-child {
    border-bottom: none;
  }
  
  .status-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--gray-600);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  
  .status-value {
    font-size: 0.9375rem;
    color: var(--gray-300);
  }
</style>
