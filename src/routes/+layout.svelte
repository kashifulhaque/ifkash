<script>
  import "../app.css";
  import { onMount } from 'svelte';
  
  let sysTime = "";
  let coordinates = "WAITING_SAT_LINK...";

  onMount(() => {
    // System Clock
    const updateTime = () => {
      const now = new Date();
      sysTime = now.toISOString().split('T')[1].split('.')[0]; // HH:MM:SS
    };
    updateTime();
    const clockInterval = setInterval(updateTime, 1000);

    // Mouse Tracking
    const handleMove = (e) => {
      const x = (e.clientX / window.innerWidth).toFixed(3);
      const y = (e.clientY / window.innerHeight).toFixed(3);
      coordinates = `X:${x} // Y:${y}`;
    };
    window.addEventListener('mousemove', handleMove);

    return () => {
      clearInterval(clockInterval);
      window.removeEventListener('mousemove', handleMove);
    };
  });
</script>

<div class="min-h-[100dvh] w-full bg-grid flex flex-col relative p-4 md:p-8 lg:p-12 border-0 md:border-[12px] border-transparent overflow-x-hidden">
  
  <div class="fixed top-2 left-2 md:top-4 md:left-4 w-4 h-4 md:w-8 md:h-8 border-l-2 border-t-2 border-[var(--matrix-accent)] z-50 pointer-events-none"></div>
  <div class="fixed top-2 right-2 md:top-4 md:right-4 w-4 h-4 md:w-8 md:h-8 border-r-2 border-t-2 border-[var(--matrix-accent)] z-50 pointer-events-none"></div>
  <div class="fixed bottom-2 left-2 md:bottom-4 md:left-4 w-4 h-4 md:w-8 md:h-8 border-l-2 border-b-2 border-[var(--matrix-accent)] z-50 pointer-events-none"></div>
  <div class="fixed bottom-2 right-2 md:bottom-4 md:right-4 w-4 h-4 md:w-8 md:h-8 border-r-2 border-b-2 border-[var(--matrix-accent)] z-50 pointer-events-none"></div>

  <div class="fixed top-1/2 left-1/2 w-[10px] md:w-[20px] h-[1px] bg-[var(--matrix-dim)] -translate-x-1/2 pointer-events-none z-0"></div>
  <div class="fixed top-1/2 left-1/2 w-[1px] h-[10px] md:h-[20px] bg-[var(--matrix-dim)] -translate-x-1/2 -translate-y-1/2 pointer-events-none z-0"></div>

  <header class="flex flex-col md:flex-row justify-between items-start md:items-end border-b border-[var(--matrix-accent)] pb-2 mb-6 md:mb-8 bg-[var(--matrix-bg)] z-10 sticky top-0 pt-4 md:pt-0">
    <div class="flex flex-col w-full md:w-auto">
      <div class="flex justify-between items-center">
        <span class="text-[10px] md:text-xs text-[var(--matrix-dim)]">ID: IFKASH-DEV-001</span>
        <span class="md:hidden text-[10px] font-bold text-[var(--matrix-accent)] font-mono">{sysTime}</span>
      </div>
      <h1 class="text-lg md:text-3xl font-bold text-[var(--matrix-accent)] mt-1 truncate">
        ~/ROOT/USER/KASHIF<span class="blink">_</span>
      </h1>
    </div>
    <div class="text-right hidden md:block">
      <div class="text-xs text-[var(--matrix-dim)]">SYSTEM STATUS: ONLINE</div>
      <div class="text-sm font-bold font-mono text-[var(--matrix-accent)]">{sysTime}</div>
    </div>
  </header>

  <nav class="grid grid-cols-2 md:flex gap-px bg-[var(--matrix-dim)] border border-[var(--matrix-dim)] mb-8 md:mb-12 z-10">
    <a href="/" class="flex-1 text-center py-3 md:py-3 bg-[var(--matrix-bg)] hover:bg-[var(--matrix-accent)] hover:text-black transition-none border-b md:border-b-0 md:border-r border-[var(--matrix-dim)] text-sm md:text-base">
      [00] HOME
    </a>
    <a href="/work" class="flex-1 text-center py-3 md:py-3 bg-[var(--matrix-bg)] hover:bg-[var(--matrix-accent)] hover:text-black transition-none border-b md:border-b-0 md:border-r border-[var(--matrix-dim)] text-sm md:text-base">
      [01] WRK_LOG
    </a>
    <a href="/projects" class="flex-1 text-center py-3 md:py-3 bg-[var(--matrix-bg)] hover:bg-[var(--matrix-accent)] hover:text-black transition-none border-r border-[var(--matrix-dim)] text-sm md:text-base">
      [02] PROJ_DB
    </a>
    <a href="/blog" class="flex-1 text-center py-3 md:py-3 bg-[var(--matrix-bg)] hover:bg-[var(--matrix-accent)] hover:text-black transition-none text-sm md:text-base">
      [03] DATA_DUMP
    </a>
  </nav>

  <main class="flex-grow z-10">
    <slot />
  </main>

  <footer class="mt-8 md:mt-12 border-t border-[var(--matrix-dim)] pt-2 flex flex-col md:flex-row justify-between text-[10px] md:text-xs text-[var(--matrix-dim)] font-mono gap-2 md:gap-0">
    <div class="flex justify-between md:block">
      <span>LOC: BENGALURU, IN</span>
      <span class="hidden md:inline mx-2">|</span>
      <span>{coordinates}</span>
    </div>
    <div class="flex gap-4 flex-wrap">
      <a href="https://github.com/kashifulhaque" class="hover:text-[var(--matrix-accent)]">GH_REPO</a>
      <a href="https://linkedin.com/in/kashifulhaque" class="hover:text-[var(--matrix-accent)]">LNKD_NET</a>
      <a href="/assets/Kashiful_Haque.pdf" class="hover:text-[var(--matrix-accent)]">DOWNLOAD_CV.PDF</a>
    </div>
  </footer>

</div>