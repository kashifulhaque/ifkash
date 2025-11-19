<script>
  import "../app.css";
  import { onMount } from 'svelte';
  
  let sysTime = "";
  let coordinates = "SIGNAL_ACQUIRING...";

  onMount(() => {
    // System Clock
    const updateTime = () => {
      const now = new Date();
      sysTime = now.toLocaleTimeString('en-US', { 
        hour12: false, 
        hour: '2-digit', 
        minute: '2-digit', 
        second: '2-digit' 
      }); // HH:MM:SS in local time
    };
    updateTime();
    const clockInterval = setInterval(updateTime, 1000);

    // Mouse Tracking
    const handleMove = (/** @type {{ clientX: number; clientY: number; }} */ e) => {
      const x = (e.clientX / window.innerWidth).toFixed(3);
      const y = (e.clientY / window.innerHeight).toFixed(3);
      coordinates = `(${x}, ${y})`;
    };
    window.addEventListener('mousemove', handleMove);

    return () => {
      clearInterval(clockInterval);
      window.removeEventListener('mousemove', handleMove);
    };
  });
</script>

<div class="min-h-[100dvh] w-full bg-grid flex flex-col relative p-5 md:p-10 overflow-x-hidden">
  
  <div class="fixed top-3 left-3 md:top-6 md:left-6 w-6 h-6 md:w-10 md:h-10 border-l-[3px] border-t-[3px] border-[var(--matrix-accent)] z-50 pointer-events-none opacity-80"></div>
  <div class="fixed top-3 right-3 md:top-6 md:right-6 w-6 h-6 md:w-10 md:h-10 border-r-[3px] border-t-[3px] border-[var(--matrix-accent)] z-50 pointer-events-none opacity-80"></div>
  <div class="fixed bottom-3 left-3 md:bottom-6 md:left-6 w-6 h-6 md:w-10 md:h-10 border-l-[3px] border-b-[3px] border-[var(--matrix-accent)] z-50 pointer-events-none opacity-80"></div>
  <div class="fixed bottom-3 right-3 md:bottom-6 md:right-6 w-6 h-6 md:w-10 md:h-10 border-r-[3px] border-b-[3px] border-[var(--matrix-accent)] z-50 pointer-events-none opacity-80"></div>

  <div class="fixed top-1/2 left-1/2 w-[16px] h-[1px] bg-[var(--matrix-dim)] -translate-x-1/2 pointer-events-none z-0 opacity-50"></div>
  <div class="fixed top-1/2 left-1/2 w-[1px] h-[16px] bg-[var(--matrix-dim)] -translate-x-1/2 -translate-y-1/2 pointer-events-none z-0 opacity-50"></div>

  <header class="flex flex-col md:flex-row justify-between items-start md:items-end border-b border-[var(--matrix-dim)] pb-4 mb-8 bg-[var(--matrix-bg)] z-10 sticky top-0 pt-4">
    <div class="flex flex-col w-full md:w-auto">
      <div class="flex justify-between items-center mb-1">
        <span class="text-[10px] font-bold text-[var(--matrix-accent)] tracking-widest">TERMINAL_ID: IFKASH_001</span>
        <span class="md:hidden text-[10px] font-bold text-[var(--matrix-dim)] font-mono">{sysTime}</span>
      </div>
      <h1 class="text-2xl md:text-4xl font-black text-[var(--matrix-text)] tracking-tighter">
        ~/ROOT/KASHIF<span class="blink text-[var(--matrix-accent)]">_</span>
      </h1>
    </div>
    <div class="text-right hidden md:block">
      <div class="text-[10px] text-[var(--matrix-dim)] tracking-widest mb-1">SYS_STATUS: NORMAL</div>
      <div class="text-xl font-bold font-mono text-[var(--matrix-accent)]">{sysTime}</div>
    </div>
  </header>

  <nav class="grid grid-cols-2 md:flex gap-2 bg-transparent mb-12 z-10 text-sm font-bold">
    <a href="/" class="flex-1 text-center py-3 bg-[var(--matrix-grid)] text-[var(--matrix-dim)] hover:bg-[var(--matrix-accent)] hover:text-black transition-colors border border-[var(--matrix-dim)] hover:border-[var(--matrix-accent)]">
      [00] Home
    </a>
    <a href="/work" class="flex-1 text-center py-3 bg-[var(--matrix-grid)] text-[var(--matrix-dim)] hover:bg-[var(--matrix-accent)] hover:text-black transition-colors border border-[var(--matrix-dim)] hover:border-[var(--matrix-accent)]">
      [01] Work
    </a>
    <a href="/projects" class="flex-1 text-center py-3 bg-[var(--matrix-grid)] text-[var(--matrix-dim)] hover:bg-[var(--matrix-accent)] hover:text-black transition-colors border border-[var(--matrix-dim)] hover:border-[var(--matrix-accent)]">
      [02] Projects
    </a>
    <a href="/blog" class="flex-1 text-center py-3 bg-[var(--matrix-grid)] text-[var(--matrix-dim)] hover:bg-[var(--matrix-accent)] hover:text-black transition-colors border border-[var(--matrix-dim)] hover:border-[var(--matrix-accent)]">
      [03] Blogs
    </a>
  </nav>

  <main class="flex-grow z-10 max-w-6xl mx-auto w-full">
    <slot />
  </main>

  <footer class="mt-12 border-t border-[var(--matrix-dim)] pt-3 flex flex-col md:flex-row justify-between text-[10px] md:text-xs text-[var(--matrix-dim)] font-mono gap-4">
    <div class="flex flex-col md:flex-row gap-2 md:gap-4">
      <span>SECTOR: BENGALURU, INDIA</span>
      <span class="hidden md:inline text-[var(--matrix-accent)]">|</span>
      <span>{coordinates}</span>
    </div>
    <div class="flex gap-6 font-bold">
      <a href="https://github.com/kashifulhaque" class="hover:text-[var(--matrix-accent)]">:: GITHUB</a>
      <a href="https://linkedin.com/in/kashifulhaque" class="hover:text-[var(--matrix-accent)]">:: LINKEDIN</a>
      <a href="/assets/Kashiful_Haque.pdf" class="hover:text-[var(--matrix-accent)]">:: RESUME</a>
    </div>
  </footer>

</div>