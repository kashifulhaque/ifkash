<svelte:head>
  <title>ferray — NumPy-like Array in Rust</title>
  <meta name="description" content="NumPy-like ndarray in Rust with Python bindings. Learning project." />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/projects">Projects</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">ferray</span>
    </nav>
    <h1 class="page-title">ferray</h1>
    <p class="lead">A tiny NumPy-like library for Python, written in Rust. Built to learn the inner workings of NumPy and explore Rust-Python interoperability with PyO3 and maturin.</p>
    <div class="btn-row">
      <a href="https://github.com/kashifulhaque/ferray" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Code <span class="ext" aria-hidden="true">↗</span>
      </a>
    </div>
  </header>

  <div class="detail-body">
    <aside class="facts" aria-label="Project facts">
      <dl>
        <div class="fact">
          <dt>Status</dt>
          <dd><span class="status"><span class="led" aria-hidden="true"></span>Work in progress</span></dd>
        </div>
        <div class="fact">
          <dt>Stack</dt>
          <dd><span>Rust</span><span>PyO3</span><span>maturin</span><span>OpenBLAS</span></dd>
        </div>
        <div class="fact">
          <dt>Target</dt>
          <dd>Python 3.11+</dd>
        </div>
        <div class="fact">
          <dt>Links</dt>
          <dd>
            <a href="https://github.com/kashifulhaque/ferray" target="_blank" rel="noopener noreferrer">GitHub <span aria-hidden="true">↗</span></a>
          </dd>
        </div>
      </dl>
    </aside>

    <div class="copy">
      <section class="section">
        <h2><span class="num">01</span>Overview</h2>
        <p>
          ferray is an educational project that implements a NumPy-like multidimensional array library in Rust with Python bindings.
          It demonstrates how to build high-performance numerical computing libraries using Rust's safety and speed while maintaining
          Python's ease of use.
        </p>
        <p>
          The project uses OpenBLAS for optimized linear algebra operations and PyO3 for seamless Python-Rust interoperability.
        </p>
      </section>

      <section class="section">
        <h2><span class="num">02</span>Key Features</h2>
        <ul class="defs">
          <li><span class="term">NumPy-like API</span><span class="desc">Familiar interface for Python developers with array operations</span></li>
          <li><span class="term">Rust Performance</span><span class="desc">Fast operations with zero-cost abstractions and memory safety</span></li>
          <li><span class="term">OpenBLAS Integration</span><span class="desc">Leverages optimized BLAS routines for linear algebra</span></li>
          <li><span class="term">Python Bindings</span><span class="desc">Seamless integration via PyO3 and maturin</span></li>
          <li><span class="term">Broadcasting Support</span><span class="desc">NumPy-style broadcasting for element-wise operations</span></li>
          <li><span class="term">NumPy Interoperability</span><span class="desc">Convert between NumPy arrays and ferray</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">03</span>Supported Operations</h2>

        <h3>Array Creation</h3>
        <ul class="defs">
          <li><span class="term"><code>NdArray([2, 3])</code></span><span class="desc">Create uninitialized array</span></li>
          <li><span class="term"><code>zeros([3, 4])</code></span><span class="desc">Create array filled with zeros</span></li>
          <li><span class="term"><code>ones([2, 3])</code></span><span class="desc">Create array filled with ones</span></li>
          <li><span class="term"><code>from_list([[1, 2], [3, 4]])</code></span><span class="desc">From Python list</span></li>
          <li><span class="term"><code>from_numpy(arr)</code></span><span class="desc">From NumPy array</span></li>
        </ul>

        <h3>Indexing & Reshaping</h3>
        <ul class="defs">
          <li><span class="term"><code>a[0, 2]</code></span><span class="desc">Element access via indexing</span></li>
          <li><span class="term"><code>a.get([0, 1])</code></span><span class="desc">Get element at position</span></li>
          <li><span class="term"><code>a.set([0, 1], 42.0)</code></span><span class="desc">Set element value</span></li>
          <li><span class="term"><code>a.reshape([3, 2])</code></span><span class="desc">Change shape</span></li>
          <li><span class="term"><code>a.transpose()</code></span><span class="desc">Matrix transpose</span></li>
        </ul>

        <h3>Element-wise Operations</h3>
        <ul class="defs">
          <li><span class="term"><code>a + b</code></span><span class="desc">Addition with broadcasting</span></li>
          <li><span class="term"><code>a - b</code></span><span class="desc">Subtraction</span></li>
          <li><span class="term"><code>a * b</code></span><span class="desc">Multiplication</span></li>
          <li><span class="term"><code>a / b</code></span><span class="desc">Division</span></li>
          <li><span class="term"><code>a + 2.0</code></span><span class="desc">Scalar operations</span></li>
        </ul>

        <h3>Properties & Conversion</h3>
        <ul class="defs">
          <li><span class="term"><code>a.ndim()</code></span><span class="desc">Number of dimensions</span></li>
          <li><span class="term"><code>a.shape()</code></span><span class="desc">Shape tuple</span></li>
          <li><span class="term"><code>a.to_list()</code></span><span class="desc">Convert to Python list</span></li>
          <li><span class="term"><code>a.to_numpy()</code></span><span class="desc">Convert to NumPy array</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">04</span>Installation</h2>

        <h3>Prerequisites</h3>
        <div class="prereqs">
          <div class="prereq">
            <p class="prereq-name">OpenBLAS</p>
            <pre><code># Ubuntu/Debian
sudo apt install libopenblas-dev

# macOS (Homebrew)
brew install openblas

# Arch Linux
sudo pacman -S openblas</code></pre>
          </div>
          <div class="prereq">
            <p class="prereq-name">Rust Toolchain</p>
            <pre><code>curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh</code></pre>
          </div>
          <div class="prereq">
            <p class="prereq-name">Maturin</p>
            <pre><code># Using uv (recommended)
uv tool install maturin

# Or using pip
pip install maturin</code></pre>
          </div>
        </div>

        <h3>Building the Package</h3>
        <pre><code># Clone the repository
git clone https://github.com/kashifulhaque/ferray.git
cd ferray

# Build and install in development mode
maturin develop --release

# Or using uv
uv run maturin develop --release</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">05</span>Usage Examples</h2>

        <h3>Basic Array Creation</h3>
        <pre><code>import ferray as tnp

# Create arrays
a = tnp.NdArray.zeros([3, 4])
b = tnp.NdArray.ones([2, 3])
c = tnp.NdArray.from_list([[1.0, 2.0], [3.0, 4.0]])

# Check properties
print(a.shape())  # [3, 4]
print(a.ndim())   # 2</code></pre>

        <h3>Indexing and Assignment</h3>
        <pre><code>a = tnp.NdArray.zeros([3, 4])

# Get and set values
print(a[0, 2])  # 0.0
a[0, 2] = 99.0
print(a[0, 2])  # 99.0

# Using methods
a.set([1, 3], 42.0)
print(a.get([1, 3]))  # 42.0</code></pre>

        <h3>Element-wise Operations</h3>
        <pre><code>a = tnp.NdArray.ones([2, 3])
b = tnp.NdArray.ones([1, 3]) * 2.0

print(a + b)  # Broadcasting addition
print(a - b)  # Subtraction
print(a * b)  # Multiplication
print(a / b)  # Division

# Scalar operations
print(a + 2.0)
print(a * 5.0)</code></pre>

        <h3>Transpose and Reshape</h3>
        <pre><code>a = tnp.NdArray.from_list([
    [1.0, 2.0, 3.0],
    [4.0, 5.0, 6.0]
])

print(a.shape())          # [2, 3]
print(a.transpose())      # Transposed array

a.reshape([3, 2])
print(a.shape())          # [3, 2]</code></pre>

        <h3>NumPy Interoperability</h3>
        <pre><code>import numpy as np
import ferray as tnp

# From NumPy to ferray
a_np = np.ones((2, 3), dtype=np.float32)
a_rust = tnp.NdArray.from_numpy(a_np)

# Back to NumPy
a_back = a_rust.to_numpy()
print(np.allclose(a_np, a_back))  # True</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">06</span>Using mise (Optional)</h2>
        <p>If you have mise installed, you can use the provided task commands:</p>
        <ul class="defs">
          <li><span class="term mono">mise run develop</span><span class="desc">Build and install in development mode</span></li>
          <li><span class="term mono">mise run test</span><span class="desc">Run test suite</span></li>
          <li><span class="term mono">mise run bench</span><span class="desc">Run performance benchmarks</span></li>
          <li><span class="term mono">mise run check</span><span class="desc">Check code quality</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">07</span>Project Structure</h2>
        <ul class="defs">
          <li><span class="term"><code>src/lib.rs</code></span><span class="desc">PyO3 module definition and exports</span></li>
          <li><span class="term"><code>src/array.rs</code></span><span class="desc">Core NdArray implementation</span></li>
          <li><span class="term"><code>src/operations.rs</code></span><span class="desc">Element-wise operations and broadcasting</span></li>
          <li><span class="term"><code>src/conversions.rs</code></span><span class="desc">NumPy interoperability</span></li>
          <li><span class="term"><code>src/utils.rs</code></span><span class="desc">Helper functions and utilities</span></li>
          <li><span class="term"><code>build.rs</code></span><span class="desc">Build script for linking OpenBLAS</span></li>
          <li><span class="term"><code>test.py</code></span><span class="desc">Basic functionality tests</span></li>
          <li><span class="term"><code>bench_matmul.py</code></span><span class="desc">Matrix multiplication benchmarks</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">08</span>Tech Stack</h2>
        <dl class="kv">
          <div><dt>Language</dt><dd>Rust</dd></div>
          <div><dt>Python Bindings</dt><dd>PyO3</dd></div>
          <div><dt>Build Tool</dt><dd>maturin</dd></div>
          <div><dt>Linear Algebra</dt><dd>OpenBLAS</dd></div>
          <div><dt>Target</dt><dd>Python 3.11+</dd></div>
        </dl>
      </section>

      <section class="section">
        <h2><span class="num">09</span>Learning Goals</h2>
        <ul class="ruled">
          <li>Understanding NumPy's internal architecture and memory layout</li>
          <li>Exploring Rust-Python interoperability with PyO3</li>
          <li>Implementing broadcasting semantics for array operations</li>
          <li>Integrating native BLAS libraries for performance</li>
          <li>Building Python extensions with maturin</li>
          <li>Managing memory safety across language boundaries</li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">10</span>Status</h2>
        <div class="callout">
          <p class="callout-title"><span class="led" aria-hidden="true"></span>Work in Progress</p>
          <p>
            This project is under active development. Core functionality is working, but many NumPy features
            are not yet implemented. Contributions and feedback are welcome!
          </p>
        </div>
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

  .status {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
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

  .term.mono {
    padding: 0;
    background: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
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

  .prereqs {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-5);
  }

  .prereq-name {
    margin: 0 0 var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .prereq pre {
    margin: 0;
  }

  .callout {
    padding: var(--s-4) var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }

  .callout p {
    margin: 0;
  }

  .callout .callout-title {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-bottom: var(--s-2);
    color: var(--ink);
    font-weight: 500;
  }
</style>
