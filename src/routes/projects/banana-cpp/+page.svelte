<script lang="ts">
  import { reveal, splitReveal, countUp } from '$lib/motion';
</script>

<svelte:head>
  <title>banana.cpp — LLM Inference Engine</title>
  <meta name="description" content="A modular, pure C++ implementation of a small language model inference engine." />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/projects">Projects</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">banana.cpp</span>
    </nav>
    <h1 class="page-title" use:splitReveal>banana.cpp</h1>
    <p class="lead">A modular, pure C++ implementation of a small language model inference engine supporting multiple model architectures.</p>
    <div class="btn-row">
      <a href="https://github.com/kashifulhaque/banana.cpp" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Code <span class="ext" aria-hidden="true">↗</span>
      </a>
    </div>
  </header>

  <div class="detail-body">
    <aside class="facts" aria-label="Project facts">
      <dl use:reveal={{ children: ".fact", gap: 70 }}>
        <div class="fact">
          <dt>Stack</dt>
          <dd><span>C++</span><span>CMake</span></dd>
        </div>
        <div class="fact">
          <dt>Models</dt>
          <dd><span>SmolLM2</span><span>Llama 3.2</span><span>Qwen</span></dd>
        </div>
        <div class="fact">
          <dt>Precision</dt>
          <dd>FP16 · BF16</dd>
        </div>
        <div class="fact">
          <dt>Links</dt>
          <dd>
            <a href="https://github.com/kashifulhaque/banana.cpp" target="_blank" rel="noopener noreferrer">GitHub <span aria-hidden="true">↗</span></a>
          </dd>
        </div>
      </dl>
    </aside>

    <div class="copy">
      <section class="section" use:reveal>
        <h2><span class="num">01</span>Overview</h2>
        <p>
          banana.cpp is a pure C++ LLM inference engine designed for modularity and extensibility.
          It supports multiple small language model architectures including SmolLM2, Llama 3.2, and Qwen variants,
          with built-in support for FP16/BF16 precision and modern architectural features like GQA, RoPE, and SwiGLU.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">02</span>Supported Models</h2>
        <ul class="defs">
          <li><span class="term">SmolLM2</span><span class="desc">135M, 360M, 1.7B parameter variants</span></li>
          <li><span class="term">Llama 3.2</span><span class="desc">1B, 3B parameter variants</span></li>
          <li><span class="term">Qwen</span><span class="desc">2.5-0.5B, 3-0.6B parameter variants</span></li>
        </ul>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">03</span>Key Features</h2>
        <ul class="defs">
          <li><span class="term">Modular Architecture</span><span class="desc">Layer-based design makes it easy to add new architectures</span></li>
          <li><span class="term">Mixed Precision</span><span class="desc">Native FP16 and BF16 support for efficient inference</span></li>
          <li><span class="term">Modern Attention</span><span class="desc">GQA, MHA, and MQA attention mechanisms</span></li>
          <li><span class="term">Position Encodings</span><span class="desc">RoPE (Rotary Position Embedding) implementation</span></li>
          <li><span class="term">Activation Functions</span><span class="desc">SwiGLU and standard MLP layers</span></li>
          <li><span class="term">Normalization</span><span class="desc">RMSNorm and LayerNorm implementations</span></li>
          <li><span class="term">Auto-detection</span><span class="desc">Model registry automatically detects model type from config</span></li>
          <li><span class="term">HuggingFace Integration</span><span class="desc">Built-in model downloader for HuggingFace models</span></li>
        </ul>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">04</span>Architecture</h2>
        <p>The engine uses a clean, modular layer-based architecture:</p>
        <ul class="defs">
          <li><span class="term">Layers</span><span class="desc">Reusable components (Attention, MLP, Normalization, RoPE)</span></li>
          <li><span class="term">Models</span><span class="desc">Compose layers based on configuration</span></li>
          <li><span class="term">Tokenizers</span><span class="desc">Separate BPE logic from chat template handling</span></li>
          <li><span class="term">Registry</span><span class="desc">Auto-detect model type from config.json</span></li>
        </ul>
        <p class="note">
          This design philosophy makes it straightforward to add new model architectures by composing existing layers,
          introduce new layer types without modifying models, and support multiple chat templates and tokenizer formats.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">05</span>Usage</h2>
        <pre><code># List supported models
./banana-cpp --list-models

# Download and run a model
./banana-cpp --model smollm2-360m --download
./banana-cpp --model smollm2-360m --prompt "What is the capital of France?"

# Interactive mode
./banana-cpp --model smollm2-360m --interactive

# Custom settings
./banana-cpp --model llama-3.2-1b \
    --temperature 0.8 \
    --top-k 40 \
    --top-p 0.95 \
    --max-tokens 512</code></pre>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">06</span>Building</h2>
        <pre><code>mkdir build && cd build
cmake ..
make -j$(nproc)</code></pre>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">07</span>Project Structure</h2>
        <ul class="defs">
          <li><span class="term"><code>include/config/</code></span><span class="desc">Model and tokenizer configuration structures</span></li>
          <li><span class="term"><code>include/core/</code></span><span class="desc">Core tensor data structures and operations</span></li>
          <li><span class="term"><code>include/layers/</code></span><span class="desc">Neural network layer implementations</span></li>
          <li><span class="term"><code>include/models/</code></span><span class="desc">Model architecture definitions</span></li>
          <li><span class="term"><code>include/tokenizers/</code></span><span class="desc">BPE tokenization and chat templates</span></li>
          <li><span class="term"><code>include/registry/</code></span><span class="desc">Model auto-detection system</span></li>
          <li><span class="term"><code>include/utils/</code></span><span class="desc">Model loading and HuggingFace downloading</span></li>
        </ul>
      </section>
    </div>
  </div>
</article>

<style>
  /* ── Detail page (shared across project write-ups) ───────────── */

  .page-title {
    max-width: 18ch;
    overflow-wrap: break-word;
  }

  /* Sections carry their own top hairline, so the head drops its rule */
  .page-head {
    border-bottom: 0;
  }

  .page-head .lead {
    margin: 0;
  }

  .page-head .btn-row {
    margin-top: var(--s-2);
  }

  .ext {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .btn:hover .ext {
    color: inherit;
    transform: translate(2px, -2px);
  }

  .detail-body {
    display: grid;
    gap: var(--s-7);
  }

  /* Facts: mono summary column */
  .facts dl {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--s-5);
    margin: 0;
    border-top: 1px solid var(--line);
  }

  .fact {
    min-width: 0;
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .fact dt {
    margin-bottom: var(--s-1);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .fact dd {
    display: grid;
    gap: 2px;
    margin: 0;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.5;
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
  }

  .fact a {
    display: inline-flex;
    align-items: baseline;
    gap: var(--s-2);
    color: var(--ink);
    text-decoration: none;
  }

  .fact a:hover {
    color: var(--fjord);
  }

  .fact a span {
    color: var(--ink-3);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .fact a:hover span {
    color: var(--fjord);
    transform: translate(2px, -2px);
  }

  /* Reading column (not .prose: that name belongs to Tailwind Typography) */
  .copy {
    min-width: 0;
  }

  .section {
    padding-top: var(--s-4);
    border-top: 1px solid var(--line);
  }

  .section + .section {
    margin-top: var(--s-8);
  }

  .section > * {
    margin: 0;
  }

  .section > * + * {
    margin-top: var(--s-4);
  }

  .section h2 {
    display: flex;
    align-items: baseline;
    gap: var(--s-4);
    font-size: var(--text-xl);
    letter-spacing: -0.022em;
  }

  .num {
    flex: none;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    letter-spacing: 0.02em;
    font-variant-numeric: tabular-nums;
  }

  .section > h2 + * {
    margin-top: var(--s-5);
  }

  .section p {
    line-height: 1.65;
  }

  @media (min-width: 960px) {
    .detail-body {
      grid-template-columns: 240px minmax(0, var(--measure));
      column-gap: var(--s-8);
      align-items: start;
      padding-top: var(--s-4);
    }

    .facts {
      position: sticky;
      top: calc(64px + var(--s-6));
    }

    .facts dl {
      grid-template-columns: minmax(0, 1fr);
    }

    .section + .section {
      margin-top: var(--s-9);
    }
  }

  /* Term / description rows */
  .defs {
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  .defs > li {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-1) var(--s-5);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .term {
    min-width: 0;
    color: var(--ink);
    font-weight: 500;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }


  .desc {
    color: var(--ink-2);
    line-height: 1.55;
  }

  @media (min-width: 600px) {
    .defs > li {
      grid-template-columns: minmax(0, 12rem) minmax(0, 1fr);
    }
  }

  .term code {
    padding: 0;
    background: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
  }

  /* Aside paragraph set off by a hairline */
  .note {
    padding-left: var(--s-4);
    border-left: 1px solid var(--line-strong);
    color: var(--ink-2);
  }
</style>
