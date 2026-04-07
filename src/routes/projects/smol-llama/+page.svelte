<svelte:head>
  <title>smol-llama — 360M LLaMA from Scratch</title>
  <meta name="description" content="360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, SwiGLU." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/projects">Projects</a>
      <span class="separator">/</span>
      <span>smol-llama</span>
    </div>
    <h1 class="page-title">smol-llama 🦙</h1>
    <p class="page-desc">A 360M parameter LLaMA-style language model trained from scratch on 6B tokens. Built with GQA, RoPE, and SwiGLU on a single H100 GPU in 22 hours for $53.</p>
    <div class="project-links">
      <a href="https://huggingface.co/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer" class="project-link">
        Model
      </a>
      <a href="https://github.com/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer" class="project-link">
        Code
      </a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Overview</h2>
      <p>
        smol-llama is a minimal, from-scratch implementation of a LLaMA-style language model for pre-training on custom data. 
        The project demonstrates that you can train a capable small language model on a reasonable budget—the entire 360M parameter 
        model was trained on 6B tokens using a single NVIDIA H100 GPU in approximately 22 hours at a total cost of around $53.
      </p>
    </div>

    <div class="section">
      <h2>Model Architecture</h2>
      <div class="stats-grid">
        <div class="stat">
          <div class="stat-label">Parameters</div>
          <div class="stat-value">~360M</div>
        </div>
        <div class="stat">
          <div class="stat-label">Hidden Dimension</div>
          <div class="stat-value">960</div>
        </div>
        <div class="stat">
          <div class="stat-label">Layers</div>
          <div class="stat-value">32</div>
        </div>
        <div class="stat">
          <div class="stat-label">Attention Heads</div>
          <div class="stat-value">15Q / 5KV</div>
        </div>
        <div class="stat">
          <div class="stat-label">Context Length</div>
          <div class="stat-value">2048</div>
        </div>
        <div class="stat">
          <div class="stat-label">Vocab Size</div>
          <div class="stat-value">49,152</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Key Features</h2>
      <ul class="feature-list">
        <li><strong>Grouped Query Attention (GQA)</strong>: Efficient inference with 15 query heads and 5 key-value heads</li>
        <li><strong>RoPE</strong>: Rotary Position Embeddings for better position encoding</li>
        <li><strong>RMSNorm</strong>: Root Mean Square normalization instead of LayerNorm</li>
        <li><strong>SwiGLU</strong>: Gated Linear Unit activation in the feed-forward network</li>
        <li><strong>Flash Attention 2</strong>: Fast and memory-efficient attention with SDPA fallback</li>
        <li><strong>Gradient Checkpointing</strong>: Memory-efficient training for larger models</li>
        <li><strong>torch.compile</strong>: Optimized training speed with PyTorch 2.0 compilation</li>
      </ul>
    </div>

    <div class="section">
      <h2>Training Details</h2>
      <div class="training-grid">
        <div class="training-item">
          <span class="training-label">GPU</span>
          <span class="training-value">1× NVIDIA H100 (80GB PCIe)</span>
        </div>
        <div class="training-item">
          <span class="training-label">Training Speed</span>
          <span class="training-value">~75,000 tokens/sec</span>
        </div>
        <div class="training-item">
          <span class="training-label">Training Time</span>
          <span class="training-value">~22 hours (1 epoch)</span>
        </div>
        <div class="training-item">
          <span class="training-label">Cloud Provider</span>
          <span class="training-value">RunPod (~$2.40/hr)</span>
        </div>
        <div class="training-item">
          <span class="training-label">Total Cost</span>
          <span class="training-value">~$53</span>
        </div>
        <div class="training-item">
          <span class="training-label">Dataset</span>
          <span class="training-value">FineWeb-6B (~6B tokens)</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Dataset</h2>
      <p>
        The model is trained on <a href="https://huggingface.co/datasets/weights-and-wires/fineweb-6b" target="_blank" rel="noopener noreferrer" class="inline-link">fineweb-6b</a>, 
        a curated 6B token dataset pre-tokenized with a custom 49K BPE vocabulary. The dataset includes 11.3 GB of training tokens 
        and 57 MB of validation tokens, all pre-processed for immediate use.
      </p>
    </div>

    <div class="section">
      <h2>Quick Start</h2>
      <div class="code-block">
        <pre><code># Install dependencies
uv sync

# Run training
uv run ./pretrain.py</code></pre>
      </div>
      <p class="section-note">
        The training script automatically downloads the pre-tokenized dataset, initializes the model, trains with gradient accumulation and mixed precision, 
        and saves checkpoints every 200 steps.
      </p>
    </div>

    <div class="section">
      <h2>Using the Pre-trained Model</h2>
      <div class="code-block">
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
      </div>
    </div>

    <div class="section">
      <h2>Training Configuration</h2>
      <div class="config-list">
        <div class="config-item">
          <span class="config-key">Batch Size</span>
          <span class="config-value">64</span>
        </div>
        <div class="config-item">
          <span class="config-key">Context Length</span>
          <span class="config-value">2048 tokens</span>
        </div>
        <div class="config-item">
          <span class="config-key">Gradient Accumulation</span>
          <span class="config-value">8 steps (effective batch: 512)</span>
        </div>
        <div class="config-item">
          <span class="config-key">Learning Rate</span>
          <span class="config-value">3e-4 (peak)</span>
        </div>
        <div class="config-item">
          <span class="config-key">Max Iterations</span>
          <span class="config-value">5,725 (~6B tokens)</span>
        </div>
        <div class="config-item">
          <span class="config-key">Warmup Steps</span>
          <span class="config-value">900 iterations</span>
        </div>
        <div class="config-item">
          <span class="config-key">Tokens per Step</span>
          <span class="config-value">~1M tokens</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Project Structure</h2>
      <ul class="structure-list">
        <li><code>pretrain.py</code>: Main training script with gradient accumulation and checkpointing</li>
        <li><code>utils/model.py</code>: Complete LLaMA architecture implementation</li>
        <li><code>utils/rotary.py</code>: Rotary position embeddings (RoPE)</li>
        <li><code>utils/data.py</code>: Efficient data loading from pre-tokenized binaries</li>
        <li><code>utils/checkpoint.py</code>: Checkpoint saving/loading and HuggingFace uploads</li>
        <li><code>utils/lr_schedule.py</code>: Cosine learning rate schedule with warmup</li>
        <li><code>utils/logging.py</code>: Weights & Biases integration for experiment tracking</li>
        <li><code>notebooks/1-train-tokenizer.ipynb</code>: Custom BPE tokenizer training</li>
      </ul>
    </div>

    <div class="section">
      <h2>Resources</h2>
      <ul class="resource-list">
        <li>
          <a href="https://huggingface.co/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer" class="resource-link">
            Pre-trained Model on HuggingFace
          </a>
        </li>
        <li>
          <a href="https://huggingface.co/datasets/weights-and-wires/fineweb-6b" target="_blank" rel="noopener noreferrer" class="resource-link">
            FineWeb-6B Training Dataset
          </a>
        </li>
        <li>
          <a href="https://github.com/weights-and-wires/smol-llama" target="_blank" rel="noopener noreferrer" class="resource-link">
            Source Code on GitHub
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
    border-bottom: 1px solid var(--border);
  }

  .breadcrumb {
    font-size: 0.875rem;
    color: var(--text-tertiary);
  }

  .breadcrumb a {
    color: var(--text-tertiary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .breadcrumb a:hover {
    color: var(--text-primary);
  }

  .separator {
    margin: 0 0.5rem;
  }

  .page-title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin: 0;
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
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    transition: color var(--dur-instant) var(--ease-out-quart);
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
  }

  .project-link:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
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
    letter-spacing: -0.01em;
    margin-bottom: 1rem;
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
    color: var(--text-tertiary);
  }

  .inline-link {
    color: var(--text-secondary);
    text-decoration: underline;
    text-decoration-color: var(--text-faint);
    transition: all var(--dur-instant) var(--ease-out-quart);
  }

  .inline-link:hover {
    color: var(--text-primary);
    text-decoration-color: var(--text-tertiary);
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
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
  }

  .stat-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-tertiary);
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
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
  }

  .training-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-tertiary);
  }

  .training-value {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .config-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .config-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--surface-raised);
    border-left: 2px solid var(--border);
  }

  .config-key {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .config-value {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
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
    color: var(--text-faint);
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
    font-size: 0.9375rem;
    color: var(--text-secondary);
    padding: 0.75rem 1rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    transition: all var(--dur-instant) var(--ease-out-quart);
  }

  .resource-link:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
    transform: translateX(4px);
  }

  .code-block {
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow-x: auto;
  }

  .code-block pre {
    margin: 0;
    padding: 1.25rem;
  }

  .code-block code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .structure-list code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.875rem;
    color: var(--text-secondary);
    background: var(--surface-raised);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @media (max-width: 768px) {
    .page-title {
      font-size: 2rem;
    }

    .section h2 {
      font-size: 1.25rem;
    }

    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
