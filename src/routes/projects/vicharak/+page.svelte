<script lang="ts">
  import { reveal, splitReveal, countUp } from '$lib/motion';
</script>

<svelte:head>
  <title>Micro-Llama on Vicharak Shrike-Lite — Bare-Metal Edge Inference</title>
  <meta name="description" content="Train a tiny Llama on TinyStories, then run bare-metal inference on the Vicharak Shrike-Lite (RP2040 + FPGA, 264KB SRAM). 213K params, C firmware." />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/projects">Projects</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">vicharak</span>
    </nav>
    <h1 class="page-title" use:splitReveal>Micro-Llama on Shrike-Lite <span class="mark">🔌</span></h1>
    <p class="lead">A tiny Llama-style language model trained from scratch on TinyStories, then deployed for bare-metal inference on the Vicharak Shrike-Lite — an RP2040 + Renesas FPGA board with only 264KB of SRAM.</p>
    <div class="btn-row">
      <a href="https://github.com/weights-and-wires/vicharak-llm" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Code <span class="ext" aria-hidden="true">↗</span>
      </a>
      <a href="https://huggingface.co/weights-and-wires/vicharak-micro-llama" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Model <span class="ext" aria-hidden="true">↗</span>
      </a>
      <a href="https://api.wandb.ai/links/dotslasha/9zs6fx7t" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        W&amp;B Report <span class="ext" aria-hidden="true">↗</span>
      </a>
    </div>
  </header>

  <div class="detail-body">
    <aside class="facts" aria-label="Project facts">
      <dl use:reveal={{ children: ".fact", gap: 70 }}>
        <div class="fact">
          <dt>Board</dt>
          <dd><span>Shrike-Lite</span><span>RP2040 + FPGA</span></dd>
        </div>
        <div class="fact">
          <dt>SRAM</dt>
          <dd>264 KB</dd>
        </div>
        <div class="fact">
          <dt>Parameters</dt>
          <dd>213,312</dd>
        </div>
        <div class="fact">
          <dt>Stack</dt>
          <dd><span>JAX / Flax</span><span>C</span></dd>
        </div>
        <div class="fact">
          <dt>Eval loss</dt>
          <dd>~3.27</dd>
        </div>
        <div class="fact">
          <dt>Perplexity</dt>
          <dd>~26.4</dd>
        </div>
        <div class="fact">
          <dt>Links</dt>
          <dd>
            <a href="https://github.com/weights-and-wires/vicharak-llm" target="_blank" rel="noopener noreferrer">GitHub <span aria-hidden="true">↗</span></a>
            <a href="https://huggingface.co/weights-and-wires/vicharak-micro-llama" target="_blank" rel="noopener noreferrer">Model <span aria-hidden="true">↗</span></a>
            <a href="https://api.wandb.ai/links/dotslasha/9zs6fx7t" target="_blank" rel="noopener noreferrer">W&amp;B report <span aria-hidden="true">↗</span></a>
          </dd>
        </div>
      </dl>
    </aside>

    <div class="copy">
      <section class="section" use:reveal>
        <h2><span class="num">01</span>Overview</h2>
        <p>
          The goal was to push a transformer language model to the absolute edge: train a small Llama from scratch, then run
          it <em>entirely bare-metal</em> on a microcontroller with no operating system, no allocator, and no floating-point
          hardware. The board is the
          <a href="https://github.com/vicharak-in/shrike" target="_blank" rel="noopener noreferrer">Vicharak Shrike-Lite</a>
          — a dual-core RP2040 paired with a Renesas SLG47910 FPGA, sharing just 264KB of SRAM.
        </p>
        <p>
          The full pipeline lives in one repo: JAX/Flax training on a GPU, export to a flat binary, then a hand-written C
          inference core that reads weights from QSPI flash via XIP and streams generated text over USB CDC.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">02</span>Pipeline</h2>
        <pre><code>TinyStories → JAX train (RunPod) → micro_llama.bin → C firmware → UF2 on Shrike-Lite</code></pre>
        <p class="note">
          The model is trained in JAX/Flax, exported to a single <code>micro_llama.bin</code> checkpoint, embedded into the
          firmware, and flashed to the board as a UF2 — replacing MicroPython entirely.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">03</span>Model Architecture</h2>
        <div class="stats" use:reveal={{ children: ".stat", gap: 80 }}>
          <div class="stat">
            <span class="stat-label">Parameters</span>
            <span class="stat-value">213,312</span>
          </div>
          <div class="stat">
            <span class="stat-label">dim / hidden_dim</span>
            <span class="stat-value">64 / 128</span>
          </div>
          <div class="stat">
            <span class="stat-label">Layers</span>
            <span class="stat-value" use:countUp>2</span>
          </div>
          <div class="stat">
            <span class="stat-label">Attention Heads</span>
            <span class="stat-value">2Q / 2KV</span>
          </div>
          <div class="stat">
            <span class="stat-label">Context Length</span>
            <span class="stat-value" use:countUp>128</span>
          </div>
          <div class="stat">
            <span class="stat-label">Vocab Size</span>
            <span class="stat-value">1,024 BPE</span>
          </div>
        </div>
        <p class="note">
          Standard Llama building blocks: GQA (2 query heads, 2 key-value heads, head_dim 32), RoPE, RMSNorm, and a SwiGLU
          feed-forward network with an untied output head. Special tokens: pad=0, unk=1, bos=2, eos=3.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">04</span>The 264KB SRAM Constraint</h2>
        <p>
          Everything has to fit on a chip with 264KB of total SRAM and no FPU. The architecture is locked by that budget:
          weights live in QSPI flash via <strong>XIP</strong> (execute-in-place), while activations and the KV cache must
          fit inside SRAM at runtime. The fp32 KV cache alone occupies ~131KB — which is why the shape above is as small as
          it is. FPGA INT8 MAC acceleration on the SLG47910 (~1120 LUTs) is mapped out but not yet started.
        </p>
        <dl class="kv">
          <div><dt>Board</dt><dd>Vicharak Shrike-Lite (RP2040 + Renesas FPGA)</dd></div>
          <div><dt>Total SRAM</dt><dd>264 KB</dd></div>
          <div><dt>Weight storage</dt><dd>QSPI Flash via XIP</dd></div>
          <div><dt>KV cache (fp32)</dt><dd>~131 KB</dd></div>
          <div><dt>Output</dt><dd>USB CDC serial text</dd></div>
        </dl>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">05</span>Training Details</h2>
        <dl class="kv">
          <div><dt>GPU</dt><dd>1× NVIDIA RTX 3090 (RunPod)</dd></div>
          <div><dt>Framework</dt><dd>JAX / Flax</dd></div>
          <div><dt>Dataset</dt><dd>TinyStories</dd></div>
          <div><dt>Training Steps</dt><dd>5,000</dd></div>
          <div><dt>Eval Loss</dt><dd>~3.27</dd></div>
          <div><dt>Perplexity</dt><dd>~26.4</dd></div>
        </dl>
        <p class="note">
          The training runtime auto-selects CUDA → Metal → CPU <em>before</em> importing JAX, so the same code trains on an
          NVIDIA GPU, an Apple Silicon Mac, or plain CPU.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">06</span>Firmware Targets</h2>
        <p>The C inference core builds to two targets from the same source (<code>-DML_TARGET=host|pico</code>):</p>
        <ul class="defs">
          <li><span class="term">host</span><span class="desc">native greedy decoder for parity testing against the JAX reference (<code>verify_greedy.py</code>)</span></li>
          <li><span class="term">pico</span><span class="desc">Shrike-Lite UF2 with weights + tokenizer baked into flash, streaming text over USB CDC</span></li>
          <li><span class="term">XIP weights</span><span class="desc">model weights and the BPE tokenizer are embedded in QSPI flash and read in place, never copied to SRAM</span></li>
          <li><span class="term">Soft-float RP2040</span><span class="desc">generation runs on the Cortex-M0+ without hardware floating point</span></li>
          <li><span class="term">JAX ↔ C parity</span><span class="desc">the host decoder is verified to match greedy generation bit-for-bit</span></li>
        </ul>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">07</span>Quick Start</h2>
        <h3>Train (JAX / Flax)</h3>
        <pre><code>uv sync
uv pip install -U "jax[cuda12]"   # NVIDIA; Mac: jax-metal or CPU
cp .env.example .env              # WANDB_API_KEY, HF_TOKEN, HF_REPO_ID

uv run python train/train.py
# dry run:
uv run python train/train.py --no-wandb --no-hub --max-steps 100</code></pre>
        <h3>Host inference + parity check</h3>
        <pre><code>bash firmware/scripts/download_weights.sh
uv run python firmware/scripts/embed_tokenizer.py
cmake -S firmware -B firmware/build-host -DML_TARGET=host && cmake --build firmware/build-host -j
./firmware/build-host/micro_llama_host \
  --checkpoint firmware/weights/micro_llama.bin --bos --max-new-tokens 32 --text
uv run python firmware/scripts/verify_greedy.py</code></pre>
        <p class="note">
          For the Pico target, build with <code>-DML_TARGET=pico</code>, then hold BOOTSEL and run
          <code>flash_uf2.sh</code>. Open the serial console before resetting so the CDC wait captures the full session.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">08</span>Project Structure</h2>
        <ul class="defs">
          <li><span class="term"><code>train/train.py</code></span><span class="desc">JAX/Flax pretraining loop with W&amp;B logging</span></li>
          <li><span class="term"><code>train/runtime.py</code></span><span class="desc">device selection (CUDA → Metal → CPU) before importing JAX</span></li>
          <li><span class="term"><code>train/export.py</code></span><span class="desc">Flax checkpoint → flat <code>micro_llama.bin</code></span></li>
          <li><span class="term"><code>train/generate.py</code></span><span class="desc">greedy generation in Python</span></li>
          <li><span class="term"><code>train/hf_upload.py</code></span><span class="desc">publish weights + tokenizer to the HuggingFace Hub</span></li>
          <li><span class="term"><code>firmware/</code></span><span class="desc">C inference core (host CLI + RP2040 UF2)</span></li>
          <li><span class="term"><code>firmware/scripts/</code></span><span class="desc">weight download, tokenizer embed, UF2 flash helpers</span></li>
        </ul>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">09</span>Resources</h2>
        <ul class="linklist">
          <li>
            <a href="https://github.com/weights-and-wires/vicharak-llm" target="_blank" rel="noopener noreferrer">
              Source Code on GitHub
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a href="https://huggingface.co/weights-and-wires/vicharak-micro-llama" target="_blank" rel="noopener noreferrer">
              Model Weights on HuggingFace
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a href="https://api.wandb.ai/links/dotslasha/9zs6fx7t" target="_blank" rel="noopener noreferrer">
              Weights &amp; Biases Training Report
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a href="https://github.com/vicharak-in/shrike" target="_blank" rel="noopener noreferrer">
              Vicharak Shrike-Lite Board
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
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

  /* Sub-headings inside a section */
  .section > * + h3 {
    margin-top: var(--s-7);
  }

  .section > h3 + * {
    margin-top: var(--s-3);
  }

  .section h3 {
    font-size: var(--text-md);
    letter-spacing: -0.01em;
  }

  .page-title .mark {
    font-size: 0.6em;
    letter-spacing: 0;
    vertical-align: 0.12em;
  }

  /* Aside paragraph set off by a hairline */
  .note {
    padding-left: var(--s-4);
    border-left: 1px solid var(--line-strong);
    color: var(--ink-2);
  }

  /* Key numbers: rows on mobile, a spec-sheet grid from 600px */
  .stats {
    display: grid;
    border-top: 1px solid var(--line);
  }

  .stat {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-4);
    min-width: 0;
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .stat-label {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .stat-value {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-lg);
    line-height: 1.1;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  @media (min-width: 600px) {
    .stats {
      grid-template-columns: repeat(3, minmax(0, 1fr));
      border-left: 1px solid var(--line);
    }

    .stat {
      flex-direction: column;
      justify-content: flex-start;
      gap: var(--s-3);
      padding: var(--s-4);
      border-right: 1px solid var(--line);
    }

    .stat-value {
      font-size: var(--text-xl);
      text-align: left;
    }
  }

  /* Key / value rows with mono values */
  .kv {
    border-top: 1px solid var(--line);
  }

  .kv > div {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(0, 3fr);
    gap: var(--s-5);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
  }

  .kv dt {
    color: var(--ink-2);
  }

  .kv dd {
    margin: 0;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.7;
    font-variant-numeric: tabular-nums;
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

  /* Outbound resource rows */
  .linklist {
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  .linklist a {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
    color: var(--ink);
    text-decoration: none;
  }

  .linklist a:hover {
    color: var(--fjord);
  }

  .arrow {
    flex: none;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .linklist a:hover .arrow {
    color: var(--fjord);
    transform: translate(2px, -2px);
  }
</style>
