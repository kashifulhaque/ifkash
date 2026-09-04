<svelte:head>
  <title>Micro-Llama on Vicharak Shrike-Lite — Bare-Metal Edge Inference</title>
  <meta name="description" content="Train a tiny Llama on TinyStories, then run bare-metal inference on the Vicharak Shrike-Lite (RP2040 + FPGA, 264KB SRAM). 213K params, C firmware." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/projects">Projects</a>
      <span class="separator">/</span>
      <span>vicharak</span>
    </div>
    <h1 class="page-title">Micro-Llama on Shrike-Lite 🔌</h1>
    <p class="page-desc">A tiny Llama-style language model trained from scratch on TinyStories, then deployed for bare-metal inference on the Vicharak Shrike-Lite — an RP2040 + Renesas FPGA board with only 264KB of SRAM.</p>
    <div class="project-links">
      <a href="https://github.com/weights-and-wires/vicharak-llm" target="_blank" rel="noopener noreferrer" class="project-link">
        Code
      </a>
      <a href="https://huggingface.co/weights-and-wires/vicharak-micro-llama" target="_blank" rel="noopener noreferrer" class="project-link">
        Model
      </a>
      <a href="https://api.wandb.ai/links/dotslasha/9zs6fx7t" target="_blank" rel="noopener noreferrer" class="project-link">
        W&amp;B Report
      </a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Overview</h2>
      <p>
        The goal was to push a transformer language model to the absolute edge: train a small Llama from scratch, then run
        it <em>entirely bare-metal</em> on a microcontroller with no operating system, no allocator, and no floating-point
        hardware. The board is the
        <a href="https://github.com/vicharak-in/shrike" target="_blank" rel="noopener noreferrer" class="inline-link">Vicharak Shrike-Lite</a>
        — a dual-core RP2040 paired with a Renesas SLG47910 FPGA, sharing just 264KB of SRAM.
      </p>
      <p>
        The full pipeline lives in one repo: JAX/Flax training on a GPU, export to a flat binary, then a hand-written C
        inference core that reads weights from QSPI flash via XIP and streams generated text over USB CDC.
      </p>
    </div>

    <div class="section">
      <h2>Pipeline</h2>
      <div class="code-block">
        <pre><code>TinyStories → JAX train (RunPod) → micro_llama.bin → C firmware → UF2 on Shrike-Lite</code></pre>
      </div>
      <p class="section-note">
        The model is trained in JAX/Flax, exported to a single <code>micro_llama.bin</code> checkpoint, embedded into the
        firmware, and flashed to the board as a UF2 — replacing MicroPython entirely.
      </p>
    </div>

    <div class="section">
      <h2>Model Architecture</h2>
      <div class="stats-grid">
        <div class="stat">
          <div class="stat-label">Parameters</div>
          <div class="stat-value">213,312</div>
        </div>
        <div class="stat">
          <div class="stat-label">dim / hidden_dim</div>
          <div class="stat-value">64 / 128</div>
        </div>
        <div class="stat">
          <div class="stat-label">Layers</div>
          <div class="stat-value">2</div>
        </div>
        <div class="stat">
          <div class="stat-label">Attention Heads</div>
          <div class="stat-value">2Q / 2KV</div>
        </div>
        <div class="stat">
          <div class="stat-label">Context Length</div>
          <div class="stat-value">128</div>
        </div>
        <div class="stat">
          <div class="stat-label">Vocab Size</div>
          <div class="stat-value">1,024 BPE</div>
        </div>
      </div>
      <p class="section-note">
        Standard Llama building blocks: GQA (2 query heads, 2 key-value heads, head_dim 32), RoPE, RMSNorm, and a SwiGLU
        feed-forward network with an untied output head. Special tokens: pad=0, unk=1, bos=2, eos=3.
      </p>
    </div>

    <div class="section">
      <h2>The 264KB SRAM Constraint</h2>
      <p>
        Everything has to fit on a chip with 264KB of total SRAM and no FPU. The architecture is locked by that budget:
        weights live in QSPI flash via <strong>XIP</strong> (execute-in-place), while activations and the KV cache must
        fit inside SRAM at runtime. The fp32 KV cache alone occupies ~131KB — which is why the shape above is as small as
        it is. FPGA INT8 MAC acceleration on the SLG47910 (~1120 LUTs) is mapped out but not yet started.
      </p>
      <div class="training-grid">
        <div class="training-item">
          <span class="training-label">Board</span>
          <span class="training-value">Vicharak Shrike-Lite (RP2040 + Renesas FPGA)</span>
        </div>
        <div class="training-item">
          <span class="training-label">Total SRAM</span>
          <span class="training-value">264 KB</span>
        </div>
        <div class="training-item">
          <span class="training-label">Weight storage</span>
          <span class="training-value">QSPI Flash via XIP</span>
        </div>
        <div class="training-item">
          <span class="training-label">KV cache (fp32)</span>
          <span class="training-value">~131 KB</span>
        </div>
        <div class="training-item">
          <span class="training-label">Output</span>
          <span class="training-value">USB CDC serial text</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Training Details</h2>
      <div class="training-grid">
        <div class="training-item">
          <span class="training-label">GPU</span>
          <span class="training-value">1× NVIDIA RTX 3090 (RunPod)</span>
        </div>
        <div class="training-item">
          <span class="training-label">Framework</span>
          <span class="training-value">JAX / Flax</span>
        </div>
        <div class="training-item">
          <span class="training-label">Dataset</span>
          <span class="training-value">TinyStories</span>
        </div>
        <div class="training-item">
          <span class="training-label">Training Steps</span>
          <span class="training-value">5,000</span>
        </div>
        <div class="training-item">
          <span class="training-label">Eval Loss</span>
          <span class="training-value">~3.27</span>
        </div>
        <div class="training-item">
          <span class="training-label">Perplexity</span>
          <span class="training-value">~26.4</span>
        </div>
      </div>
      <p class="section-note">
        The training runtime auto-selects CUDA → Metal → CPU <em>before</em> importing JAX, so the same code trains on an
        NVIDIA GPU, an Apple Silicon Mac, or plain CPU.
      </p>
    </div>

    <div class="section">
      <h2>Firmware Targets</h2>
      <p>The C inference core builds to two targets from the same source (<code>-DML_TARGET=host|pico</code>):</p>
      <ul class="feature-list">
        <li><strong>host</strong> — native greedy decoder for parity testing against the JAX reference (<code>verify_greedy.py</code>)</li>
        <li><strong>pico</strong> — Shrike-Lite UF2 with weights + tokenizer baked into flash, streaming text over USB CDC</li>
        <li><strong>XIP weights</strong> — model weights and the BPE tokenizer are embedded in QSPI flash and read in place, never copied to SRAM</li>
        <li><strong>Soft-float RP2040</strong> — generation runs on the Cortex-M0+ without hardware floating point</li>
        <li><strong>JAX ↔ C parity</strong> — the host decoder is verified to match greedy generation bit-for-bit</li>
      </ul>
    </div>

    <div class="section">
      <h2>Quick Start</h2>
      <h3 class="subsection-title">Train (JAX / Flax)</h3>
      <div class="code-block">
        <pre><code>uv sync
uv pip install -U "jax[cuda12]"   # NVIDIA; Mac: jax-metal or CPU
cp .env.example .env              # WANDB_API_KEY, HF_TOKEN, HF_REPO_ID

uv run python train/train.py
# dry run:
uv run python train/train.py --no-wandb --no-hub --max-steps 100</code></pre>
      </div>
      <h3 class="subsection-title">Host inference + parity check</h3>
      <div class="code-block">
        <pre><code>bash firmware/scripts/download_weights.sh
uv run python firmware/scripts/embed_tokenizer.py
cmake -S firmware -B firmware/build-host -DML_TARGET=host && cmake --build firmware/build-host -j
./firmware/build-host/micro_llama_host \
  --checkpoint firmware/weights/micro_llama.bin --bos --max-new-tokens 32 --text
uv run python firmware/scripts/verify_greedy.py</code></pre>
      </div>
      <p class="section-note">
        For the Pico target, build with <code>-DML_TARGET=pico</code>, then hold BOOTSEL and run
        <code>flash_uf2.sh</code>. Open the serial console before resetting so the CDC wait captures the full session.
      </p>
    </div>

    <div class="section">
      <h2>Project Structure</h2>
      <ul class="structure-list">
        <li><code>train/train.py</code> — JAX/Flax pretraining loop with W&amp;B logging</li>
        <li><code>train/runtime.py</code> — device selection (CUDA → Metal → CPU) before importing JAX</li>
        <li><code>train/export.py</code> — Flax checkpoint → flat <code>micro_llama.bin</code></li>
        <li><code>train/generate.py</code> — greedy generation in Python</li>
        <li><code>train/hf_upload.py</code> — publish weights + tokenizer to the HuggingFace Hub</li>
        <li><code>firmware/</code> — C inference core (host CLI + RP2040 UF2)</li>
        <li><code>firmware/scripts/</code> — weight download, tokenizer embed, UF2 flash helpers</li>
      </ul>
    </div>

    <div class="section">
      <h2>Resources</h2>
      <ul class="resource-list">
        <li>
          <a href="https://github.com/weights-and-wires/vicharak-llm" target="_blank" rel="noopener noreferrer" class="resource-link">
            Source Code on GitHub
          </a>
        </li>
        <li>
          <a href="https://huggingface.co/weights-and-wires/vicharak-micro-llama" target="_blank" rel="noopener noreferrer" class="resource-link">
            Model Weights on HuggingFace
          </a>
        </li>
        <li>
          <a href="https://api.wandb.ai/links/dotslasha/9zs6fx7t" target="_blank" rel="noopener noreferrer" class="resource-link">
            Weights &amp; Biases Training Report
          </a>
        </li>
        <li>
          <a href="https://github.com/vicharak-in/shrike" target="_blank" rel="noopener noreferrer" class="resource-link">
            Vicharak Shrike-Lite Board
          </a>
        </li>
      </ul>
    </div>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 3rem;
    max-width: 48rem;
    animation: fade-up var(--dur-base) var(--ease-out-quart);
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--line-soft);
  }

  .breadcrumb {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-faint);
  }

  .breadcrumb a {
    color: var(--text-faint);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .breadcrumb a:hover {
    color: var(--signal-hi);
  }

  .separator {
    margin: 0 0.5rem;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.6;
    color: var(--text-secondary);
    margin: 0;
  }

  .project-links {
    display: flex;
    gap: 1rem;
  }

  .project-link {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    transition: color var(--dur-instant) var(--ease-out-quart),
      background-color var(--dur-instant) var(--ease-out-quart),
      border-color var(--dur-instant) var(--ease-out-quart);
    padding: 0.5rem 1rem;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
  }

  .project-link:hover {
    background: var(--ink);
    color: var(--void);
    border-color: var(--ink);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2.5rem;
  }

  .section h2 {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.02em;
    text-transform: uppercase;
    margin-bottom: 1rem;
  }

  .section h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.75rem;
  }

  .subsection-title {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
  }

  .section p {
    font-size: 0.9375rem;
    line-height: 1.7;
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .section p:last-child {
    margin-bottom: 0;
  }

  .section-note {
    margin-top: 1rem;
    font-size: 0.875rem;
    color: var(--text-faint);
  }

  .inline-link {
    color: var(--text-secondary);
    text-decoration: underline;
    text-decoration-color: var(--text-faint);
    transition: color var(--dur-instant) var(--ease-out-quart),
      text-decoration-color var(--dur-instant) var(--ease-out-quart);
  }

  .inline-link:hover {
    color: var(--signal);
    text-decoration-color: var(--signal);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    padding: 1rem;
    background: var(--panel);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
  }

  .stat-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-value {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .training-grid {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .training-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.875rem 1rem;
    background: var(--panel);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
  }

  .training-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-faint);
  }

  .training-value {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .feature-list,
  .structure-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
  }

  .feature-list li,
  .structure-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--text-secondary);
    padding-left: 1.5rem;
    position: relative;
  }

  .feature-list li::before,
  .structure-list li::before {
    content: '→';
    position: absolute;
    left: 0;
    color: var(--signal);
  }

  .resource-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
  }

  .resource-link {
    display: inline-flex;
    align-items: center;
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--text-secondary);
    padding: 0.75rem 1rem;
    background: var(--panel);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    transition: color var(--dur-instant) var(--ease-out-quart),
      background-color var(--dur-instant) var(--ease-out-quart),
      border-color var(--dur-instant) var(--ease-out-quart);
  }

  .resource-link:hover {
    background: var(--ink);
    color: var(--void);
    border-color: var(--ink);
  }

  .code-block {
    background: var(--code-bg);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    overflow-x: auto;
  }

  .code-block pre {
    margin: 0;
    padding: 1.25rem;
  }

  .code-block code {
    font-family: var(--font-mono-g);
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .structure-list code,
  .section-note code {
    font-family: var(--font-mono-g);
    font-size: 0.875rem;
    color: var(--text-secondary);
    background: var(--panel-hi);
    border: 1px solid var(--line-soft);
    padding: 0.125rem 0.375rem;
    border-radius: var(--radius-sm);
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @media (max-width: 768px) {
    .section h2 {
      font-size: 1.25rem;
    }

    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
