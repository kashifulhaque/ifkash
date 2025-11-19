<script>
  // Data visualization logic
</script>

<svelte:head>
  <title>SYSTEM_ROOT // KASHIF</title>
  <meta name="description" content="System Interface for Kashif. MLE at Wand AI.">
</svelte:head>

<section class="grid grid-cols-1 lg:grid-cols-12 gap-4 mb-12">
  
  <div class="lg:col-span-5 border border-[var(--matrix-dim)] bg-[rgba(2,6,23,0.8)] flex flex-col items-center justify-center p-8 relative overflow-hidden">
    <div class="absolute inset-0 bg-[linear-gradient(transparent_50%,rgba(6,182,212,0.05)_50%)] bg-[length:100%_4px] pointer-events-none"></div>
    
    <pre class="text-[9px] sm:text-[11px] leading-[10px] sm:leading-[12px] text-[var(--matrix-accent)] font-black select-none text-center z-10">
      ▄▄▄█████▓ ██▓███   ▄▄▄       ██▓    
      ▓  ██▒ ▓▒▓██░  ██▒▒████▄    ▓██▒    
      ▒ ▓██░ ▒░▓██░ ██▓▒▒██  ▀█▄  ▒██░    
      ░ ▓██▓ ░ ▒██▄█▓▒ ▒░██▄▄▄▄██ ▒██░    
        ▒██▒ ░ ▒██▒ ░  ░ ▓█   ▓██▒░██████▒
        ▒ ░░   ▒▓▒░ ░  ░ ▒▒   ▓▒█░░ ▒░▓  ░
          ░    ░▒ ░       ▒   ▒▒ ░░ ░ ▒  ░
        ░      ░░         ░   ▒     ░ ░   
               ░              ░  ░    ░  ░
    </pre>
    
    <div class="mt-8 text-center z-10 border-t border-[var(--matrix-dim)] pt-4 w-full">
      <div class="text-xs text-[var(--matrix-text)] font-bold tracking-widest mb-1">CLASS: APPLIED ML ENGINEER</div>
      <div class="text-xs text-[var(--matrix-accent)] animate-pulse">STATUS: OPERATIONAL</div>
    </div>
  </div>

  <div class="lg:col-span-7 bg-[rgba(2,6,23,0.8)] border border-[var(--matrix-dim)] p-6 md:p-8 flex flex-col justify-between">
    <div>
      <h2 class="text-xl font-black text-[var(--matrix-text)] mb-6 flex items-center gap-3">
        <span class="w-3 h-3 bg-[var(--matrix-accent)]"></span>
        PERSONNEL_FILE_LOADED
      </h2>
      
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-8 gap-y-4 text-sm font-mono mb-8">
        <div class="flex flex-col">
          <span class="text-[var(--matrix-dim)] text-[10px] mb-1">SUBJECT_NAME</span>
          <span class="text-[var(--matrix-text)] font-bold border-l-2 border-[var(--matrix-dim)] pl-3">KASHIFUL HAQUE</span>
        </div>
        <div class="flex flex-col">
          <span class="text-[var(--matrix-dim)] text-[10px] mb-1">CURRENT_ASSIGNMENT</span>
          <span class="text-[var(--matrix-text)] font-bold border-l-2 border-[var(--matrix-dim)] pl-3">WAND AI (PALO ALTO)</span>
        </div>
        <div class="flex flex-col">
          <span class="text-[var(--matrix-dim)] text-[10px] mb-1">PRIMARY_STACK</span>
          <span class="text-[var(--matrix-text)] font-bold border-l-2 border-[var(--matrix-dim)] pl-3">RUST, PYTHON, CUDA</span>
        </div>
        <div class="flex flex-col">
          <span class="text-[var(--matrix-dim)] text-[10px] mb-1">ORIGIN_DATA</span>
          <span class="text-[var(--matrix-text)] font-bold border-l-2 border-[var(--matrix-dim)] pl-3">IIT MADRAS</span>
        </div>
      </div>
    </div>

    <div class="border border-[var(--matrix-dim)] bg-[var(--matrix-grid)] p-4">
      <div class="flex justify-between items-center mb-2 border-b border-[var(--matrix-dim)] pb-2">
        <span class="text-[10px] font-bold text-[var(--matrix-accent)]">BIO_SUMMARY.TXT</span>
        <span class="text-[10px] text-[var(--matrix-dim)]">READ_ONLY</span>
      </div>
      <p class="text-sm md:text-base leading-relaxed text-[var(--matrix-text)]">
        I design robust systems and ship thoughtful tools. Currently optimizing LLM inference and bridging models with production infrastructure.
      </p>
    </div>
  </div>
</section>

<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
  
  <a href="/work" class="group block bg-[rgba(2,6,23,0.6)] border border-[var(--matrix-dim)] p-5 hover:border-[var(--matrix-accent)] transition-all hover:bg-[rgba(6,182,212,0.05)] relative overflow-hidden">
    <div class="absolute top-0 right-0 bg-[var(--matrix-dim)] text-[var(--matrix-bg)] text-[9px] font-bold px-1 py-0.5 group-hover:bg-[var(--matrix-accent)] transition-colors">
      DIR_01
    </div>
    <h3 class="text-xl font-black mb-3 text-[var(--matrix-text)] group-hover:text-[var(--matrix-accent)]">
      WORK_EXP
    </h3>
    <ul class="text-xs text-[var(--matrix-dim)] space-y-1 font-mono">
      <li>> LOAD WAND_AI...</li>
      <li>> LOAD AMEX...</li>
      <li>> LOAD FIERY...</li>
    </ul>
  </a>

  <a href="/projects" class="group block bg-[rgba(2,6,23,0.6)] border border-[var(--matrix-dim)] p-5 hover:border-[var(--matrix-accent)] transition-all hover:bg-[rgba(6,182,212,0.05)] relative overflow-hidden">
    <div class="absolute top-0 right-0 bg-[var(--matrix-dim)] text-[var(--matrix-bg)] text-[9px] font-bold px-1 py-0.5 group-hover:bg-[var(--matrix-accent)] transition-colors">
      DIR_02
    </div>
    <h3 class="text-xl font-black mb-3 text-[var(--matrix-text)] group-hover:text-[var(--matrix-accent)]">
      PROJECTS
    </h3>
    <ul class="text-xs text-[var(--matrix-dim)] space-y-1 font-mono">
      <li>> BOO_BOT [ACTIVE]</li>
      <li>> NOPOKEDB [STABLE]</li>
      <li>> SMOLTORCH [ARCHIVE]</li>
    </ul>
  </a>

  <a href="/leetcode" class="group block bg-[rgba(2,6,23,0.6)] border border-[var(--matrix-dim)] p-5 hover:border-[var(--matrix-accent)] transition-all hover:bg-[rgba(6,182,212,0.05)] relative overflow-hidden">
    <div class="absolute top-0 right-0 bg-[var(--matrix-dim)] text-[var(--matrix-bg)] text-[9px] font-bold px-1 py-0.5 group-hover:bg-[var(--matrix-accent)] transition-colors">
      DIR_03
    </div>
    <h3 class="text-xl font-black mb-3 text-[var(--matrix-text)] group-hover:text-[var(--matrix-accent)]">
      METRICS
    </h3>
    <ul class="text-xs text-[var(--matrix-dim)] space-y-1 font-mono">
      <li>> LEETCODE STATS</li>
      <li>> RECENT SOLVES</li>
      <li>> PROBLEM BREAKDOWN</li>
    </ul>
  </a>

  <a href="/news" class="group block bg-[rgba(2,6,23,0.6)] border border-[var(--matrix-dim)] p-5 hover:border-[var(--matrix-accent)] transition-all hover:bg-[rgba(6,182,212,0.05)] relative overflow-hidden">
    <div class="absolute top-0 right-0 bg-[var(--matrix-dim)] text-[var(--matrix-bg)] text-[9px] font-bold px-1 py-0.5 group-hover:bg-[var(--matrix-accent)] transition-colors">
      DIR_04
    </div>
    <h3 class="text-xl font-black mb-3 text-[var(--matrix-text)] group-hover:text-[var(--matrix-accent)]">
      UPLINK
    </h3>
    <ul class="text-xs text-[var(--matrix-dim)] space-y-1 font-mono">
      <li>> HACKERNEWS FEED</li>
      <li>> LATEST HEADLINES</li>
      <li>> SYNC COMPLETE</li>
    </ul>
  </a>

</div>

<div class="mt-12 pt-4 border-t border-[var(--matrix-dim)]">
  <div class="flex justify-between items-end h-10 gap-1 opacity-50">
    {#each Array(32) as _, i}
      <div 
        class="bg-[var(--matrix-accent)] w-full transition-all duration-500 ease-in-out"
        style="height: {10 + Math.random() * 60}%; opacity: {0.2 + Math.random() * 0.8}"
      ></div>
    {/each}
  </div>
  <div class="flex justify-between text-[9px] text-[var(--matrix-dim)] mt-1 font-mono">
    <span>CORE_ACTIVITY</span>
    <span>BUFFER_USAGE: 14%</span>
  </div>
</div>