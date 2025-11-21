#set text(font: "Crimson Pro")
#let cardo = text.with(font: "Cardo")

#show link: underline
#set page(margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

#let chiline() = {v(-3pt); line(length: 100%); v(-5pt)}

= #cardo[Kashiful Haque]
+918240868544 • #link("mailto:haque.kashiful7@gmail.com")[haque.kashiful7\@gmail.com] • #link("https://github.com/kashifulhaque")[github] • #link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • #link("https://ifkash.dev")[ifkash.dev]

Systems-focused ML engineer with 3.5 YOE building numerical computing components, inference-optimized pipelines and agentic coding environments. I work at the intersection of ML systems, low-level frameworks and RL for code, designing execution sandboxes, structured task environments and deep-dive into unfamiliar codebases to build testable evals. Previously fine-tuned and deployed models and optimized inference systems org-wide. \

= #cardo[work experience]

*#cardo[Wand AI (via Nityo)]* #h(1fr) _Palo Alto (Remote)_ \
_Backend AI/ML Engineer_ #h(1fr) 11/2025 -- Present \
- Building internal ML systems and execution-layer infra; focusing on agent tooling and structured task pipelines. \

*#cardo[American Express (via IntraEdge)]* #h(1fr) _Bangalore_ \
_Engineer III_ #h(1fr) 02/2025 -- 11/2025 \
- Architected a distributed low-latency streaming pipeline with backpressure control and token-stream handling; deployed on RedHat OpenShift using Helm/Jenkins.
- Rebuilt search infra for 200k+ Confluence documents using hybrid semantic + trigram retrieval; improved p95 latency from 30s → $<$2s.
- Reverse-engineered internal systems to build testable interfaces and mocks for multi-step automation pipelines.

*#cardo[Fiery]* #h(1fr) _Bangalore_ \
_Associate Software Engineer_ #h(1fr) 01/2023 — 02/2025 \
- Fine-tuned Mistral-7B with QLoRA; instrumented memory usage, evaluated kernel bottlenecks, and integrated into a scalable RAG pipeline.
- Delivered high-throughput inference using vLLM with dynamic batching and quantized kernels, hitting sub-second p95 TTFT on T4 clusters.
- Built Fiery Scribe, an NER-driven automation engine using a fine-tuned ModernBERT model; designed structured evaluation pipelines and test harnesses for model outputs.
- Developed AskDB, an LLM→SQL agent system with programmatic query analysis, partial-plan evaluation, and structured error recovery.

*#cardo[Corteva Agriscience]* #h(1fr) _Hyderabad_ \
_Internship_ #h(1fr) 07/2022 -- 12/2022 \

= #cardo[projects]
*#cardo[smoltorch: minimal autograd engine]* • #link("https://github.com/kashifulhaque/smoltorch")[github] • #link("https://pypi.org/project/smoltorch/")[pypi] • #link("https://blog.ifkash.dev/smoltorch")[blog]
- Reverse-mode autograd engine with tape-based graphs and topological scheduling.
- NumPy-backed tensor ops, broadcasting, and a minimal training loop inspired by PyTorch internals.

*#cardo[tinyndarray: mini numpy in rust]* • #link("https://github.com/kashifulhaque/tinyndarray")[github]
- Stride-aware ndarray implementation in Rust with slicing + broadcasting, mirroring ML framework tensor layouts.
- Python bindings via PyO3 enabling fast numerical kernels and early graph/JIT experimentation.

*#cardo[nopokedb: lightweight vector db]* • #link("https://github.com/kashifulhaque/nopokedb")[github] • #link("https://pypi.org/project/nopokedb/")[pypi] • #link("https://blog.ifkash.dev/tiny-vector-db")[blog]
- Disk-backed HNSW vector DB with oplog durability, crash recovery, and minimal RAM usage.
- Fast metadata lookups + efficient ANN search via SQLite + hnswlib.

*#cardo[Boo: AI-powered Discord Bot]* • #link("https://github.com/VVIP-Kitchen/boo")[github] • #link("https://deepwiki.com/VVIP-Kitchen/boo")[deepwiki] • #link("https://vvip.ifkash.dev/blog/boo")[blog]
- Agentic, containerized code-execution sandbox with filesystem isolation, resource limits, deterministic traces, and multi-step tool APIs.
- Added unit-test–based evaluation hooks and structured feedback signals, forming basis for RL coding environments.

= #cardo[education]
*#cardo[IIT Madras]* BS, _Data Science and Applications_ #h(1fr) 2020 -- 2024 \

= #cardo[skills]
- python, go, rust, c++, pytorch, vllm, cuda, docker, k8s, redis, postgres, agentic systems (code sandboxes, test-based evals)
