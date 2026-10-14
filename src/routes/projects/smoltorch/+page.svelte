<svelte:head>
  <title>smoltorch — Autograd Engine & Neural Networks</title>
  <meta name="description" content="Autograd engine and neural networks in ~500 lines of NumPy. Educational deep learning." />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/projects">Projects</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">smoltorch</span>
    </nav>
    <h1 class="page-title">smoltorch <span class="mark">🔥</span></h1>
    <p class="lead">A tiny autograd engine and neural network library built from first principles. Implements automatic differentiation and deep learning in ~500 lines of readable Python code using only NumPy.</p>
    <div class="btn-row">
      <a href="https://github.com/kashifulhaque/smoltorch" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Code <span class="ext" aria-hidden="true">↗</span>
      </a>
      <a href="https://pypi.org/project/smoltorch/" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        PyPI <span class="ext" aria-hidden="true">↗</span>
      </a>
      <a href="https://blog.ifkash.dev/smoltorch" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Blog <span class="ext" aria-hidden="true">↗</span>
      </a>
    </div>
  </header>

  <div class="detail-body">
    <aside class="facts" aria-label="Project facts">
      <dl>
        <div class="fact">
          <dt>Stack</dt>
          <dd><span>Python</span><span>NumPy</span></dd>
        </div>
        <div class="fact">
          <dt>Size</dt>
          <dd>~500 lines</dd>
        </div>
        <div class="fact">
          <dt>Best result</dt>
          <dd>96.5% · Breast Cancer</dd>
        </div>
        <div class="fact">
          <dt>Inspired by</dt>
          <dd>micrograd</dd>
        </div>
        <div class="fact">
          <dt>Install</dt>
          <dd>uv add smoltorch</dd>
        </div>
        <div class="fact">
          <dt>Links</dt>
          <dd>
            <a href="https://github.com/kashifulhaque/smoltorch" target="_blank" rel="noopener noreferrer">GitHub <span aria-hidden="true">↗</span></a>
            <a href="https://pypi.org/project/smoltorch/" target="_blank" rel="noopener noreferrer">PyPI <span aria-hidden="true">↗</span></a>
            <a href="https://blog.ifkash.dev/smoltorch" target="_blank" rel="noopener noreferrer">Blog <span aria-hidden="true">↗</span></a>
          </dd>
        </div>
      </dl>
    </aside>

    <div class="copy">
      <section class="section">
        <h2><span class="num">01</span>Overview</h2>
        <p>
          smoltorch is a minimalist deep learning library that implements automatic differentiation (autograd) and neural networks
          from scratch using only NumPy. Inspired by Andrej Karpathy's micrograd, it's designed to be educational, transparent,
          and functional—you can train real models on real datasets with competitive performance.
        </p>
        <p>
          The name comes from "Smol" + PyTorch: a tiny implementation that captures the essence of modern deep learning frameworks.
        </p>
      </section>

      <section class="section">
        <h2><span class="num">02</span>Design Philosophy</h2>
        <div class="tiles">
          <div class="tile">
            <h3>Educational</h3>
            <p>Understand how modern deep learning frameworks work under the hood</p>
          </div>
          <div class="tile">
            <h3>Transparent</h3>
            <p>Every operation is visible and understandable</p>
          </div>
          <div class="tile">
            <h3>Functional</h3>
            <p>Train real models on real datasets with competitive performance</p>
          </div>
          <div class="tile">
            <h3>Minimal</h3>
            <p>~500 lines of readable, well-documented Python code</p>
          </div>
        </div>
      </section>

      <section class="section">
        <h2><span class="num">03</span>Features</h2>

        <h3>Core Engine</h3>
        <ul class="ruled">
          <li>Automatic differentiation with dynamic computational graphs</li>
          <li>NumPy-backed tensors for efficient numerical computing</li>
          <li>Broadcasting support with proper gradient handling</li>
          <li>Topological sorting for correct backpropagation</li>
        </ul>

        <h3>Operations</h3>
        <ul class="defs">
          <li><span class="term">Arithmetic</span><span class="desc">+, -, *, /, **</span></li>
          <li><span class="term">Matrix operations</span><span class="desc">@ (matmul)</span></li>
          <li><span class="term">Activations</span><span class="desc">ReLU, tanh, sigmoid</span></li>
          <li><span class="term">Reductions</span><span class="desc">sum, mean</span></li>
          <li><span class="term">Element-wise</span><span class="desc">log</span></li>
        </ul>

        <h3>Neural Networks</h3>
        <ul class="defs">
          <li><span class="term">Layers</span><span class="desc">Linear (fully connected)</span></li>
          <li><span class="term">Models</span><span class="desc">Multi-layer perceptron (MLP)</span></li>
          <li><span class="term">Loss functions</span><span class="desc">MSE, Binary Cross-Entropy</span></li>
          <li><span class="term">Optimizers</span><span class="desc">SGD (Stochastic Gradient Descent)</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">04</span>Performance</h2>
        <p>smoltorch achieves competitive results on standard benchmarks:</p>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th scope="col">Dataset</th>
                <th scope="col">Task</th>
                <th scope="col" class="r">Accuracy</th>
                <th scope="col" class="r">Epochs</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td class="strong">Breast Cancer</td>
                <td>Binary Classification</td>
                <td class="mono r">96.5%</td>
                <td class="mono r">200</td>
              </tr>
              <tr>
                <td class="strong">Synthetic Regression</td>
                <td>Regression</td>
                <td class="mono r">MSE: 95.7</td>
                <td class="mono r">100</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section class="section">
        <h2><span class="num">05</span>Installation</h2>
        <pre><code>uv add smoltorch</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">06</span>Quick Start</h2>

        <h3>Basic Tensor Operations</h3>
        <pre><code>from smoltorch import Tensor

# Create tensors
x = Tensor([1.0, 2.0, 3.0])
y = Tensor([4.0, 5.0, 6.0])

# Operations
z = x + y           # Element-wise addition
w = x * y           # Element-wise multiplication
a = x @ y.T         # Matrix multiplication

# Backward pass
a.backward()
print(x.grad)       # Gradients computed automatically!</code></pre>

        <h3>Training a Neural Network</h3>
        <pre><code>from smoltorch import Tensor, MLP, SGD
from sklearn.datasets import make_regression
import numpy as np

# Generate data
X, y = make_regression(n_samples=100, n_features=5, noise=10)
y = y.reshape(-1, 1)

# Create model
model = MLP([5, 16, 16, 1])  # 5 inputs -> 16 -> 16 -> 1 output
optimizer = SGD(model.parameters(), lr=0.001)

# Training loop
for epoch in range(100):
    # Forward pass
    X_tensor = Tensor(X)
    y_tensor = Tensor(y)
    y_pred = model(X_tensor)

    # Compute loss (MSE)
    loss = ((y_pred - y_tensor) ** 2).mean()

    # Backward pass
    optimizer.zero_grad()
    loss.backward()

    # Update weights
    optimizer.step()

    if (epoch + 1) % 10 == 0:
        print(f"Epoch {'{'}epoch + 1{'}'}, Loss: {'{'}loss.data:.4f{'}'}\")</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">07</span>How Autograd Works</h2>
        <p>smoltorch builds a dynamic computational graph during the forward pass:</p>
        <ol class="steps">
          <li><span><strong>Forward pass</strong>: Build computational graph with operations as nodes</span></li>
          <li><span><strong>Topological sort</strong>: Order nodes for correct gradient flow</span></li>
          <li><span><strong>Backward pass</strong>: Apply chain rule in reverse topological order</span></li>
          <li><span><strong>Gradient accumulation</strong>: Sum gradients from multiple paths</span></li>
        </ol>

        <pre><code>x = Tensor([2.0])
y = Tensor([3.0])
z = (x * y) + (x ** 2)  # Graph: z -> [+] -> [*, **] -> [x, y]

z.backward()  # Backpropagate through graph
print(x.grad)  # dz/dx = y + 2x = 3 + 4 = 7.0</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">08</span>Broadcasting Support</h2>
        <p>smoltorch correctly handles broadcasting in both forward and backward passes:</p>
        <pre><code>x = Tensor([[1, 2, 3]])    # shape (1, 3)
y = Tensor([[1], [2]])      # shape (2, 1)
z = x + y                   # shape (2, 3) - broadcasting!

z.backward()
# x.grad sums over broadcast dimensions: shape (1, 3)
# y.grad sums over broadcast dimensions: shape (2, 1)</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">09</span>Supported Operations</h2>
        <ul class="defs">
          <li>
            <span class="term">Element-wise</span>
            <span class="chips">
              <code>x + y</code>
              <code>x - y</code>
              <code>x * y</code>
              <code>x / y</code>
              <code>x ** 2</code>
            </span>
          </li>
          <li>
            <span class="term">Matrix Operations</span>
            <span class="chips">
              <code>x @ y</code>
            </span>
          </li>
          <li>
            <span class="term">Activations</span>
            <span class="chips">
              <code>x.relu()</code>
              <code>x.tanh()</code>
              <code>x.sigmoid()</code>
            </span>
          </li>
          <li>
            <span class="term">Reductions</span>
            <span class="chips">
              <code>x.sum()</code>
              <code>x.sum(axis=0)</code>
              <code>x.mean()</code>
              <code>x.mean(axis=1)</code>
            </span>
          </li>
          <li>
            <span class="term">Other</span>
            <span class="chips">
              <code>x.log()</code>
            </span>
          </li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">10</span>Project Structure</h2>
        <ul class="defs">
          <li><span class="term"><code>smoltorch/tensor.py</code></span><span class="desc">Core Tensor class with autograd implementation</span></li>
          <li><span class="term"><code>smoltorch/nn.py</code></span><span class="desc">Neural network layers and models (Linear, MLP)</span></li>
          <li><span class="term"><code>smoltorch/optim.py</code></span><span class="desc">Optimizers (SGD)</span></li>
          <li><span class="term"><code>examples/train_regression.py</code></span><span class="desc">Regression training example</span></li>
          <li><span class="term"><code>examples/train_classification.py</code></span><span class="desc">Classification training example</span></li>
          <li><span class="term"><code>tests/</code></span><span class="desc">Comprehensive test suite covering all operations</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">11</span>Roadmap</h2>

        <h3>Coming Soon</h3>
        <ul class="ruled">
          <li>More optimizers: Adam, RMSprop with momentum</li>
          <li>More activations: Leaky ReLU, ELU, Softmax</li>
          <li>Regularization: Dropout, L2 weight decay</li>
          <li>Mini-batch training: Efficient batch processing</li>
          <li>Multi-class classification: Softmax + Cross-Entropy loss</li>
        </ul>

        <h3>Future</h3>
        <ul class="ruled">
          <li>Convolutional layers: CNN support for images</li>
          <li>Model serialization: Save/load weights in safetensors format</li>
          <li>GPU acceleration: Explore Metal Performance Shaders for Apple Silicon</li>
          <li>Better initialization: He initialization for ReLU networks</li>
          <li>Learning rate scheduling: Decay strategies</li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">12</span>Learning Resources</h2>
        <ul class="linklist">
          <li>
            <a href="https://github.com/karpathy/micrograd" target="_blank" rel="noopener noreferrer">
              Andrej Karpathy's micrograd — The original inspiration
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a href="https://www.youtube.com/playlist?list=PLAqhIrjkxbuWI23v9cThsA9GvCAUhRvKZ" target="_blank" rel="noopener noreferrer">
              Neural Networks: Zero to Hero — Video series by Andrej Karpathy
              <span class="arrow" aria-hidden="true">↗</span>
            </a>
          </li>
          <li>
            <a href="https://arxiv.org/abs/1802.01528" target="_blank" rel="noopener noreferrer">
              The Matrix Calculus You Need For Deep Learning — Paper on backpropagation math
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

  /* Plain hairline-ruled rows */
  .ruled {
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }

  .ruled > li {
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
    color: var(--ink-2);
    line-height: 1.6;
  }

  /* Hairline table */
  .table-wrap {
    overflow-x: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
  }

  table {
    width: 100%;
    min-width: 32rem;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  th {
    padding: var(--s-2) var(--s-4);
    border-bottom: 1px solid var(--line);
    background: var(--surface-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    text-align: left;
    white-space: nowrap;
  }

  td {
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--line);
    color: var(--ink-2);
    line-height: 1.5;
    vertical-align: top;
  }

  tbody tr:last-child td {
    border-bottom: 0;
  }

  td.strong {
    color: var(--ink);
    font-weight: 500;
  }

  td.mono {
    color: var(--ink);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
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

  th.r,
  td.r {
    text-align: right;
  }

  /* Numbered hairline steps */
  .steps {
    padding: 0;
    list-style: none;
    counter-reset: step;
    border-top: 1px solid var(--line);
  }

  .steps > li {
    display: grid;
    grid-template-columns: var(--s-7) minmax(0, 1fr);
    padding: var(--s-3) 0;
    border-bottom: 1px solid var(--line);
    color: var(--ink-2);
    line-height: 1.55;
    counter-increment: step;
  }

  .steps > li::before {
    content: counter(step, decimal-leading-zero);
    padding-top: 0.2em;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
</style>
