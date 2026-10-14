<svelte:head>
  <title>NoPokeDB — Lightweight Vector Database</title>
  <meta name="description" content="Lightweight vector DB with hnswlib + SQLite. Crash recovery, 2K+ PyPI downloads." />
</svelte:head>

<article class="detail">
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/projects">Projects</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">NoPokeDB</span>
    </nav>
    <h1 class="page-title">NoPokeDB <span class="mark">🐹</span></h1>
    <p class="lead">A tiny, disk-backed vector database built on hnswlib and SQLite. Simple, lightweight, and crash-safe—perfect for prototypes, side projects, or when you don't want a heavyweight vector DB service.</p>
    <div class="btn-row">
      <a href="https://github.com/kashifulhaque/nopokedb" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        Code <span class="ext" aria-hidden="true">↗</span>
      </a>
      <a href="https://pypi.org/project/nopokedb/" target="_blank" rel="noopener noreferrer" class="btn btn-sm">
        PyPI <span class="ext" aria-hidden="true">↗</span>
      </a>
    </div>
  </header>

  <div class="detail-body">
    <aside class="facts" aria-label="Project facts">
      <dl>
        <div class="fact">
          <dt>Stack</dt>
          <dd><span>Python</span><span>hnswlib</span><span>SQLite</span></dd>
        </div>
        <div class="fact">
          <dt>PyPI downloads</dt>
          <dd>2,000+</dd>
        </div>
        <div class="fact">
          <dt>Size</dt>
          <dd>~500 lines</dd>
        </div>
        <div class="fact">
          <dt>Metrics</dt>
          <dd>cosine · l2 · ip</dd>
        </div>
        <div class="fact">
          <dt>Install</dt>
          <dd>pip install nopokedb</dd>
        </div>
        <div class="fact">
          <dt>Links</dt>
          <dd>
            <a href="https://github.com/kashifulhaque/nopokedb" target="_blank" rel="noopener noreferrer">GitHub <span aria-hidden="true">↗</span></a>
            <a href="https://pypi.org/project/nopokedb/" target="_blank" rel="noopener noreferrer">PyPI <span aria-hidden="true">↗</span></a>
          </dd>
        </div>
      </dl>
    </aside>

    <div class="copy">
      <section class="section">
        <h2><span class="num">01</span>Overview</h2>
        <p>
          NoPokeDB is a minimalist vector database that combines the speed of hnswlib's approximate nearest neighbor search
          with the reliability of SQLite for metadata storage. It's designed to be simple, durable, and fast—everything
          you need for vector search without the complexity of enterprise solutions.
        </p>
        <p>
          With over 2,000 PyPI downloads, NoPokeDB has proven useful for developers who need a quick, no-fuss vector database
          for embeddings, semantic search, or recommendation systems.
        </p>
      </section>

      <section class="section">
        <h2><span class="num">02</span>Key Features</h2>
        <ul class="defs">
          <li><span class="term">Durable</span><span class="desc">HNSW index + SQLite metadata, fully persisted on disk</span></li>
          <li><span class="term">Crash Safety</span><span class="desc">Write-ahead operation log with automatic replay on restart</span></li>
          <li><span class="term">Fast</span><span class="desc">Batch inserts, bulk metadata fetch, auto-resize index for performance</span></li>
          <li><span class="term">Flexible Metrics</span><span class="desc">Supports cosine similarity, L2 distance, and inner product</span></li>
          <li><span class="term">CRUD Operations</span><span class="desc">add, add_many, get, delete, upsert—everything you need</span></li>
          <li><span class="term">Consistent Scoring</span><span class="desc">Higher scores = better matches across all metrics</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">03</span>Installation</h2>
        <pre><code>pip install nopokedb</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">04</span>Quick Start</h2>
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
      </section>

      <section class="section">
        <h2><span class="num">05</span>Example Query Result</h2>
        <pre><code>{'{'}
  'id': 0,
  'metadata': {'{'}\"name\": \"foo\"{'}'},
  'score': 0.991,
  'distance': 0.009
{'}'}</code></pre>
      </section>

      <section class="section">
        <h2><span class="num">06</span>Architecture</h2>
        <div class="tiles">
          <div class="tile">
            <h3>HNSW Index</h3>
            <p>
              Uses hnswlib for fast approximate nearest neighbor search. Hierarchical Navigable Small World graphs
              provide excellent performance with configurable precision-speed tradeoffs.
            </p>
          </div>
          <div class="tile">
            <h3>SQLite Metadata</h3>
            <p>
              Stores associated metadata with ACID guarantees. Lightweight, embedded, and perfect for small to medium-sized
              vector collections without external dependencies.
            </p>
          </div>
          <div class="tile">
            <h3>Write-Ahead Log</h3>
            <p>
              Operations are logged before execution with fsync. If a crash occurs, the oplog replays automatically
              on restart, ensuring no data loss.
            </p>
          </div>
          <div class="tile">
            <h3>Auto-Resize</h3>
            <p>
              Automatically expands the HNSW index capacity as needed. Start small and grow organically without
              manual intervention.
            </p>
          </div>
        </div>
      </section>

      <section class="section">
        <h2><span class="num">07</span>API Reference</h2>

        <h3>Initialization</h3>
        <pre><code>db = NoPokeDB(
    dim=128,                  # Vector dimensionality
    max_elements=10_000,      # Initial capacity (auto-resizes)
    path="./vdb_data",        # Storage directory
    space="cosine",           # Distance metric: cosine, l2, ip
    M=16,                     # HNSW graph connectivity
    ef_construction=200,      # Index build quality
    ef=50                     # Query quality
)</code></pre>

        <h3>Core Operations</h3>
        <ul class="defs">
          <li><span class="term mono">add(vector, metadata)</span><span class="desc">Insert a single vector with metadata. Returns the assigned ID.</span></li>
          <li><span class="term mono">add_many(vectors, metadatas)</span><span class="desc">Batch insert multiple vectors. Much faster than repeated add() calls.</span></li>
          <li><span class="term mono">query(vector, k=10)</span><span class="desc">Find k nearest neighbors. Returns list of results with id, score, distance, metadata.</span></li>
          <li><span class="term mono">get(id)</span><span class="desc">Retrieve metadata for a specific vector ID.</span></li>
          <li><span class="term mono">upsert(id, metadata)</span><span class="desc">Update metadata for an existing vector.</span></li>
          <li><span class="term mono">delete(id)</span><span class="desc">Remove a vector and its metadata from the database.</span></li>
          <li><span class="term mono">save()</span><span class="desc">Persist the HNSW index to disk. Called automatically on close().</span></li>
          <li><span class="term mono">close()</span><span class="desc">Save and close the database connection.</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">08</span>Use Cases</h2>
        <ul class="defs">
          <li><span class="term">Semantic Search</span><span class="desc">Find similar documents or passages using text embeddings</span></li>
          <li><span class="term">Recommendation Systems</span><span class="desc">Content-based recommendations with item embeddings</span></li>
          <li><span class="term">Image Similarity</span><span class="desc">Visual search using image embeddings from CNNs</span></li>
          <li><span class="term">RAG Applications</span><span class="desc">Vector store for retrieval-augmented generation pipelines</span></li>
          <li><span class="term">Prototyping</span><span class="desc">Quick vector search setup without Docker or cloud services</span></li>
          <li><span class="term">Side Projects</span><span class="desc">Lightweight alternative to Pinecone, Weaviate, or Milvus</span></li>
        </ul>
      </section>

      <section class="section">
        <h2><span class="num">09</span>Distance Metrics</h2>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th scope="col">Metric</th>
                <th scope="col">Parameter</th>
                <th scope="col">Use</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td class="strong">Cosine Similarity</td>
                <td class="mono">space="cosine"</td>
                <td>Measures angle between vectors. Perfect for text embeddings and normalized features.</td>
              </tr>
              <tr>
                <td class="strong">L2 Distance</td>
                <td class="mono">space="l2"</td>
                <td>Euclidean distance. Good for spatial coordinates and absolute magnitude differences.</td>
              </tr>
              <tr>
                <td class="strong">Inner Product</td>
                <td class="mono">space="ip"</td>
                <td>Dot product similarity. Useful for pre-normalized embeddings and linear models.</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section class="section">
        <h2><span class="num">10</span>Why NoPokeDB?</h2>
        <div class="tiles">
          <div class="tile">
            <h3>No Server Required</h3>
            <p>Embedded database—no Docker containers, no API servers, just import and use.</p>
          </div>
          <div class="tile">
            <h3>Persistent Storage</h3>
            <p>Everything saved to disk with crash recovery. Restart your app without losing data.</p>
          </div>
          <div class="tile">
            <h3>Simple API</h3>
            <p>Just a handful of methods. No complex configuration, no YAML files.</p>
          </div>
          <div class="tile">
            <h3>Fast Enough</h3>
            <p>HNSW provides sub-linear search time. Great for thousands to millions of vectors.</p>
          </div>
        </div>
      </section>

      <section class="section">
        <h2><span class="num">11</span>Project Stats</h2>
        <div class="stats">
          <div class="stat">
            <span class="stat-label">PyPI Downloads</span>
            <span class="stat-value">2,000+</span>
          </div>
          <div class="stat">
            <span class="stat-label">Lines of Code</span>
            <span class="stat-value">~500</span>
          </div>
          <div class="stat">
            <span class="stat-label">External Services</span>
            <span class="stat-value">Zero</span>
          </div>
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

  .term.mono {
    padding: 0;
    background: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
  }

  /* Two-up notes, each topped by a hairline */
  .tiles {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-5) var(--s-6);
  }

  .tile {
    padding-top: var(--s-3);
    border-top: 1px solid var(--line);
  }

  .tile h3 {
    margin: 0 0 var(--s-1);
    font-size: var(--text-base);
    letter-spacing: -0.005em;
  }

  .tile p {
    margin: 0;
    line-height: 1.55;
  }

  @media (min-width: 600px) {
    .tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
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
</style>
