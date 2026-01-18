<script>
  import { auth } from "$lib/stores/auth";
  import { dev } from "$app/environment";

  const resumeUrl = dev
    ? "http://localhost:8787/api/resume?format=view"
    : "/api/resume?format=view";
</script>

<svelte:head>
  <title>Kashiful Haque — ML Engineer</title>
  <meta
    name="description"
    content="Building intelligent systems at the intersection of research and production. ML Engineer focused on LLMs, inference optimization, and scalable AI infrastructure."
  />
</svelte:head>

<!-- Hero Section -->
<section class="hero">
  <div class="hero-bg">
    <img src="/images/hero-bg.png" alt="" loading="eager" />
  </div>

  <div class="hero-content">
    <a
      href="https://wand.ai"
      target="_blank"
      rel="noopener noreferrer"
      class="announcement-badge"
    >
      ML Engineer @ Wand AI
    </a>

    <h1 class="hero-title">
      Building intelligent<br />systems that scale.
    </h1>

    <p class="hero-subtitle">
      ML Engineer focused on LLMs, inference optimization, and production AI
      infrastructure
    </p>

    <a
      href="https://linkedin.com/in/kashifulhaque"
      target="_blank"
      rel="noopener noreferrer"
      class="btn btn-primary"
    >
      Get in touch
    </a>
  </div>

  <div class="hero-logos">
    <a
      href="https://github.com/kashifulhaque"
      target="_blank"
      rel="noopener noreferrer"
      class="hero-logo-item"
    >
      <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
        <path
          d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"
        />
      </svg>
    </a>
    <a
      href="https://hf.co/ifkash"
      target="_blank"
      rel="noopener noreferrer"
      class="hero-logo-item"
    >
      <svg width="28" height="24" viewBox="0 0 120 120" fill="currentColor">
        <path
          fill-rule="evenodd"
          d="M49.5 23c-5.5 0-10 4.5-10 10v21c0 2.8-2.2 5-5 5s-5-2.2-5-5V33c0-11 9-20 20-20s20 9 20 20v21c0 2.8-2.2 5-5 5s-5-2.2-5-5V33c0-5.5-4.5-10-10-10zM80.5 23c-5.5 0-10 4.5-10 10v21c0 2.8 2.2 5 5 5s5-2.2 5-5V33c0-5.5 4.5-10 10-10s10 4.5 10 10v21c0 11-9 20-20 20s-20-9-20-20V33c0-5.5 4.5-10 10-10z"
        />
        <circle cx="45" cy="85" r="10" />
        <circle cx="75" cy="85" r="10" />
      </svg>
    </a>
    <a
      href="https://linkedin.com/in/kashifulhaque"
      target="_blank"
      rel="noopener noreferrer"
      class="hero-logo-item"
    >
      <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
        <path
          d="M19 0h-14c-2.761 0-5 2.239-5 5v14c0 2.761 2.239 5 5 5h14c2.762 0 5-2.239 5-5v-14c0-2.761-2.238-5-5-5zm-11 19h-3v-11h3v11zm-1.5-12.268c-.966 0-1.75-.79-1.75-1.764s.784-1.764 1.75-1.764 1.75.79 1.75 1.764-.783 1.764-1.75 1.764zm13.5 12.268h-3v-5.604c0-3.368-4-3.113-4 0v5.604h-3v-11h3v1.765c1.396-2.586 7-2.777 7 2.476v6.759z"
        />
      </svg>
    </a>
  </div>
</section>

<!-- Stats Section -->
<section class="stats-section">
  <div class="container">
    <div class="stats-row">
      <p class="stats-intro">
        Solving complex ML challenges with AI, shipping production systems in
        record time.
      </p>
      <div class="stats-grid">
        <div class="stat-item">
          <span class="stat-label">Projects Shipped</span>
          <span class="stat-value">10+</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Tokens Trained</span>
          <span class="stat-value">6B</span>
        </div>
      </div>
    </div>
  </div>
</section>

<!-- Focus Areas Section -->
<section class="focus-section">
  <div class="container">
    <div class="section-header">
      <span class="category-badge">Focus Areas</span>
      <h2 class="section-title">Built to handle<br />complexity</h2>
    </div>

    <div class="features-grid">
      <div class="feature-card">
        <div class="feature-icon">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
          </svg>
        </div>
        <h3 class="feature-title">LLMs & Inference</h3>
        <p class="feature-desc">
          Fine-tuning, training, and optimizing large language models for
          production workloads
        </p>
      </div>

      <div class="feature-card">
        <div class="feature-icon">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
            <line x1="8" y1="21" x2="16" y2="21" />
            <line x1="12" y1="17" x2="12" y2="21" />
          </svg>
        </div>
        <h3 class="feature-title">Production ML</h3>
        <p class="feature-desc">
          Building scalable ML infrastructure that handles real-world traffic
          reliably
        </p>
      </div>

      <div class="feature-card">
        <div class="feature-icon">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"
            />
          </svg>
        </div>
        <h3 class="feature-title">Open Source</h3>
        <p class="feature-desc">
          Contributing models, tools, and datasets to advance the ML ecosystem
        </p>
      </div>
    </div>
  </div>
</section>

<!-- Featured Work Section -->
<section class="work-preview-section">
  <div class="container">
    <div class="content-grid">
      <div class="content-text">
        <span class="category-badge">Featured Work</span>
        <h2 class="section-title-sm">smol-llama</h2>
        <p class="content-desc">
          360M parameter LLaMA model trained from scratch on 6B tokens. Features
          GQA, RoPE, and SwiGLU. Trained on a single H100 in 22 hours for $53.
        </p>
        <div class="content-actions">
          <a
            href="https://huggingface.co/ifkash/smol-llama"
            target="_blank"
            rel="noopener noreferrer"
            class="btn btn-outline"
          >
            View Model
          </a>
          <a href="/projects" class="btn btn-ghost"> All Projects → </a>
        </div>
      </div>

      <div class="work-steps">
        <div class="work-step">
          <span class="step-number">1</span>
          <div class="step-content">
            <h4>Pre-train from scratch</h4>
            <p>Custom tokenizer with 49K vocab, efficient data pipeline</p>
          </div>
        </div>
        <div class="work-step">
          <span class="step-number">2</span>
          <div class="step-content">
            <h4>Optimize for inference</h4>
            <p>Flash attention, KV cache, and memory optimizations</p>
          </div>
        </div>
        <div class="work-step">
          <span class="step-number">3</span>
          <div class="step-content">
            <h4>Ship to production</h4>
            <p>Model weights on Hugging Face, ready for deployment</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</section>

<!-- CTA Section -->
<section class="cta-section">
  <div class="container">
    <div class="cta-content">
      <h2 class="cta-title">Let's build something together.</h2>
      <p class="cta-desc">
        Currently open to interesting ML challenges and collaborations.
      </p>
      <div class="cta-actions">
        <a
          href="https://linkedin.com/in/kashifulhaque"
          target="_blank"
          rel="noopener noreferrer"
          class="btn btn-primary"
        >
          Get in touch
        </a>
        <a href={resumeUrl} class="btn btn-outline"> View Resume </a>
      </div>
    </div>
  </div>
</section>

<!-- Footer for home page -->
<footer class="home-footer">
  <div class="container">
    <div class="footer-inner">
      <span class="footer-text"
        >© {new Date().getFullYear()} Kashiful Haque</span
      >
      <div class="footer-links">
        <a
          href="https://github.com/kashifulhaque"
          target="_blank"
          rel="noopener noreferrer">GitHub</a
        >
        <span class="footer-sep">·</span>
        <a href="https://hf.co/ifkash" target="_blank" rel="noopener noreferrer"
          >Hugging Face</a
        >
        <span class="footer-sep">·</span>
        <a
          href="https://linkedin.com/in/kashifulhaque"
          target="_blank"
          rel="noopener noreferrer">LinkedIn</a
        >
      </div>
    </div>
  </div>
</footer>

<style>
  /* ═══════════════════════════════════════════════════════════════════════
     HERO SECTION
     ═══════════════════════════════════════════════════════════════════════ */

  .hero {
    position: relative;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    overflow: hidden;
  }

  .hero-bg {
    position: absolute;
    inset: 0;
    z-index: 0;
  }

  .hero-bg img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center;
  }

  .hero-bg::after {
    content: "";
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to bottom,
      rgba(13, 13, 13, 0.4) 0%,
      rgba(13, 13, 13, 0.1) 30%,
      rgba(13, 13, 13, 0.3) 70%,
      rgba(13, 13, 13, 1) 100%
    );
  }

  .hero-content {
    position: relative;
    z-index: 1;
    max-width: 900px;
    padding: 0 2rem;
    animation: fade-up 0.8s ease-out;
  }

  .announcement-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--gray-200);
    backdrop-filter: blur(8px);
    margin-bottom: 2rem;
    transition: all 0.2s ease;
  }

  .announcement-badge::before {
    content: "";
    width: 6px;
    height: 6px;
    background: var(--accent);
    border-radius: 50%;
  }

  .announcement-badge::after {
    content: "→";
    color: var(--gray-400);
  }

  .announcement-badge:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .hero-title {
    font-family: var(--font-serif);
    font-size: clamp(2.5rem, 7vw, 4.5rem);
    font-weight: 400;
    color: var(--white);
    line-height: 1.1;
    margin-bottom: 1.5rem;
    text-shadow: 0 2px 40px rgba(0, 0, 0, 0.3);
  }

  .hero-subtitle {
    font-size: 1.125rem;
    color: var(--gray-300);
    max-width: 480px;
    margin: 0 auto 2.5rem;
    text-shadow: 0 1px 20px rgba(0, 0, 0, 0.5);
  }

  .hero-logos {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2rem;
    position: absolute;
    bottom: 3rem;
    left: 0;
    right: 0;
  }

  .hero-logo-item {
    color: var(--gray-400);
    opacity: 0.6;
    transition: all 0.2s ease;
  }

  .hero-logo-item:hover {
    color: var(--white);
    opacity: 1;
    transform: translateY(-2px);
  }

  @keyframes fade-up {
    from {
      opacity: 0;
      transform: translateY(30px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* ═══════════════════════════════════════════════════════════════════════
     STATS SECTION
     ═══════════════════════════════════════════════════════════════════════ */

  .stats-section {
    padding: 5rem 0;
    background: var(--black);
    border-bottom: 1px solid var(--color-border);
  }

  .stats-row {
    display: flex;
    flex-direction: column;
    gap: 3rem;
  }

  @media (min-width: 768px) {
    .stats-row {
      flex-direction: row;
      justify-content: space-between;
      align-items: flex-start;
    }
  }

  .stats-intro {
    font-size: 1rem;
    color: var(--gray-400);
    max-width: 320px;
    line-height: 1.6;
  }

  .stats-grid {
    display: flex;
    gap: 4rem;
  }

  .stat-item {
    text-align: left;
  }

  .stat-label {
    display: block;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.15em;
    color: var(--gray-500);
    margin-bottom: 0.5rem;
  }

  .stat-value {
    font-family: var(--font-serif);
    font-size: 3rem;
    font-weight: 400;
    color: var(--white);
    line-height: 1;
  }

  /* ═══════════════════════════════════════════════════════════════════════
     FOCUS SECTION
     ═══════════════════════════════════════════════════════════════════════ */

  .focus-section {
    padding: 6rem 0;
    background: var(--black);
  }

  .section-header {
    margin-bottom: 4rem;
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

  .section-title {
    font-family: var(--font-serif);
    font-size: clamp(2rem, 5vw, 3.5rem);
    font-weight: 400;
    color: var(--white);
    line-height: 1.1;
  }

  .features-grid {
    display: grid;
    grid-template-columns: repeat(1, 1fr);
    gap: 2rem;
  }

  @media (min-width: 768px) {
    .features-grid {
      grid-template-columns: repeat(3, 1fr);
      gap: 3rem;
    }
  }

  .feature-card {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1.5rem 0;
    border-top: 1px solid var(--color-border);
  }

  .feature-icon {
    color: var(--gray-400);
    margin-bottom: 0.5rem;
  }

  .feature-title {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--white);
  }

  .feature-desc {
    font-size: 0.875rem;
    color: var(--gray-500);
    line-height: 1.6;
  }

  /* ═══════════════════════════════════════════════════════════════════════
     WORK PREVIEW SECTION
     ═══════════════════════════════════════════════════════════════════════ */

  .work-preview-section {
    padding: 6rem 0;
    background: var(--gray-950);
  }

  .content-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 3rem;
  }

  @media (min-width: 1024px) {
    .content-grid {
      grid-template-columns: 1fr 1fr;
      gap: 4rem;
      align-items: start;
    }
  }

  .content-text {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .section-title-sm {
    font-family: var(--font-serif);
    font-size: 2.5rem;
    font-weight: 400;
    color: var(--white);
    line-height: 1.1;
  }

  .content-desc {
    font-size: 1rem;
    color: var(--gray-400);
    line-height: 1.7;
    max-width: 400px;
  }

  .content-actions {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .work-steps {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .work-step {
    display: flex;
    gap: 1rem;
    padding: 1.5rem 0;
    border-bottom: 1px solid var(--color-border);
  }

  .work-step:first-child {
    border-top: 1px solid var(--color-border);
  }

  .step-number {
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--gray-400);
    background: var(--gray-900);
    border-radius: 50%;
  }

  .step-content h4 {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--white);
    margin-bottom: 0.25rem;
  }

  .step-content p {
    font-size: 0.8125rem;
    color: var(--gray-500);
  }

  /* ═══════════════════════════════════════════════════════════════════════
     CTA SECTION
     ═══════════════════════════════════════════════════════════════════════ */

  .cta-section {
    padding: 6rem 0;
    background: var(--black);
    text-align: center;
  }

  .cta-content {
    max-width: 500px;
    margin: 0 auto;
  }

  .cta-title {
    font-family: var(--font-serif);
    font-size: clamp(1.75rem, 4vw, 2.5rem);
    font-weight: 400;
    color: var(--white);
    margin-bottom: 1rem;
  }

  .cta-desc {
    font-size: 1rem;
    color: var(--gray-400);
    margin-bottom: 2rem;
  }

  .cta-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
    flex-wrap: wrap;
  }

  /* ═══════════════════════════════════════════════════════════════════════
     HOME FOOTER
     ═══════════════════════════════════════════════════════════════════════ */

  .home-footer {
    padding: 2rem 0;
    background: var(--black);
    border-top: 1px solid var(--color-border);
  }

  .footer-inner {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: center;
    text-align: center;
  }

  @media (min-width: 640px) {
    .footer-inner {
      flex-direction: row;
      justify-content: space-between;
    }
  }

  .footer-text {
    font-size: 0.8125rem;
    color: var(--gray-600);
  }

  .footer-links {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.8125rem;
  }

  .footer-links a {
    color: var(--gray-500);
    transition: color 0.15s ease;
  }

  .footer-links a:hover {
    color: var(--white);
  }

  .footer-sep {
    color: var(--gray-700);
  }

</style>
