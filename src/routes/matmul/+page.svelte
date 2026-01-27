<svelte:head>
  <title>Matrix Multiplication Visualizer — Kashif</title>
  <meta name="description" content="Understanding why loop order matters for cache performance in matrix multiplication." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';

  let currentSize = 4;
  let animationSpeed = 5;
  let animationRunning = false;
  let stepMode = false;
  let stepResolve: (() => void) | null = null;
  let progressPercent = 0;

  let stats = { totalOps: 0, naiveMisses: 0, optMisses: 0 };
  let loopOrder: 'naive' | 'optimized' | 'both' = 'naive';
  let visualizationArea: HTMLElement;

  const speedLabels = ['Very Slow', 'Slower', 'Slow', 'Medium-Slow', 'Normal', 'Medium-Fast', 'Fast', 'Faster', 'Very Fast', 'Instant'];

  function getDelay(): number {
    const baseDelay = 300;
    return Math.max(10, baseDelay - (animationSpeed - 1) * 30);
  }

  function sleep(ms: number): Promise<void> {
    if (stepMode) {
      return new Promise(resolve => { stepResolve = resolve; });
    }
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  function renderVisualization() {
    if (!visualizationArea) return;
    visualizationArea.innerHTML = '';

    if (loopOrder === 'both') {
      visualizationArea.innerHTML = `
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 24px;">
          <div id="naiveViz"></div>
          <div id="optimizedViz"></div>
        </div>
      `;
      renderSingleViz('naiveViz', 'Naive (row→col→inner)', 'naive');
      renderSingleViz('optimizedViz', 'Optimized (row→inner→col)', 'optimized');
    } else {
      visualizationArea.innerHTML = `<div id="singleViz"></div>`;
      const label = loopOrder === 'naive' ? 'Naive (row→col→inner)' : 'Optimized (row→inner→col)';
      renderSingleViz('singleViz', label, loopOrder);
    }
  }

  function renderSingleViz(containerId: string, title: string, mode: string) {
    const container = document.getElementById(containerId);
    if (!container) return;

    container.innerHTML = `
      <h3 style="margin-bottom: 16px; font-size: 1.1rem; color: var(--color-heading);">${title}</h3>
      <div class="viz-container">
        <div style="display: flex; gap: 24px; flex-wrap: wrap;">
          <div class="matrix-wrapper">
            <div class="matrix-label">Matrix A</div>
            <div class="matrix-grid" id="${containerId}-matrixA" style="grid-template-columns: repeat(${currentSize}, 44px);"></div>
          </div>
          <div class="matrix-wrapper">
            <div class="matrix-label">Matrix B</div>
            <div class="matrix-grid" id="${containerId}-matrixB" style="grid-template-columns: repeat(${currentSize}, 44px);"></div>
          </div>
          <div class="matrix-wrapper">
            <div class="matrix-label">Result C</div>
            <div class="matrix-grid" id="${containerId}-matrixC" style="grid-template-columns: repeat(${currentSize}, 44px);"></div>
          </div>
        </div>
      </div>
      <div class="memory-strip-container" style="margin-top: 20px;">
        <div class="memory-label">💾 Matrix B in Memory (Row-Major)</div>
        <div class="memory-strip" id="${containerId}-memoryB"></div>
      </div>
      <div class="access-indicator" id="${containerId}-indicator">
        <span style="font-weight: 600;">Access Pattern:</span>
        <span id="${containerId}-pattern">—</span>
      </div>
      <div class="step-display" id="${containerId}-step">
        <div class="label">Current Access</div>
        <div>Waiting to start...</div>
      </div>
    `;

    // Populate matrices
    const matrixA = document.getElementById(`${containerId}-matrixA`);
    const matrixB = document.getElementById(`${containerId}-matrixB`);
    const matrixC = document.getElementById(`${containerId}-matrixC`);
    const memoryB = document.getElementById(`${containerId}-memoryB`);

    for (let i = 0; i < currentSize; i++) {
      for (let j = 0; j < currentSize; j++) {
        ['A', 'B', 'C'].forEach((mat) => {
          const container = mat === 'A' ? matrixA : mat === 'B' ? matrixB : matrixC;
          const cell = document.createElement('div');
          cell.className = 'matrix-cell';
          cell.textContent = mat === 'C' ? '—' : `${mat}${i}${j}`;
          cell.dataset.row = String(i);
          cell.dataset.col = String(j);
          container?.appendChild(cell);
        });
      }
    }

    // Memory strip
    for (let i = 0; i < currentSize; i++) {
      for (let j = 0; j < currentSize; j++) {
        const memCell = document.createElement('div');
        memCell.className = 'memory-cell';
        memCell.textContent = `B${i}${j}`;
        memCell.dataset.row = String(i);
        memCell.dataset.col = String(j);
        memCell.dataset.index = String(i * currentSize + j);
        if (j === 0 && i > 0) memCell.classList.add('cache-line');
        memoryB?.appendChild(memCell);
      }
    }
  }

  async function runNaiveAnimation(containerId: string) {
    let lastMemIndex = -1;
    const stepEl = document.getElementById(`${containerId}-step`);

    for (let row = 0; row < currentSize; row++) {
      for (let col = 0; col < currentSize; col++) {
        highlightResultCell(containerId, row, col);
        
        for (let inner = 0; inner < currentSize; inner++) {
          if (!animationRunning) return;

          if (stepEl) {
            stepEl.innerHTML = `
              <div class="label">Computing C[${row}][${col}]</div>
              <div>A[${row}][<span class="value">${inner}</span>] × B[<span class="value">${inner}</span>][${col}]</div>
            `;
          }

          clearHighlights(containerId);
          highlightCell(`${containerId}-matrixA`, row, inner, 'current');
          highlightCell(`${containerId}-matrixB`, inner, col, 'current');
          highlightRow(`${containerId}-matrixA`, row);
          highlightCol(`${containerId}-matrixB`, col);

          const memIndex = inner * currentSize + col;
          const memCells = document.querySelectorAll(`#${containerId}-memoryB .memory-cell`);
          memCells.forEach(c => c.classList.remove('accessed', 'strided', 'sequential'));
          if (memCells[memIndex]) memCells[memIndex].classList.add('accessed', 'strided');

          if (lastMemIndex === -1 || Math.abs(memIndex - lastMemIndex) > 1) {
            stats.naiveMisses++;
          }
          lastMemIndex = memIndex;

          const patternEl = document.getElementById(`${containerId}-pattern`);
          if (patternEl) {
            patternEl.innerHTML = `
              <span class="access-arrow good">A: row[${row}] sequential ✓</span>
              <span class="access-arrow bad">B: col[${col}] strided ✗ (jumping by ${currentSize})</span>
            `;
          }

          stats.totalOps++;
          updateProgress();
          await sleep(getDelay());
        }

        markComputed(containerId, row, col);
      }
    }
  }

  async function runOptimizedAnimation(containerId: string) {
    let lastMemIndex = -1;
    const stepEl = document.getElementById(`${containerId}-step`);

    for (let row = 0; row < currentSize; row++) {
      for (let inner = 0; inner < currentSize; inner++) {
        for (let col = 0; col < currentSize; col++) {
          if (!animationRunning) return;

          if (stepEl) {
            stepEl.innerHTML = `
              <div class="label">Updating C[${row}][${col}]</div>
              <div>A[${row}][${inner}] × B[${inner}][<span class="value">${col}</span>]</div>
            `;
          }

          clearHighlights(containerId);
          highlightCell(`${containerId}-matrixA`, row, inner, 'current');
          highlightCell(`${containerId}-matrixB`, inner, col, 'current');
          highlightRow(`${containerId}-matrixA`, row);
          highlightRow(`${containerId}-matrixB`, inner);

          const memIndex = inner * currentSize + col;
          const memCells = document.querySelectorAll(`#${containerId}-memoryB .memory-cell`);
          memCells.forEach(c => c.classList.remove('accessed', 'strided', 'sequential'));
          if (memCells[memIndex]) memCells[memIndex].classList.add('accessed', 'sequential');

          if (col === 0 || lastMemIndex === -1) {
            stats.optMisses++;
          }
          lastMemIndex = memIndex;

          const patternEl = document.getElementById(`${containerId}-pattern`);
          if (patternEl) {
            patternEl.innerHTML = `
              <span class="access-arrow good">A: row[${row}] sequential ✓</span>
              <span class="access-arrow good">B: row[${inner}] sequential ✓</span>
            `;
          }

          stats.totalOps++;
          updateProgress();
          await sleep(getDelay());
        }
      }
    }

    for (let i = 0; i < currentSize; i++) {
      for (let j = 0; j < currentSize; j++) {
        markComputed(containerId, i, j);
      }
    }
  }

  function highlightCell(containerId: string, row: number, col: number, className: string) {
    const cell = document.querySelector(`#${containerId} .matrix-cell[data-row="${row}"][data-col="${col}"]`);
    if (cell) cell.classList.add(className);
  }

  function highlightRow(containerId: string, row: number) {
    document.querySelectorAll(`#${containerId} .matrix-cell[data-row="${row}"]`).forEach(c => {
      if (!c.classList.contains('current')) c.classList.add('row-active');
    });
  }

  function highlightCol(containerId: string, col: number) {
    document.querySelectorAll(`#${containerId} .matrix-cell[data-col="${col}"]`).forEach(c => {
      if (!c.classList.contains('current')) c.classList.add('col-active');
    });
  }

  function highlightResultCell(containerId: string, row: number, col: number) {
    const cell = document.querySelector(`#${containerId}-matrixC .matrix-cell[data-row="${row}"][data-col="${col}"]`) as HTMLElement;
    if (cell) {
      cell.style.background = 'var(--gray-800)';
      cell.style.borderColor = 'var(--gray-600)';
    }
  }

  function markComputed(containerId: string, row: number, col: number) {
    const cell = document.querySelector(`#${containerId}-matrixC .matrix-cell[data-row="${row}"][data-col="${col}"]`) as HTMLElement;
    if (cell) {
      cell.classList.add('computed');
      cell.textContent = '✓';
    }
  }

  function clearHighlights(containerId: string) {
    document.querySelectorAll(`#${containerId} .matrix-cell`).forEach(c => {
      c.classList.remove('current', 'row-active', 'col-active');
    });
  }

  function updateProgress() {
    const total = currentSize * currentSize * currentSize * (loopOrder === 'both' ? 2 : 1);
    progressPercent = (stats.totalOps / total) * 100;
  }

  async function startAnimation() {
    if (animationRunning) return;
    animationRunning = true;
    stepMode = false;
    
    renderVisualization();
    stats = { totalOps: 0, naiveMisses: 0, optMisses: 0 };
    progressPercent = 0;

    if (loopOrder === 'naive' || loopOrder === 'both') {
      await runNaiveAnimation(loopOrder === 'both' ? 'naiveViz' : 'singleViz');
    }

    if (loopOrder === 'optimized' || loopOrder === 'both') {
      if (loopOrder === 'both') {
        renderSingleViz('optimizedViz', 'Optimized (row→inner→col)', 'optimized');
      }
      await runOptimizedAnimation(loopOrder === 'both' ? 'optimizedViz' : 'singleViz');
    }

    animationRunning = false;
  }

  function stepAnimation() {
    if (stepResolve) {
      stepResolve();
      stepResolve = null;
    } else if (!animationRunning) {
      stepMode = true;
      startAnimation();
    }
  }

  function reset() {
    animationRunning = false;
    stepMode = false;
    stepResolve = null;
    progressPercent = 0;
    stats = { totalOps: 0, naiveMisses: 0, optMisses: 0 };
    renderVisualization();
  }

  $: speedupRatio = stats.naiveMisses > 0 && stats.optMisses > 0 
    ? (stats.naiveMisses / stats.optMisses).toFixed(1) 
    : '-';

  onMount(() => {
    renderVisualization();
  });
</script>

<div class="page matmul-page">
  <header class="page-header">
    <h1 class="page-title">Matrix Multiplication Visualizer</h1>
    <p class="page-desc">Understanding why loop order matters for cache performance</p>
  </header>

  <!-- Controls -->
  <div class="controls-panel">
    <div class="controls-grid">
      <div class="control-group">
        <label>📐 Matrix Size</label>
        <select bind:value={currentSize} on:change={reset} class="glass-input">
          <option value={3}>3 × 3</option>
          <option value={4}>4 × 4</option>
          <option value={5}>5 × 5</option>
        </select>
      </div>
      <div class="control-group">
        <label>⚡ Animation Speed</label>
        <input type="range" bind:value={animationSpeed} min="1" max="10" />
        <div class="speed-display">{speedLabels[animationSpeed - 1]} ({animationSpeed}x)</div>
      </div>
      <div class="control-group">
        <label>🔄 Loop Order</label>
        <select bind:value={loopOrder} on:change={reset} class="glass-input">
          <option value="naive">Naive: row → col → inner (Slow)</option>
          <option value="optimized">Optimized: row → inner → col (Fast)</option>
          <option value="both">Compare Both Side-by-Side</option>
        </select>
      </div>
      <div class="btn-group">
        <button class="btn-primary" on:click={startAnimation} disabled={animationRunning}>
          ▶ Start
        </button>
        <button class="btn-secondary" on:click={stepAnimation}>
          ⏭ Step
        </button>
        <button class="btn-danger" on:click={reset}>
          ↺ Reset
        </button>
      </div>
    </div>
    <div class="progress-bar">
      <div class="progress-fill" style="width: {progressPercent}%"></div>
    </div>
  </div>

  <!-- Loop Comparison -->
  <section class="section">
    <div class="section-header">
      <h2 class="section-title">🔀 Loop Order Comparison</h2>
    </div>
    
    <div class="loop-comparison">
      <div class="loop-panel slow">
        <div class="loop-header">
          <span class="loop-title">❌ Naive Order</span>
          <span class="badge badge-slow">Cache Unfriendly</span>
        </div>
        <div class="code-block">
<span class="keyword">for</span> (<span class="variable">row</span> = 0..N)
  <span class="keyword">for</span> (<span class="variable">col</span> = 0..N)
    <span class="keyword">for</span> (<span class="variable">inner</span> = 0..N)
      C[row][col] += A[row][<span class="variable">inner</span>] × B[<span class="variable">inner</span>][col]
                     <span style="color: var(--color-success)">↑ sequential</span>    <span style="color: var(--color-danger)">↑ STRIDED!</span>
        </div>
      </div>

      <div class="loop-panel fast">
        <div class="loop-header">
          <span class="loop-title">✅ Optimized Order</span>
          <span class="badge badge-fast">Cache Friendly</span>
        </div>
        <div class="code-block">
<span class="keyword">for</span> (<span class="variable">row</span> = 0..N)
  <span class="keyword">for</span> (<span class="variable">inner</span> = 0..N)
    <span class="keyword">for</span> (<span class="variable">col</span> = 0..N)
      C[row][<span class="variable">col</span>] += A[row][inner] × B[inner][<span class="variable">col</span>]
                     <span style="color: var(--color-success)">↑ sequential</span>    <span style="color: var(--color-success)">↑ sequential!</span>
        </div>
      </div>
    </div>

    <div class="cache-demo">
      <h4>💡 The Key Insight</h4>
      <p>
        In <strong>row-major layout</strong>, elements in the same row are stored next to each other.
        The CPU loads entire <strong>cache lines</strong> (64 bytes ≈ 16 floats).
        Sequential access is essentially free after the first load!
      </p>
    </div>
  </section>

  <!-- Visualization -->
  <section class="section">
    <div class="section-header">
      <h2 class="section-title">🎬 Live Memory Access</h2>
    </div>

    <div bind:this={visualizationArea} class="visualization-area"></div>

    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-value">{stats.totalOps}</div>
        <div class="stat-label">Total Accesses</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{stats.naiveMisses}</div>
        <div class="stat-label">Cache Misses (Naive)</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{stats.optMisses}</div>
        <div class="stat-label">Cache Misses (Optimized)</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{speedupRatio}</div>
        <div class="stat-label">Estimated Speedup</div>
      </div>
    </div>

    <div class="legend">
      <div class="legend-item">
        <div class="legend-box current"></div>
        <span>Currently Accessing</span>
      </div>
      <div class="legend-item">
        <div class="legend-box row-active"></div>
        <span>Active Row</span>
      </div>
      <div class="legend-item">
        <div class="legend-box col-active"></div>
        <span>Active Column</span>
      </div>
      <div class="legend-item">
        <div class="legend-box sequential"></div>
        <span>Sequential Access ✓</span>
      </div>
      <div class="legend-item">
        <div class="legend-box strided"></div>
        <span>Strided Access ✗</span>
      </div>
    </div>
  </section>

  <!-- Why It Matters -->
  <section class="section">
    <div class="section-header">
      <h2 class="section-title">🚀 Why This Matters</h2>
    </div>

    <div class="explanation-grid">
      <div class="explanation-card">
        <h4>📦 Cache Lines</h4>
        <p>
          CPUs load entire cache lines (64 bytes) at once. When you access <code>array[0]</code>,
          you get <code>array[1..15]</code> for free!
        </p>
      </div>
      <div class="explanation-card">
        <h4>🎯 Spatial Locality</h4>
        <p>
          Sequential access exploits this perfectly. Strided access wastes the loaded cache lines,
          causing repeated memory fetches.
        </p>
      </div>
      <div class="explanation-card">
        <h4>⏱️ Real Performance</h4>
        <p>
          For 1024×1024 matrices, loop reordering gives <strong>2-10× speedup</strong>.
          With SIMD and threading: <strong>100-1000× faster</strong>!
        </p>
      </div>
      <div class="explanation-card">
        <h4>🔧 Implementation</h4>
        <p>
          The optimized version accesses both A and B sequentially in the inner loop,
          maximizing cache hits and memory bandwidth utilization.
        </p>
      </div>
    </div>
  </section>
</div>

<style>
  :global(.main-content:has(.matmul-page)) {
    max-width: 100% !important;
    padding-left: 1rem;
    padding-right: 1rem;
  }

  .matmul-page {
    width: 100%;
  }

  .controls-panel {
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .controls-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1.25rem;
    align-items: end;
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .control-group label {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-heading);
  }

  .speed-display {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    text-align: center;
  }

  input[type="range"] {
    width: 100%;
    height: 8px;
    border-radius: 4px;
    background: var(--gray-800);
    -webkit-appearance: none;
    cursor: pointer;
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 20px;
    height: 20px;
    background: var(--white);
    border-radius: 50%;
    cursor: pointer;
  }

  .btn-group {
    display: flex;
    gap: 0.625rem;
    flex-wrap: wrap;
  }

  button {
    padding: 0.75rem 1.25rem;
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    background: var(--glass-bg);
    color: var(--color-heading);
  }

  button:hover:not(:disabled) {
    background: var(--glass-bg-hover);
    border-color: var(--glass-border-hover);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--white);
    color: var(--black);
    border-color: var(--white);
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--gray-200);
  }

  .btn-secondary {
    /* inherits default button styles */
  }

  .btn-danger {
    background: var(--gray-800);
    border-color: var(--gray-700);
  }

  .progress-bar {
    height: 4px;
    background: var(--gray-800);
    border-radius: 2px;
    margin-top: 1rem;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--white);
    border-radius: 2px;
    transition: width 0.3s ease-out;
  }

  .section {
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    padding: 1.75rem;
    margin-bottom: 1.5rem;
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--glass-border);
  }

  .section-title {
    font-size: 1.4rem;
    font-weight: 700;
    color: var(--color-heading);
  }

  .loop-comparison {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .loop-panel {
    background: rgba(255, 255, 255, 0.01);
    border-radius: var(--radius-md);
    padding: 1.25rem;
    border: 1px solid var(--glass-border);
  }

  .loop-panel.slow {
    border-color: rgba(239, 68, 68, 0.3);
  }

  .loop-panel.fast {
    border-color: rgba(34, 197, 94, 0.3);
  }

  .loop-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .loop-title {
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--color-heading);
  }

  .badge {
    padding: 0.25rem 0.625rem;
    border-radius: 20px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .badge-slow {
    background: rgba(239, 68, 68, 0.2);
    color: #fca5a5;
  }

  .badge-fast {
    background: rgba(34, 197, 94, 0.2);
    color: #86efac;
  }

  .code-block {
    background: var(--gray-950);
    color: var(--gray-300);
    padding: 1rem;
    border-radius: var(--radius-md);
    font-family: var(--font-mono);
    font-size: 0.85rem;
    line-height: 1.6;
    margin-bottom: 1rem;
    overflow-x: auto;
  }

  .code-block .keyword { color: #c084fc; }
  .code-block .variable { color: #38bdf8; }

  .cache-demo {
    background: rgba(251, 191, 36, 0.05);
    border: 1px solid rgba(251, 191, 36, 0.2);
    border-radius: var(--radius-md);
    padding: 1.25rem;
    margin-top: 1.25rem;
  }

  .cache-demo h4 {
    color: var(--color-heading);
    margin-bottom: 0.75rem;
  }

  .cache-demo p {
    color: var(--color-text);
    line-height: 1.6;
    font-size: 0.9rem;
  }

  .visualization-area {
    margin: 1.5rem 0;
  }

  :global(.viz-container) {
    margin: 1rem 0;
  }

  :global(.matrix-wrapper) {
    text-align: center;
  }

  :global(.matrix-label) {
    font-weight: 600;
    color: var(--color-heading);
    margin-bottom: 0.75rem;
    font-size: 0.95rem;
  }

  :global(.matrix-grid) {
    display: inline-grid;
    gap: 3px;
    padding: 8px;
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-md);
  }

  :global(.matrix-cell) {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--gray-900);
    border: 2px solid var(--gray-800);
    border-radius: 6px;
    font-weight: 600;
    font-size: 0.85rem;
    color: var(--color-text);
    transition: all 0.15s ease-out;
  }

  :global(.matrix-cell.current) {
    background: var(--white);
    border-color: var(--gray-300);
    color: var(--black);
    transform: scale(1.15);
    z-index: 10;
  }

  :global(.matrix-cell.row-active) {
    background: var(--gray-800);
    border-color: var(--gray-700);
  }

  :global(.matrix-cell.col-active) {
    background: var(--gray-800);
    border-color: var(--gray-700);
  }

  :global(.matrix-cell.computed) {
    background: var(--white);
    color: var(--black);
    border-color: var(--gray-400);
  }

  :global(.memory-strip-container) {
    margin-top: 1.25rem;
  }

  :global(.memory-label) {
    font-weight: 600;
    color: var(--color-heading);
    margin-bottom: 0.75rem;
    font-size: 0.95rem;
  }

  :global(.memory-strip) {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.01);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    min-height: 60px;
  }

  :global(.memory-cell) {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--gray-900);
    border: 2px solid var(--gray-800);
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 600;
    transition: all 0.15s ease-out;
  }

  :global(.memory-cell.accessed) {
    background: var(--white);
    color: var(--black);
    transform: scale(1.1);
  }

  :global(.memory-cell.sequential) {
    border-color: #22c55e;
  }

  :global(.memory-cell.strided) {
    border-color: #ef4444;
  }

  :global(.memory-cell.cache-line) {
    border-left: 3px solid var(--white);
  }

  :global(.access-indicator) {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.01);
    border-radius: var(--radius-md);
    margin-top: 1rem;
    font-size: 0.85rem;
  }

  :global(.access-arrow) {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-weight: 600;
  }

  :global(.access-arrow.good) {
    color: #22c55e;
  }

  :global(.access-arrow.bad) {
    color: #ef4444;
  }

  :global(.step-display) {
    background: var(--gray-950);
    color: var(--gray-300);
    padding: 1rem 1.25rem;
    border-radius: var(--radius-md);
    margin-top: 1rem;
    font-family: var(--font-mono);
    font-size: 0.9rem;
  }

  :global(.step-display .label) {
    color: var(--gray-500);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: 0.375rem;
  }

  :global(.step-display .value) {
    color: #fbbf24;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-top: 1.5rem;
  }

  .stat-card {
    padding: 1.25rem;
    border-radius: var(--radius-md);
    text-align: center;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--glass-border);
  }

  .stat-value {
    font-size: 2.2rem;
    font-weight: 700;
    color: var(--color-heading);
  }

  .stat-label {
    font-size: 0.85rem;
    color: var(--color-text-muted);
    margin-top: 0.25rem;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.01);
    border-radius: var(--radius-md);
    margin-top: 1.25rem;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .legend-box {
    width: 24px;
    height: 24px;
    border-radius: 4px;
    border: 2px solid;
  }

  .legend-box.current {
    background: var(--white);
    border-color: var(--gray-300);
  }

  .legend-box.row-active {
    background: var(--gray-800);
    border-color: var(--gray-700);
  }

  .legend-box.col-active {
    background: var(--gray-800);
    border-color: var(--gray-700);
  }

  .legend-box.sequential {
    background: var(--gray-900);
    border-color: #22c55e;
  }

  .legend-box.strided {
    background: var(--gray-900);
    border-color: #ef4444;
  }

  .explanation-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
    margin-top: 1.5rem;
  }

  .explanation-card {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-md);
    padding: 1.25rem;
    border: 1px solid var(--glass-border);
  }

  .explanation-card h4 {
    font-size: 1rem;
    font-weight: 700;
    color: var(--color-heading);
    margin-bottom: 0.75rem;
  }

  .explanation-card p {
    font-size: 0.9rem;
    color: var(--color-text);
    line-height: 1.6;
  }

  .explanation-card code {
    background: var(--gray-900);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 0.85em;
    color: var(--gray-300);
  }

  @media (max-width: 900px) {
    .loop-comparison {
      grid-template-columns: 1fr;
    }
    
    .controls-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
