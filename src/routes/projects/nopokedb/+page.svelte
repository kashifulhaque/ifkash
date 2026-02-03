<svelte:head>
  <title>NoPokeDB — Lightweight Vector Database</title>
  <meta name="description" content="Lightweight vector DB with hnswlib + SQLite. Crash recovery, 2K+ PyPI downloads." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/projects">Projects</a>
      <span class="separator">/</span>
      <span>NoPokeDB</span>
    </div>
    <h1 class="page-title">NoPokeDB 🐹</h1>
    <p class="page-desc">A tiny, disk-backed vector database built on hnswlib and SQLite. Simple, lightweight, and crash-safe—perfect for prototypes, side projects, or when you don't want a heavyweight vector DB service.</p>
    <div class="project-links">
      <a href="https://github.com/kashifulhaque/nopokedb" target="_blank" rel="noopener noreferrer" class="project-link">
        Code
      </a>
      <a href="https://pypi.org/project/nopokedb/" target="_blank" rel="noopener noreferrer" class="project-link">
        PyPI
      </a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Overview</h2>
      <p>
        NoPokeDB is a minimalist vector database that combines the speed of hnswlib's approximate nearest neighbor search 
        with the reliability of SQLite for metadata storage. It's designed to be simple, durable, and fast—everything 
        you need for vector search without the complexity of enterprise solutions.
      </p>
      <p>
        With over 2,000 PyPI downloads, NoPokeDB has proven useful for developers who need a quick, no-fuss vector database 
        for embeddings, semantic search, or recommendation systems.
      </p>
    </div>

    <div class="section">
      <h2>Key Features</h2>
      <ul class="feature-list">
        <li><strong>Durable</strong>: HNSW index + SQLite metadata, fully persisted on disk</li>
        <li><strong>Crash Safety</strong>: Write-ahead operation log with automatic replay on restart</li>
        <li><strong>Fast</strong>: Batch inserts, bulk metadata fetch, auto-resize index for performance</li>
        <li><strong>Flexible Metrics</strong>: Supports cosine similarity, L2 distance, and inner product</li>
        <li><strong>CRUD Operations</strong>: add, add_many, get, delete, upsert—everything you need</li>
        <li><strong>Consistent Scoring</strong>: Higher scores = better matches across all metrics</li>
      </ul>
    </div>

    <div class="section">
      <h2>Installation</h2>
      <div class="code-block">
        <pre><code>pip install nopokedb</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Quick Start</h2>
      <div class="code-block">
        <pre><code>import numpy as np
from nopokedb import NoPokeDB

# Create (or load existing) DB with 128-d vectors
db = NoPokeDB(dim=128, max_elements=10_000, path="./vdb_data", space="cosine")

# Insert one vector
vec = np.random.rand(128).astype(np.float32)
vid = db.add(vec, metadata={'{'}\"name\": \"foo\"{'}'})

# Insert many at once (faster)
V = np.random.rand(5, 128).astype(np.float32)
metas = [{'{'}\"i\": i{'}'} for i in range(len(V))]
ids = db.add_many(V, metas)

# Query nearest neighbors
q = np.random.rand(128).astype(np.float32)
hits = db.query(q, k=3)
for h in hits:
    print(h["id"], h["score"], h["metadata"])

# Get / update / delete
print(db.get(vid))
db.upsert(vid, metadata={'{'}\"name\": \"bar\"{'}'})
db.delete(ids[0])

# Persist & close
db.save()
db.close()</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Example Query Result</h2>
      <div class="result-example">
        <pre><code>{'{'}
  'id': 0,
  'metadata': {'{'}\"name\": \"foo\"{'}'},
  'score': 0.991,
  'distance': 0.009
{'}'}</code></pre>
      </div>
    </div>

    <div class="section">
      <h2>Architecture</h2>
      <div class="arch-grid">
        <div class="arch-item">
          <div class="arch-title">HNSW Index</div>
          <div class="arch-desc">
            Uses hnswlib for fast approximate nearest neighbor search. Hierarchical Navigable Small World graphs 
            provide excellent performance with configurable precision-speed tradeoffs.
          </div>
        </div>
        <div class="arch-item">
          <div class="arch-title">SQLite Metadata</div>
          <div class="arch-desc">
            Stores associated metadata with ACID guarantees. Lightweight, embedded, and perfect for small to medium-sized 
            vector collections without external dependencies.
          </div>
        </div>
        <div class="arch-item">
          <div class="arch-title">Write-Ahead Log</div>
          <div class="arch-desc">
            Operations are logged before execution with fsync. If a crash occurs, the oplog replays automatically 
            on restart, ensuring no data loss.
          </div>
        </div>
        <div class="arch-item">
          <div class="arch-title">Auto-Resize</div>
          <div class="arch-desc">
            Automatically expands the HNSW index capacity as needed. Start small and grow organically without 
            manual intervention.
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>API Reference</h2>
      
      <div class="api-section">
        <h3>Initialization</h3>
        <div class="code-block">
          <pre><code>db = NoPokeDB(
    dim=128,                  # Vector dimensionality
    max_elements=10_000,      # Initial capacity (auto-resizes)
    path="./vdb_data",        # Storage directory
    space="cosine",           # Distance metric: cosine, l2, ip
    M=16,                     # HNSW graph connectivity
    ef_construction=200,      # Index build quality
    ef=50                     # Query quality
)</code></pre>
        </div>
      </div>

      <div class="api-section">
        <h3>Core Operations</h3>
        <div class="api-grid">
          <div class="api-item">
            <div class="api-method">add(vector, metadata)</div>
            <div class="api-desc">Insert a single vector with metadata. Returns the assigned ID.</div>
          </div>
          <div class="api-item">
            <div class="api-method">add_many(vectors, metadatas)</div>
            <div class="api-desc">Batch insert multiple vectors. Much faster than repeated add() calls.</div>
          </div>
          <div class="api-item">
            <div class="api-method">query(vector, k=10)</div>
            <div class="api-desc">Find k nearest neighbors. Returns list of results with id, score, distance, metadata.</div>
          </div>
          <div class="api-item">
            <div class="api-method">get(id)</div>
            <div class="api-desc">Retrieve metadata for a specific vector ID.</div>
          </div>
          <div class="api-item">
            <div class="api-method">upsert(id, metadata)</div>
            <div class="api-desc">Update metadata for an existing vector.</div>
          </div>
          <div class="api-item">
            <div class="api-method">delete(id)</div>
            <div class="api-desc">Remove a vector and its metadata from the database.</div>
          </div>
          <div class="api-item">
            <div class="api-method">save()</div>
            <div class="api-desc">Persist the HNSW index to disk. Called automatically on close().</div>
          </div>
          <div class="api-item">
            <div class="api-method">close()</div>
            <div class="api-desc">Save and close the database connection.</div>
          </div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Use Cases</h2>
      <ul class="usecase-list">
        <li><strong>Semantic Search</strong>: Find similar documents or passages using text embeddings</li>
        <li><strong>Recommendation Systems</strong>: Content-based recommendations with item embeddings</li>
        <li><strong>Image Similarity</strong>: Visual search using image embeddings from CNNs</li>
        <li><strong>RAG Applications</strong>: Vector store for retrieval-augmented generation pipelines</li>
        <li><strong>Prototyping</strong>: Quick vector search setup without Docker or cloud services</li>
        <li><strong>Side Projects</strong>: Lightweight alternative to Pinecone, Weaviate, or Milvus</li>
      </ul>
    </div>

    <div class="section">
      <h2>Distance Metrics</h2>
      <div class="metrics-grid">
        <div class="metric-item">
          <div class="metric-name">Cosine Similarity</div>
          <div class="metric-code">space="cosine"</div>
          <div class="metric-desc">Measures angle between vectors. Perfect for text embeddings and normalized features.</div>
        </div>
        <div class="metric-item">
          <div class="metric-name">L2 Distance</div>
          <div class="metric-code">space="l2"</div>
          <div class="metric-desc">Euclidean distance. Good for spatial coordinates and absolute magnitude differences.</div>
        </div>
        <div class="metric-item">
          <div class="metric-name">Inner Product</div>
          <div class="metric-code">space="ip"</div>
          <div class="metric-desc">Dot product similarity. Useful for pre-normalized embeddings and linear models.</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Why NoPokeDB?</h2>
      <div class="why-grid">
        <div class="why-item">
          <div class="why-icon">⚡</div>
          <div class="why-title">No Server Required</div>
          <div class="why-desc">Embedded database—no Docker containers, no API servers, just import and use.</div>
        </div>
        <div class="why-item">
          <div class="why-icon">💾</div>
          <div class="why-title">Persistent Storage</div>
          <div class="why-desc">Everything saved to disk with crash recovery. Restart your app without losing data.</div>
        </div>
        <div class="why-item">
          <div class="why-icon">🎯</div>
          <div class="why-title">Simple API</div>
          <div class="why-desc">Just a handful of methods. No complex configuration, no YAML files.</div>
        </div>
        <div class="why-item">
          <div class="why-icon">🚀</div>
          <div class="why-title">Fast Enough</div>
          <div class="why-desc">HNSW provides sub-linear search time. Great for thousands to millions of vectors.</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Project Stats</h2>
      <div class="stats-row">
        <div class="stat-item">
          <div class="stat-value">2,000+</div>
          <div class="stat-label">PyPI Downloads</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">~500</div>
          <div class="stat-label">Lines of Code</div>
        </div>
        <div class="stat-item">
          <div class="stat-value">Zero</div>
          <div class="stat-label">External Services</div>
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
  .usecase-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
  }

  .feature-list li,
  .usecase-list li {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--gray-400);
    padding-left: 1.5rem;
    position: relative;
  }

  .feature-list li::before,
  .usecase-list li::before {
    content: '→';
    position: absolute;
    left: 0;
    color: var(--gray-600);
  }

  .code-block,
  .result-example {
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
    overflow-x: auto;
  }

  .code-block pre,
  .result-example pre {
    margin: 0;
    padding: 1.25rem;
  }

  .code-block code,
  .result-example code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--gray-300);
  }

  .arch-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 1rem;
  }

  .arch-item {
    padding: 1.25rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .arch-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
    margin-bottom: 0.5rem;
  }

  .arch-desc {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--gray-400);
  }

  .api-section {
    margin-bottom: 1.5rem;
  }

  .api-section:last-child {
    margin-bottom: 0;
  }

  .api-grid {
    display: grid;
    gap: 0.75rem;
  }

  .api-item {
    padding: 0.875rem 1rem;
    background: var(--gray-950);
    border-left: 2px solid var(--gray-800);
  }

  .api-method {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--gray-200);
    margin-bottom: 0.375rem;
  }

  .api-desc {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--gray-400);
  }

  .metrics-grid {
    display: grid;
    gap: 1rem;
  }

  .metric-item {
    padding: 1rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .metric-name {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
    margin-bottom: 0.375rem;
  }

  .metric-code {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 0.8125rem;
    color: var(--gray-500);
    margin-bottom: 0.5rem;
  }

  .metric-desc {
    font-size: 0.875rem;
    line-height: 1.6;
    color: var(--gray-400);
  }

  .why-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .why-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1.25rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .why-icon {
    font-size: 2rem;
    line-height: 1;
  }

  .why-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
  }

  .why-desc {
    font-size: 0.875rem;
    line-height: 1.5;
    color: var(--gray-400);
  }

  .stats-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1.5rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: 0.5rem;
  }

  .stat-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--white);
  }

  .stat-label {
    font-size: 0.875rem;
    color: var(--gray-500);
    text-align: center;
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

    .arch-grid,
    .why-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
