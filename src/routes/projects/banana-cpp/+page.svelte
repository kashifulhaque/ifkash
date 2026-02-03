<svelte:head>
  <title>banana.cpp — LLM Inference Engine</title>
  <meta name="description" content="A modular, pure C++ implementation of a small language model inference engine." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/projects">Projects</a>
      <span class="separator">/</span>
      <span>banana.cpp</span>
    </div>
    <h1 class="page-title">banana.cpp</h1>
    <p class="page-desc">A modular, pure C++ implementation of a small language model inference engine supporting multiple model architectures.</p>
    <div class="project-links">
      <a href="https://github.com/kashifulhaque/banana.cpp" target="_blank" rel="noopener noreferrer" class="project-link">
        Code
      </a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Overview</h2>
      <p>
        banana.cpp is a pure C++ LLM inference engine designed for modularity and extensibility. 
        It supports multiple small language model architectures including SmolLM2, Llama 3.2, and Qwen variants,
        with built-in support for FP16/BF16 precision and modern architectural features like GQA, RoPE, and SwiGLU.
      </p>
    </div>

    <div class="section">
      <h2>Supported Models</h2>
      <ul class="model-list">
        <li><strong>SmolLM2</strong>: 135M, 360M, 1.7B parameter variants</li>
        <li><strong>Llama 3.2</strong>: 1B, 3B parameter variants</li>
        <li><strong>Qwen</strong>: 2.5-0.5B, 3-0.6B parameter variants</li>
      </ul>
    </div>

    <div class="section">
      <h2>Key Features</h2>
      <ul class="feature-list">
        <li><strong>Modular Architecture</strong>: Layer-based design makes it easy to add new architectures</li>
        <li><strong>Mixed Precision</strong>: Native FP16 and BF16 support for efficient inference</li>
        <li><strong>Modern Attention</strong>: GQA, MHA, and MQA attention mechanisms</li>
        <li><strong>Position Encodings</strong>: RoPE (Rotary Position Embedding) implementation</li>
        <li><strong>Activation Functions</strong>: SwiGLU and standard MLP layers</li>
        <li><strong>Normalization</strong>: RMSNorm and LayerNorm implementations</li>
        <li><strong>Auto-detection</strong>: Model registry automatically detects model type from config</li>
        <li><strong>HuggingFace Integration</strong>: Built-in model downloader for HuggingFace models</li>
      </ul>
    </div>

    <div class="section">
      <h2>Architecture</h2>
      <p>The engine uses a clean, modular layer-based architecture:</p>
      <ul class="arch-list">
        <li><strong>Layers</strong>: Reusable components (Attention, MLP, Normalization, RoPE)</li>
        <li><strong>Models</strong>: Compose layers based on configuration</li>
        <li><strong>Tokenizers</strong>: Separate BPE logic from chat template handling</li>
        <li><strong>Registry</strong>: Auto-detect model type from config.json</li>
      </ul>
      <p class="arch-desc">
        This design philosophy makes it straightforward to add new model architectures by composing existing layers,
        introduce new layer types without modifying models, and support multiple chat templates and tokenizer formats.
      </p>
    </div>

    <div class="section">
      <h2>Usage</h2>
      <div class="code-block">
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
      </div>
    </div>

    <div class="section">
      <h2>Building</h2>
      <div class="code-block">
        <pre><code>mkdir build && cd build
cmake ..
make -j$(nproc)</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Project Structure</h2>
      <ul class="structure-list">
        <li><code>include/config/</code>: Model and tokenizer configuration structures</li>
        <li><code>include/core/</code>: Core tensor data structures and operations</li>
        <li><code>include/layers/</code>: Neural network layer implementations</li>
        <li><code>include/models/</code>: Model architecture definitions</li>
        <li><code>include/tokenizers/</code>: BPE tokenization and chat templates</li>
        <li><code>include/registry/</code>: Model auto-detection system</li>
        <li><code>include/utils/</code>: Model loading and HuggingFace downloading</li>
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
    animation: fade-up var(--duration-slow) var(--ease-out);
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--gray-800);
  }

  .breadcrumb {
    font-size: 0.875rem;
    color: var(--gray-500);
  }

  .breadcrumb a {
    color: var(--gray-500);
    transition: color var(--duration-fast) var(--ease-out);
  }

  .breadcrumb a:hover {
    color: var(--white);
  }

  .separator {
    margin: 0 0.5rem;
  }

  .page-title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--white);
    margin: 0;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.6;
    color: var(--gray-400);
    margin: 0;
  }

  .project-links {
    display: flex;
    gap: 1rem;
  }

  .project-link {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--gray-600);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    transition: color var(--duration-fast) var(--ease-out);
    padding: 0.5rem 1rem;
    border: 1px solid var(--gray-800);
    border-radius: 0.375rem;
  }

  .project-link:hover {
    color: var(--white);
    border-color: var(--gray-700);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2.5rem;
  }

  .section h2 {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--white);
    letter-spacing: -0.01em;
    margin-bottom: 1rem;
  }

  .section p {
    font-size: 0.9375rem;
    line-height: 1.7;
    color: var(--gray-400);
    margin-bottom: 1rem;
  }

  .section p:last-child {
    margin-bottom: 0;
  }

  .model-list,
  .feature-list,
  .arch-list,
  .structure-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
  }

  .model-list li,
  .feature-list li,
  .arch-list li,
  .structure-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--gray-400);
    padding-left: 1.5rem;
    position: relative;
  }

  .model-list li::before,
  .feature-list li::before,
  .arch-list li::before,
  .structure-list li::before {
    content: '→';
    position: absolute;
    left: 0;
    color: var(--gray-600);
  }

  .arch-desc {
    margin-top: 1rem;
    padding: 1rem;
    background: var(--gray-950);
    border-left: 2px solid var(--gray-800);
    border-radius: 0.25rem;
  }

  .code-block {
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
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
    color: var(--gray-300);
  }

  .structure-list code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.875rem;
    color: var(--gray-300);
    background: var(--gray-950);
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
  }
</style>
