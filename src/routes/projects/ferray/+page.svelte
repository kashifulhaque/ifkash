<svelte:head>
  <title>ferray — NumPy-like Array in Rust</title>
  <meta name="description" content="NumPy-like ndarray in Rust with Python bindings. Learning project." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/projects">Projects</a>
      <span class="separator">/</span>
      <span>ferray</span>
    </div>
    <h1 class="page-title">ferray</h1>
    <p class="page-desc">A tiny NumPy-like library for Python, written in Rust. Built to learn the inner workings of NumPy and explore Rust-Python interoperability with PyO3 and maturin.</p>
    <div class="project-links">
      <a href="https://github.com/kashifulhaque/ferray" target="_blank" rel="noopener noreferrer" class="project-link">
        Code
      </a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Overview</h2>
      <p>
        ferray is an educational project that implements a NumPy-like multidimensional array library in Rust with Python bindings. 
        It demonstrates how to build high-performance numerical computing libraries using Rust's safety and speed while maintaining 
        Python's ease of use.
      </p>
      <p>
        The project uses OpenBLAS for optimized linear algebra operations and PyO3 for seamless Python-Rust interoperability.
      </p>
    </div>

    <div class="section">
      <h2>Key Features</h2>
      <ul class="feature-list">
        <li><strong>NumPy-like API</strong>: Familiar interface for Python developers with array operations</li>
        <li><strong>Rust Performance</strong>: Fast operations with zero-cost abstractions and memory safety</li>
        <li><strong>OpenBLAS Integration</strong>: Leverages optimized BLAS routines for linear algebra</li>
        <li><strong>Python Bindings</strong>: Seamless integration via PyO3 and maturin</li>
        <li><strong>Broadcasting Support</strong>: NumPy-style broadcasting for element-wise operations</li>
        <li><strong>NumPy Interoperability</strong>: Convert between NumPy arrays and ferray</li>
      </ul>
    </div>

    <div class="section">
      <h2>Supported Operations</h2>
      <div class="ops-grid">
        <div class="ops-category">
          <div class="ops-title">Array Creation</div>
          <ul class="ops-list">
            <li><code>NdArray([2, 3])</code> — Create uninitialized array</li>
            <li><code>zeros([3, 4])</code> — Create array filled with zeros</li>
            <li><code>ones([2, 3])</code> — Create array filled with ones</li>
            <li><code>from_list([[1, 2], [3, 4]])</code> — From Python list</li>
            <li><code>from_numpy(arr)</code> — From NumPy array</li>
          </ul>
        </div>
        <div class="ops-category">
          <div class="ops-title">Indexing & Reshaping</div>
          <ul class="ops-list">
            <li><code>a[0, 2]</code> — Element access via indexing</li>
            <li><code>a.get([0, 1])</code> — Get element at position</li>
            <li><code>a.set([0, 1], 42.0)</code> — Set element value</li>
            <li><code>a.reshape([3, 2])</code> — Change shape</li>
            <li><code>a.transpose()</code> — Matrix transpose</li>
          </ul>
        </div>
        <div class="ops-category">
          <div class="ops-title">Element-wise Operations</div>
          <ul class="ops-list">
            <li><code>a + b</code> — Addition with broadcasting</li>
            <li><code>a - b</code> — Subtraction</li>
            <li><code>a * b</code> — Multiplication</li>
            <li><code>a / b</code> — Division</li>
            <li><code>a + 2.0</code> — Scalar operations</li>
          </ul>
        </div>
        <div class="ops-category">
          <div class="ops-title">Properties & Conversion</div>
          <ul class="ops-list">
            <li><code>a.ndim()</code> — Number of dimensions</li>
            <li><code>a.shape()</code> — Shape tuple</li>
            <li><code>a.to_list()</code> — Convert to Python list</li>
            <li><code>a.to_numpy()</code> — Convert to NumPy array</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Installation</h2>
      
      <h3 class="subsection-title">Prerequisites</h3>
      <div class="prereq-grid">
        <div class="prereq-item">
          <div class="prereq-name">OpenBLAS</div>
          <div class="code-block">
            <pre><code># Ubuntu/Debian
sudo apt install libopenblas-dev

# macOS (Homebrew)
brew install openblas

# Arch Linux
sudo pacman -S openblas</code></pre>
          </div>
        </div>
        <div class="prereq-item">
          <div class="prereq-name">Rust Toolchain</div>
          <div class="code-block">
            <pre><code>curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh</code></pre>
          </div>
        </div>
        <div class="prereq-item">
          <div class="prereq-name">Maturin</div>
          <div class="code-block">
            <pre><code># Using uv (recommended)
uv tool install maturin

# Or using pip
pip install maturin</code></pre>
          </div>
        </div>
      </div>

      <h3 class="subsection-title">Building the Package</h3>
      <div class="code-block">
        <pre><code># Clone the repository
git clone https://github.com/kashifulhaque/ferray.git
cd ferray

# Build and install in development mode
maturin develop --release

# Or using uv
uv run maturin develop --release</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Usage Examples</h2>

      <h3 class="subsection-title">Basic Array Creation</h3>
      <div class="code-block">
        <pre><code>import ferray as tnp

# Create arrays
a = tnp.NdArray.zeros([3, 4])
b = tnp.NdArray.ones([2, 3])
c = tnp.NdArray.from_list([[1.0, 2.0], [3.0, 4.0]])

# Check properties
print(a.shape())  # [3, 4]
print(a.ndim())   # 2</code></pre>
      </div>

      <h3 class="subsection-title">Indexing and Assignment</h3>
      <div class="code-block">
        <pre><code>a = tnp.NdArray.zeros([3, 4])

# Get and set values
print(a[0, 2])  # 0.0
a[0, 2] = 99.0
print(a[0, 2])  # 99.0

# Using methods
a.set([1, 3], 42.0)
print(a.get([1, 3]))  # 42.0</code></pre>
      </div>

      <h3 class="subsection-title">Element-wise Operations</h3>
      <div class="code-block">
        <pre><code>a = tnp.NdArray.ones([2, 3])
b = tnp.NdArray.ones([1, 3]) * 2.0

print(a + b)  # Broadcasting addition
print(a - b)  # Subtraction
print(a * b)  # Multiplication
print(a / b)  # Division

# Scalar operations
print(a + 2.0)
print(a * 5.0)</code></pre>
      </div>

      <h3 class="subsection-title">Transpose and Reshape</h3>
      <div class="code-block">
        <pre><code>a = tnp.NdArray.from_list([
    [1.0, 2.0, 3.0],
    [4.0, 5.0, 6.0]
])

print(a.shape())          # [2, 3]
print(a.transpose())      # Transposed array

a.reshape([3, 2])
print(a.shape())          # [3, 2]</code></pre>
      </div>

      <h3 class="subsection-title">NumPy Interoperability</h3>
      <div class="code-block">
        <pre><code>import numpy as np
import ferray as tnp

# From NumPy to ferray
a_np = np.ones((2, 3), dtype=np.float32)
a_rust = tnp.NdArray.from_numpy(a_np)

# Back to NumPy
a_back = a_rust.to_numpy()
print(np.allclose(a_np, a_back))  # True</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Using mise (Optional)</h2>
      <p>If you have mise installed, you can use the provided task commands:</p>
      <div class="mise-grid">
        <div class="mise-item">
          <div class="mise-cmd">mise run develop</div>
          <div class="mise-desc">Build and install in development mode</div>
        </div>
        <div class="mise-item">
          <div class="mise-cmd">mise run test</div>
          <div class="mise-desc">Run test suite</div>
        </div>
        <div class="mise-item">
          <div class="mise-cmd">mise run bench</div>
          <div class="mise-desc">Run performance benchmarks</div>
        </div>
        <div class="mise-item">
          <div class="mise-cmd">mise run check</div>
          <div class="mise-desc">Check code quality</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Project Structure</h2>
      <ul class="structure-list">
        <li><code>src/lib.rs</code>: PyO3 module definition and exports</li>
        <li><code>src/array.rs</code>: Core NdArray implementation</li>
        <li><code>src/operations.rs</code>: Element-wise operations and broadcasting</li>
        <li><code>src/conversions.rs</code>: NumPy interoperability</li>
        <li><code>src/utils.rs</code>: Helper functions and utilities</li>
        <li><code>build.rs</code>: Build script for linking OpenBLAS</li>
        <li><code>test.py</code>: Basic functionality tests</li>
        <li><code>bench_matmul.py</code>: Matrix multiplication benchmarks</li>
      </ul>
    </div>

    <div class="section">
      <h2>Tech Stack</h2>
      <div class="tech-row">
        <div class="tech-badge">
          <span class="tech-label">Language</span>
          <span class="tech-value">Rust</span>
        </div>
        <div class="tech-badge">
          <span class="tech-label">Python Bindings</span>
          <span class="tech-value">PyO3</span>
        </div>
        <div class="tech-badge">
          <span class="tech-label">Build Tool</span>
          <span class="tech-value">maturin</span>
        </div>
        <div class="tech-badge">
          <span class="tech-label">Linear Algebra</span>
          <span class="tech-value">OpenBLAS</span>
        </div>
        <div class="tech-badge">
          <span class="tech-label">Target</span>
          <span class="tech-value">Python 3.11+</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Learning Goals</h2>
      <ul class="goals-list">
        <li>Understanding NumPy's internal architecture and memory layout</li>
        <li>Exploring Rust-Python interoperability with PyO3</li>
        <li>Implementing broadcasting semantics for array operations</li>
        <li>Integrating native BLAS libraries for performance</li>
        <li>Building Python extensions with maturin</li>
        <li>Managing memory safety across language boundaries</li>
      </ul>
    </div>

    <div class="section">
      <h2>Status</h2>
      <div class="status-banner">
        <div class="status-icon">🚧</div>
        <div class="status-content">
          <div class="status-title">Work in Progress</div>
          <div class="status-desc">
            This project is under active development. Core functionality is working, but many NumPy features 
            are not yet implemented. Contributions and feedback are welcome!
          </div>
        </div>
      </div>
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

  .feature-list,
  .goals-list,
  .structure-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
  }

  .feature-list li,
  .goals-list li,
  .structure-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--gray-400);
    padding-left: 1.5rem;
    position: relative;
  }

  .feature-list li::before,
  .goals-list li::before,
  .structure-list li::before {
    content: '→';
    position: absolute;
    left: 0;
    color: var(--gray-600);
  }

  .structure-list code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.875rem;
    color: var(--gray-300);
    background: var(--gray-950);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .ops-grid {
    display: grid;
    gap: 1rem;
  }

  .ops-category {
    padding: 1rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .ops-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
    margin-bottom: 0.75rem;
  }

  .ops-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    list-style: none;
    padding: 0;
  }

  .ops-list li {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--gray-400);
  }

  .ops-list code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.875rem;
    color: var(--gray-300);
    background: var(--gray-900);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .prereq-grid {
    display: grid;
    gap: 1rem;
  }

  .prereq-item {
    padding: 1rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .prereq-name {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--gray-400);
    margin-bottom: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
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

  .mise-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 0.75rem;
  }

  .mise-item {
    padding: 0.875rem 1rem;
    background: var(--gray-950);
    border-left: 2px solid var(--gray-800);
  }

  .mise-cmd {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--gray-200);
    margin-bottom: 0.375rem;
  }

  .mise-desc {
    font-size: 0.8125rem;
    line-height: 1.5;
    color: var(--gray-500);
  }

  .tech-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .tech-badge {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem 1rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.375rem;
  }

  .tech-label {
    font-size: 0.75rem;
    color: var(--gray-500);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .tech-value {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--white);
  }

  .status-banner {
    display: flex;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .status-icon {
    font-size: 2rem;
    line-height: 1;
  }

  .status-content {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .status-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
  }

  .status-desc {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--gray-400);
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

    .mise-grid {
      grid-template-columns: 1fr;
    }

    .tech-row {
      flex-direction: column;
    }
  }
</style>
