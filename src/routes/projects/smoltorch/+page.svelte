<svelte:head>
  <title>smoltorch — Autograd Engine & Neural Networks</title>
  <meta name="description" content="Autograd engine and neural networks in ~500 lines of NumPy. Educational deep learning." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/projects">Projects</a>
      <span class="separator">/</span>
      <span>smoltorch</span>
    </div>
    <h1 class="page-title">smoltorch 🔥</h1>
    <p class="page-desc">A tiny autograd engine and neural network library built from first principles. Implements automatic differentiation and deep learning in ~500 lines of readable Python code using only NumPy.</p>
    <div class="project-links">
      <a href="https://github.com/kashifulhaque/smoltorch" target="_blank" rel="noopener noreferrer" class="project-link">
        Code
      </a>
      <a href="https://pypi.org/project/smoltorch/" target="_blank" rel="noopener noreferrer" class="project-link">
        PyPI
      </a>
      <a href="https://blog.ifkash.dev/smoltorch" target="_blank" rel="noopener noreferrer" class="project-link">
        Blog
      </a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Overview</h2>
      <p>
        smoltorch is a minimalist deep learning library that implements automatic differentiation (autograd) and neural networks 
        from scratch using only NumPy. Inspired by Andrej Karpathy's micrograd, it's designed to be educational, transparent, 
        and functional—you can train real models on real datasets with competitive performance.
      </p>
      <p>
        The name comes from "Smol" + PyTorch: a tiny implementation that captures the essence of modern deep learning frameworks.
      </p>
    </div>

    <div class="section">
      <h2>Design Philosophy</h2>
      <div class="philosophy-grid">
        <div class="philosophy-item">
          <div class="philosophy-icon">📚</div>
          <div class="philosophy-title">Educational</div>
          <div class="philosophy-desc">Understand how modern deep learning frameworks work under the hood</div>
        </div>
        <div class="philosophy-item">
          <div class="philosophy-icon">🔍</div>
          <div class="philosophy-title">Transparent</div>
          <div class="philosophy-desc">Every operation is visible and understandable</div>
        </div>
        <div class="philosophy-item">
          <div class="philosophy-icon">⚡</div>
          <div class="philosophy-title">Functional</div>
          <div class="philosophy-desc">Train real models on real datasets with competitive performance</div>
        </div>
        <div class="philosophy-item">
          <div class="philosophy-icon">✨</div>
          <div class="philosophy-title">Minimal</div>
          <div class="philosophy-desc">~500 lines of readable, well-documented Python code</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Features</h2>
      <div class="features-section">
        <div class="feature-category">
          <h3>Core Engine</h3>
          <ul class="feature-list">
            <li>Automatic differentiation with dynamic computational graphs</li>
            <li>NumPy-backed tensors for efficient numerical computing</li>
            <li>Broadcasting support with proper gradient handling</li>
            <li>Topological sorting for correct backpropagation</li>
          </ul>
        </div>
        <div class="feature-category">
          <h3>Operations</h3>
          <ul class="feature-list">
            <li><strong>Arithmetic</strong>: +, -, *, /, **</li>
            <li><strong>Matrix operations</strong>: @ (matmul)</li>
            <li><strong>Activations</strong>: ReLU, tanh, sigmoid</li>
            <li><strong>Reductions</strong>: sum, mean</li>
            <li><strong>Element-wise</strong>: log</li>
          </ul>
        </div>
        <div class="feature-category">
          <h3>Neural Networks</h3>
          <ul class="feature-list">
            <li><strong>Layers</strong>: Linear (fully connected)</li>
            <li><strong>Models</strong>: Multi-layer perceptron (MLP)</li>
            <li><strong>Loss functions</strong>: MSE, Binary Cross-Entropy</li>
            <li><strong>Optimizers</strong>: SGD (Stochastic Gradient Descent)</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Performance</h2>
      <p>smoltorch achieves competitive results on standard benchmarks:</p>
      <div class="performance-table">
        <div class="perf-row perf-header">
          <div class="perf-cell">Dataset</div>
          <div class="perf-cell">Task</div>
          <div class="perf-cell">Accuracy</div>
          <div class="perf-cell">Epochs</div>
        </div>
        <div class="perf-row">
          <div class="perf-cell">Breast Cancer</div>
          <div class="perf-cell">Binary Classification</div>
          <div class="perf-cell">96.5%</div>
          <div class="perf-cell">200</div>
        </div>
        <div class="perf-row">
          <div class="perf-cell">Synthetic Regression</div>
          <div class="perf-cell">Regression</div>
          <div class="perf-cell">MSE: 95.7</div>
          <div class="perf-cell">100</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Installation</h2>
      <div class="code-block">
        <pre><code>uv add smoltorch</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Quick Start</h2>
      
      <h3 class="subsection-title">Basic Tensor Operations</h3>
      <div class="code-block">
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
      </div>

      <h3 class="subsection-title">Training a Neural Network</h3>
      <div class="code-block">
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
      </div>
    </div>

    <div class="section">
      <h2>How Autograd Works</h2>
      <p>smoltorch builds a dynamic computational graph during the forward pass:</p>
      <ol class="autograd-steps">
        <li><strong>Forward pass</strong>: Build computational graph with operations as nodes</li>
        <li><strong>Topological sort</strong>: Order nodes for correct gradient flow</li>
        <li><strong>Backward pass</strong>: Apply chain rule in reverse topological order</li>
        <li><strong>Gradient accumulation</strong>: Sum gradients from multiple paths</li>
      </ol>
      
      <div class="code-block">
        <pre><code>x = Tensor([2.0])
y = Tensor([3.0])
z = (x * y) + (x ** 2)  # Graph: z -> [+] -> [*, **] -> [x, y]

z.backward()  # Backpropagate through graph
print(x.grad)  # dz/dx = y + 2x = 3 + 4 = 7.0</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Broadcasting Support</h2>
      <p>smoltorch correctly handles broadcasting in both forward and backward passes:</p>
      <div class="code-block">
        <pre><code>x = Tensor([[1, 2, 3]])    # shape (1, 3)
y = Tensor([[1], [2]])      # shape (2, 1)
z = x + y                   # shape (2, 3) - broadcasting!

z.backward()
# x.grad sums over broadcast dimensions: shape (1, 3)
# y.grad sums over broadcast dimensions: shape (2, 1)</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Supported Operations</h2>
      <div class="ops-grid">
        <div class="ops-category">
          <div class="ops-title">Element-wise</div>
          <div class="ops-list">
            <code>x + y</code>
            <code>x - y</code>
            <code>x * y</code>
            <code>x / y</code>
            <code>x ** 2</code>
          </div>
        </div>
        <div class="ops-category">
          <div class="ops-title">Matrix Operations</div>
          <div class="ops-list">
            <code>x @ y</code>
          </div>
        </div>
        <div class="ops-category">
          <div class="ops-title">Activations</div>
          <div class="ops-list">
            <code>x.relu()</code>
            <code>x.tanh()</code>
            <code>x.sigmoid()</code>
          </div>
        </div>
        <div class="ops-category">
          <div class="ops-title">Reductions</div>
          <div class="ops-list">
            <code>x.sum()</code>
            <code>x.sum(axis=0)</code>
            <code>x.mean()</code>
            <code>x.mean(axis=1)</code>
          </div>
        </div>
        <div class="ops-category">
          <div class="ops-title">Other</div>
          <div class="ops-list">
            <code>x.log()</code>
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Project Structure</h2>
      <ul class="structure-list">
        <li><code>smoltorch/tensor.py</code>: Core Tensor class with autograd implementation</li>
        <li><code>smoltorch/nn.py</code>: Neural network layers and models (Linear, MLP)</li>
        <li><code>smoltorch/optim.py</code>: Optimizers (SGD)</li>
        <li><code>examples/train_regression.py</code>: Regression training example</li>
        <li><code>examples/train_classification.py</code>: Classification training example</li>
        <li><code>tests/</code>: Comprehensive test suite covering all operations</li>
      </ul>
    </div>

    <div class="section">
      <h2>Roadmap</h2>
      <div class="roadmap-section">
        <div class="roadmap-category">
          <h3>Coming Soon</h3>
          <ul class="roadmap-list">
            <li>More optimizers: Adam, RMSprop with momentum</li>
            <li>More activations: Leaky ReLU, ELU, Softmax</li>
            <li>Regularization: Dropout, L2 weight decay</li>
            <li>Mini-batch training: Efficient batch processing</li>
            <li>Multi-class classification: Softmax + Cross-Entropy loss</li>
          </ul>
        </div>
        <div class="roadmap-category">
          <h3>Future</h3>
          <ul class="roadmap-list">
            <li>Convolutional layers: CNN support for images</li>
            <li>Model serialization: Save/load weights in safetensors format</li>
            <li>GPU acceleration: Explore Metal Performance Shaders for Apple Silicon</li>
            <li>Better initialization: He initialization for ReLU networks</li>
            <li>Learning rate scheduling: Decay strategies</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Learning Resources</h2>
      <ul class="resource-list">
        <li>
          <a href="https://github.com/karpathy/micrograd" target="_blank" rel="noopener noreferrer" class="resource-link">
            Andrej Karpathy's micrograd — The original inspiration
          </a>
        </li>
        <li>
          <a href="https://www.youtube.com/playlist?list=PLAqhIrjkxbuWI23v9cThsA9GvCAUhRvKZ" target="_blank" rel="noopener noreferrer" class="resource-link">
            Neural Networks: Zero to Hero — Video series by Andrej Karpathy
          </a>
        </li>
        <li>
          <a href="https://arxiv.org/abs/1802.01528" target="_blank" rel="noopener noreferrer" class="resource-link">
            The Matrix Calculus You Need For Deep Learning — Paper on backpropagation math
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

  .section h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--white);
    margin-bottom: 0.75rem;
  }

  .subsection-title {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
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

  .philosophy-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1.25rem;
  }

  .philosophy-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1.25rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .philosophy-icon {
    font-size: 2rem;
    line-height: 1;
  }

  .philosophy-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
  }

  .philosophy-desc {
    font-size: 0.875rem;
    line-height: 1.5;
    color: var(--gray-500);
  }

  .features-section {
    display: grid;
    gap: 1.5rem;
  }

  .feature-category h3 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
    margin-bottom: 0.75rem;
  }

  .feature-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    list-style: none;
    padding: 0;
  }

  .feature-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--gray-400);
    padding-left: 1.5rem;
    position: relative;
  }

  .feature-list li::before {
    content: '→';
    position: absolute;
    left: 0;
    color: var(--gray-600);
  }

  .performance-table {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .perf-row {
    display: grid;
    grid-template-columns: 2fr 2fr 1.5fr 1fr;
    gap: 1rem;
  }

  .perf-header {
    background: var(--gray-900);
    border-bottom: 1px solid var(--gray-800);
  }

  .perf-header .perf-cell {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--gray-400);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .perf-row:not(.perf-header) {
    border-bottom: 1px solid var(--gray-900);
  }

  .perf-row:last-child {
    border-bottom: none;
  }

  .perf-cell {
    padding: 0.875rem 1rem;
    font-size: 0.875rem;
    color: var(--gray-300);
  }

  .autograd-steps {
    padding-left: 1.5rem;
    margin: 1rem 0;
  }

  .autograd-steps li {
    font-size: 0.9375rem;
    line-height: 1.7;
    color: var(--gray-400);
    margin-bottom: 0.5rem;
  }

  .ops-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1rem;
  }

  .ops-category {
    padding: 1rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .ops-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--gray-400);
    margin-bottom: 0.75rem;
  }

  .ops-list {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .ops-list code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.8125rem;
    color: var(--gray-300);
    background: var(--gray-900);
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
  }

  .roadmap-section {
    display: grid;
    gap: 1.5rem;
  }

  .roadmap-category h3 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
    margin-bottom: 0.75rem;
  }

  .roadmap-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    list-style: none;
    padding: 0;
  }

  .roadmap-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--gray-400);
    padding-left: 1.5rem;
    position: relative;
  }

  .roadmap-list li::before {
    content: '□';
    position: absolute;
    left: 0;
    color: var(--gray-600);
  }

  .structure-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
  }

  .structure-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--gray-400);
    padding-left: 1.5rem;
    position: relative;
  }

  .structure-list li::before {
    content: '→';
    position: absolute;
    left: 0;
    color: var(--gray-600);
  }

  .structure-list code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.875rem;
    color: var(--gray-300);
    background: var(--gray-950);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
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
    color: var(--gray-300);
    padding: 0.75rem 1rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.375rem;
    transition: all var(--duration-fast) var(--ease-out);
  }

  .resource-link:hover {
    color: var(--white);
    border-color: var(--gray-700);
    transform: translateX(4px);
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

    .philosophy-grid {
      grid-template-columns: 1fr;
    }

    .ops-grid {
      grid-template-columns: 1fr;
    }

    .perf-row {
      grid-template-columns: 1fr 1fr;
      gap: 0.5rem;
    }

    .perf-cell:nth-child(3),
    .perf-cell:nth-child(4) {
      grid-column: span 1;
    }
  }
</style>
