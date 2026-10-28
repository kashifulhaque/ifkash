<script lang="ts">
  import { reveal, splitReveal, countUp } from '$lib/motion';
</script>

<svelte:head>
  <title>smol-llama — 360M LLaMA from Scratch</title>
  <meta name="description" content="360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, SwiGLU." />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/projects">Projects</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">smol-llama</span>
    </nav>
    <h1 class="page-title" use:splitReveal>smol-llama <span class="mark">🦙</span></h1>
    <p class="lead">A 360M parameter LLaMA-style language model trained from scratch on 6B tokens. Built with GQA, RoPE, and SwiGLU on a single H100 GPU in 22 hours for $53.</p>
    <div class="btn-row">
      <a href="https://huggingface.co/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Model <span class="ext" aria-hidden="true">↗</span>
      </a>
      <a href="https://github.com/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Code <span class="ext" aria-hidden="true">↗</span>
      </a>
    </div>
  </header>

  <div class="detail-body">
    <aside class="facts" aria-label="Project facts">
      <dl use:reveal={{ children: ".fact", gap: 70 }}>
        <div class="fact">
          <dt>Parameters</dt>
          <dd>~360M</dd>
        </div>
        <div class="fact">
          <dt>Tokens</dt>
          <dd>~6B · FineWeb-6B</dd>
        </div>
        <div class="fact">
          <dt>Hardware</dt>
          <dd>1× H100 80GB</dd>
        </div>
        <div class="fact">
          <dt>Training time</dt>
          <dd>~22 h</dd>
        </div>
        <div class="fact">
          <dt>Cost</dt>
          <dd>~$53</dd>
        </div>
        <div class="fact">
          <dt>Stack</dt>
          <dd><span>PyTorch</span><span>uv</span></dd>
        </div>
        <div class="fact">
          <dt>Links</dt>
          <dd>
            <a href="https://huggingface.co/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer">Model <span aria-hidden="true">↗</span></a>
            <a href="https://github.com/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer">GitHub <span aria-hidden="true">↗</span></a>
            <a href="https://huggingface.co/datasets/weights-and-wires/fineweb-6b" target="_blank" rel="noopener noreferrer">Dataset <span aria-hidden="true">↗</span></a>
          </dd>
        </div>
      </dl>
    </aside>

    <div class="copy">
      <section class="section" use:reveal>
        <h2><span class="num">01</span>Overview</h2>
        <p>
          smol-llama is a minimal, from-scratch implementation of a LLaMA-style language model for pre-training on custom data.
          The project demonstrates that you can train a capable small language model on a reasonable budget—the entire 360M parameter
          model was trained on 6B tokens using a single NVIDIA H100 GPU in approximately 22 hours at a total cost of around $53.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">02</span>Model Architecture</h2>
        <div class="stats" use:reveal={{ children: ".stat", gap: 80 }}>
          <div class="stat">
            <span class="stat-label">Parameters</span>
            <span class="stat-value" use:countUp>~360M</span>
          </div>
          <div class="stat">
            <span class="stat-label">Hidden Dimension</span>
            <span class="stat-value" use:countUp>960</span>
          </div>
          <div class="stat">
            <span class="stat-label">Layers</span>
            <span class="stat-value" use:countUp>32</span>
          </div>
          <div class="stat">
            <span class="stat-label">Attention Heads</span>
            <span class="stat-value">15Q / 5KV</span>
          </div>
          <div class="stat">
            <span class="stat-label">Context Length</span>
            <span class="stat-value" use:countUp>2048</span>
          </div>
          <div class="stat">
            <span class="stat-label">Vocab Size</span>
            <span class="stat-value">49,152</span>
          </div>
        </div>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">03</span>Key Features</h2>
        <ul class="defs">
          <li><span class="term">Grouped Query Attention (GQA)</span><span class="desc">Efficient inference with 15 query heads and 5 key-value heads</span></li>
          <li><span class="term">RoPE</span><span class="desc">Rotary Position Embeddings for better position encoding</span></li>
          <li><span class="term">RMSNorm</span><span class="desc">Root Mean Square normalization instead of LayerNorm</span></li>
          <li><span class="term">SwiGLU</span><span class="desc">Gated Linear Unit activation in the feed-forward network</span></li>
          <li><span class="term">Flash Attention 2</span><span class="desc">Fast and memory-efficient attention with SDPA fallback</span></li>
          <li><span class="term">Gradient Checkpointing</span><span class="desc">Memory-efficient training for larger models</span></li>
          <li><span class="term">torch.compile</span><span class="desc">Optimized training speed with PyTorch 2.0 compilation</span></li>
        </ul>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">04</span>Training Details</h2>
        <dl class="kv">
          <div><dt>GPU</dt><dd>1× NVIDIA H100 (80GB PCIe)</dd></div>
          <div><dt>Training Speed</dt><dd>~75,000 tokens/sec</dd></div>
          <div><dt>Training Time</dt><dd>~22 hours (1 epoch)</dd></div>
          <div><dt>Cloud Provider</dt><dd>RunPod (~$2.40/hr)</dd></div>
          <div><dt>Total Cost</dt><dd>~$53</dd></div>
          <div><dt>Dataset</dt><dd>FineWeb-6B (~6B tokens)</dd></div>
        </dl>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">05</span>Dataset</h2>
        <p>
          The model is trained on <a href="https://huggingface.co/datasets/weights-and-wires/fineweb-6b" target="_blank" rel="noopener noreferrer">fineweb-6b</a>,
          a curated 6B token dataset pre-tokenized with a custom 49K BPE vocabulary. The dataset includes 11.3 GB of training tokens
          and 57 MB of validation tokens, all pre-processed for immediate use.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">06</span>Quick Start</h2>
        <pre><code># Install dependencies
uv sync

# Run training
uv run ./pretrain.py</code></pre>
        <p class="note">
          The training script automatically downloads the pre-tokenized dataset, initializes the model, trains with gradient accumulation and mixed precision,
          and saves checkpoints every 200 steps.
        </p>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">07</span>Using the Pre-trained Model</h2>
        <pre><code>from transformers import AutoModelForCausalLM, AutoTokenizer
import torch

# Load model and tokenizer
model = AutoModelForCausalLM.from_pretrained(
    "ifkash/smol-llama",
    torch_dtype=torch.bfloat16,
    device_map="auto"
)
tokenizer = AutoTokenizer.from_pretrained("ifkash/smol-llama")

# Generate text
prompt = "The future of artificial intelligence is"
inputs = tokenizer(prompt, return_tensors="pt").to(model.device)

outputs = model.generate(
    **inputs,
    max_new_tokens=100,
    temperature=0.7,
    top_p=0.9,
    do_sample=True,
)

print(tokenizer.decode(outputs[0], skip_special_tokens=True))</code></pre>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">08</span>Training Configuration</h2>
        <dl class="kv">
          <div><dt>Batch Size</dt><dd>64</dd></div>
          <div><dt>Context Length</dt><dd>2048 tokens</dd></div>
          <div><dt>Gradient Accumulation</dt><dd>8 steps (effective batch: 512)</dd></div>
          <div><dt>Learning Rate</dt><dd>3e-4 (peak)</dd></div>
          <div><dt>Max Iterations</dt><dd>5,725 (~6B tokens)</dd></div>
          <div><dt>Warmup Steps</dt><dd>900 iterations</dd></div>
          <div><dt>Tokens per Step</dt><dd>~1M tokens</dd></div>
        </dl>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">09</span>Project Structure</h2>
        <ul class="defs">
          <li><span class="term"><code>pretrain.py</code></span><span class="desc">Main training script with gradient accumulation and checkpointing</span></li>
          <li><span class="term"><code>utils/model.py</code></span><span class="desc">Complete LLaMA architecture implementation</span></li>
          <li><span class="term"><code>utils/rotary.py</code></span><span class="desc">Rotary position embeddings (RoPE)</span></li>
          <li><span class="term"><code>utils/data.py</code></span><span class="desc">Efficient data loading from pre-tokenized binaries</span></li>
          <li><span class="term"><code>utils/checkpoint.py</code></span><span class="desc">Checkpoint saving/loading and HuggingFace uploads</span></li>
          <li><span class="term"><code>utils/lr_schedule.py</code></span><span class="desc">Cosine learning rate schedule with warmup</span></li>
          <li><span class="term"><code>utils/logging.py</code></span><span class="desc">Weights & Biases integration for experiment tracking</span></li>
          <li><span class="term"><code>notebooks/1-train-tokenizer.ipynb</code></span><span class="desc">Custom BPE tokenizer training</span></li>
        </ul>
      </section>

      <section class="section" use:reveal>
        <h2><span class="num">10</span>Resources</h2>
        <ul class="linklist">
          <li>
            <a href="https://huggingface.co/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer">
              Pre-trained Model on HuggingFace
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a href="https://huggingface.co/datasets/weights-and-wires/fineweb-6b" target="_blank" rel="noopener noreferrer">
              FineWeb-6B Training Dataset
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a href="https://github.com/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer">
              Source Code on GitHub
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
