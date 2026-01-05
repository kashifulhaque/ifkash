#set text(font: "Crimson Pro")
#let cardo = text.with(font: "Cardo")

#show link: underline
#set page(margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

#let chiline() = {v(-3pt); line(length: 100%); v(-5pt)}

= #cardo[Kashiful Haque]
+91 8240868544 • #link("mailto:me@ifkash.dev")[me\@ifkash.dev] • 
#link("https://github.com/kashifulhaque")[github] • 
#link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • 
#link("https://ifkash.dev")[ifkash.dev]

= #cardo[Work Experience]

*#cardo[wand.ai]* #h(1fr) _Palo Alto (Remote)_ \
_Backend AI / ML Engineer_ #h(1fr) 11/2025 -- Present \
- Building AI agents to automate workforce.

*#cardo[American Express]* #h(1fr) _Bangalore_ \
_Engineer III_ #h(1fr) 02/2025 -- 11/2025 \
- Engineered a real-time system to capture Webex transcripts and generate automated GPT-4o summaries.
- Built large-scale search over 200k+ docs using hybrid semantic + lexical retrieval, reducing p95 latency from 30s to under 2s.

*#cardo[Fiery]* #h(1fr) _Bangalore_ \
_Associate Software Engineer_ #h(1fr) 01/2023 -- 02/2025 \
- Fine-tuned LLMs using SFT/QLoRA; profiled kernel-level bottlenecks during training and inference.
- High-throughput inference using vLLM, achieving sub-second p95 time-to-first-token on T4 GPU clusters.
- Developed Fiery Scribe, an automation engine that translates natural language into printer instructions via fine-tuned ModernBERT model, slashing costs by replacing GPU-dependent LLMs.
- Developed "AskDB," an AI agent that converts queries into SQL and Python-generated reports, eliminating manual data requests and providing instant access to business KPIs.

*#cardo[Corteva Agriscience]* #h(1fr) _Hyderabad_ \
_Internship_ #h(1fr) 07/2022 -- 12/2022 \
- Worked on applied ML pipelines and migration of Flask monolith to microservices.

= #cardo[Projects]

*#cardo[smoltorch: Minimal Autograd Engine]* • 
#link("https://github.com/kashifulhaque/smoltorch")[github] • 
#link("https://pypi.org/project/smoltorch/")[pypi] • 
#link("https://ifkash.dev/blog/smoltorch")[blog] \
- Implemented a reverse-mode autograd engine with tape-based computation graphs and topological scheduling.
- Designed NumPy-backed tensor ops with a minimal training loop inspired by PyTorch internals.

*#cardo[tinyndarray: Mini NumPy in Rust]* • 
#link("https://github.com/kashifulhaque/tinyndarray")[github] \
- Built stride-aware ndarray with slicing and broadcasting, mirroring tensor layouts used in ML frameworks.
- Exposed Rust numerical kernels to Python via PyO3 for efficient tensor operations and future JIT/acceleration work.

*#cardo[nopokedb: Lightweight Vector Database]* • 
#link("https://github.com/kashifulhaque/nopokedb")[github] • 
#link("https://pypi.org/project/nopokedb/")[pypi] • 
#link("https://ifkash.dev/blog/tiny-vector-db")[blog] \
- Designed a disk-backed HNSW vector database with crash recovery, oplog durability, and low memory footprint.
- Implemented fast ANN search and metadata filtering, relevant for retrieval-augmented and domain-adapted NLP systems.

*#cardo[Boo: AI-powered Discord Bot]* • 
#link("https://github.com/VVIP-Kitchen/boo")[github] • 
#link("https://deepwiki.com/VVIP-Kitchen/boo")[deepwiki] • 
#link("https://vvip.ifkash.dev/blog/boo")[blog] \
- Built a distributed, multi-service AI agent with realtime chat, multimodal vision analysis and tool-calling.
- Engineered a secure execution environment, a coding gym, using a sandboxed Python runtime and handed it over to Boo.

= #cardo[Education]

*#cardo[IIT Madras]* \
BS, _Data Science and Applications_ #h(1fr) 2020 -- 2024

= #cardo[Skills]

- Python, Go, Rust, C++
- PyTorch, LLM fine-tuning, quantization (QLoRA), vLLM, ModernBERT, RAG systems
- CUDA-aware profiling, dynamic batching, low-latency inference, distributed pipelines
- Docker, Kubernetes, OpenShift, Redis, PostgreSQL
- Paper implementation, structured evals, error analysis, test-based feedback, agentic environments
