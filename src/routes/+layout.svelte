<script>
  import "../app.css";
  import { page } from '$app/stores';
</script>

<div class="min-h-screen flex flex-col max-w-3xl mx-auto px-6 py-8 md:py-12">
  
  <header class="flex flex-col md:flex-row justify-between items-start md:items-center mb-16 gap-4">
    <a href="/" class="text-xl font-bold tracking-tight text-[var(--color-headline)] hover:text-[var(--color-highlight)] transition-colors">
      ifkash.dev
    </a>

    <nav class="flex gap-6 text-sm font-medium">
      <a href="/work" class:active={$page.url.pathname.startsWith('/work')} class="nav-link">Work</a>
      <a href="/projects" class:active={$page.url.pathname.startsWith('/projects')} class="nav-link">Projects</a>
      <a href="/blog" class:active={$page.url.pathname.startsWith('/blog')} class="nav-link">Blog</a>
    </nav>
  </header>

  <main class="flex-grow">
    <slot />
  </main>

  <footer class="mt-24 pt-8 border-t border-[var(--color-border)] flex flex-col md:flex-row justify-between items-center text-xs text-[var(--color-secondary)] gap-4">
    <div class="flex gap-4">
      <span>&copy; {new Date().getFullYear()} Kashiful Haque</span>
    </div>
    <div class="flex gap-6 font-medium">
      <a href="https://github.com/kashifulhaque" target="_blank" rel="noopener noreferrer" class="hover:text-[var(--color-headline)] transition-colors">GitHub</a>
      <a href="https://linkedin.com/in/kashifulhaque" target="_blank" rel="noopener noreferrer" class="hover:text-[var(--color-headline)] transition-colors">LinkedIn</a>
      <a href="/assets/Kashiful_Haque.pdf" target="_blank" rel="noopener noreferrer" class="hover:text-[var(--color-headline)] transition-colors">Resume</a>
    </div>
  </footer>

</div>

<style>
  .nav-link {
    color: var(--color-paragraph);
    transition: color 0.2s ease;
  }
  .nav-link:hover, .nav-link.active {
    color: var(--color-headline);
  }
</style>
