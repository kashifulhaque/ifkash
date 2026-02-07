#set text(font: "Crimson Pro")
#let cardo = text.with(font: "Cardo")

#show link: underline
#set page(margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

= #cardo[Kashiful Haque]
+91 8240868544 • #link("mailto:me@ifkash.dev")[me\@ifkash.dev] • #link("https://github.com/kashifulhaque")[github] • #link("https://hf.co/ifkash")[huggingface] • #link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • #link("https://ifkash.dev")[ifkash.dev]

ML Engineer with 4 YOE building high-throughput services and scalable hybrid search systems. Deep systems expertise, including pre-training LLMs from scratch (360M parameters on H100) and developing custom inference engines in C++ and Rust. Passionate about optimizing ML internals and low-level model architecture.

= #cardo[Work Experience]

*#cardo[wand.ai]* #h(1fr) _Palo Alto (Remote)_ \
_Backend AI / ML Engineer_ #h(1fr) 11/2025 -- Present \
- Engineered a deterministic execution runtime for agent workflows using a custom DSL, enforcing strict typing to mitigate LLM hallucination risks.

*#cardo[American Express]* #h(1fr) _Bangalore_ \
_Engineer III_ #h(1fr) 02/2025 -- 11/2025 \
- Built hybrid kb search over 200k+ documents using dense embeddings + keyword retrieval.
- Reduced p95 latency from 30s to under 2s by refactoring pipelines, caching, and optimizing I/O + query execution.


*#cardo[Fiery]* #h(1fr) _Bangalore_ \
_Associate Software Engineer_ #h(1fr) 01/2023 -- 02/2025 \
- Finetuned domain LLMs using SFT/QLoRA, serving via vLLM on T4 clusters with sub-second p95 TTFT.
- Built Fiery Scribe, an NLP system to translate print requests to printer XML using ModernBERT + LLM.
- Developed AskDB, an LLM agent that maps user inputs to SQL over prod DBs and generate reports, eliminating manual requests for business KPIs.


= #cardo[Projects]

*#cardo[smol-llama: High-efficiency 360M LLaMA Pre-training]* • #link("https://github.com/weights-and-wires/smol-llama")[github] • #link("https://huggingface.co/weights-and-wires/smol-llama")[huggingface] \
- Designed and trained a 360M LLaMA model from scratch in PyTorch, featuring GQA, RoPE, RMSNorm, and SwiGLU.
- Pre-trained on 6B tokens of FineWeb on 1x H100, achieving 75k tok/s throughput via FlashAttention.
- Engineered a cost-effective training pipeline (<\$60 total) with gradient accumulation, automatic checkpoint versioning to Hugging Face, and W&B experiment tracking.

*#cardo[smoltorch: Minimal Autograd Engine]* • 
#link("https://github.com/kashifulhaque/smoltorch")[github] • 
#link("https://pypi.org/project/smoltorch/")[pypi] • 
#link("https://ifkash.dev/blog/smoltorch")[blog] \
- Implemented a reverse-mode autograd engine with tape-based computation graphs and topological scheduling.
- Designed NumPy-backed tensor ops with a minimal training loop inspired by PyTorch internals.

*#cardo[ferray: Rust Based N-dim Array]* • 
#link("https://github.com/kashifulhaque/ferray")[github] \
- Built stride-aware ndarray with slicing and broadcasting, mirroring tensor layouts used in ML frameworks.
- Exposed Rust numerical kernels to Python via PyO3 for efficient tensor operations and future JIT/acceleration work.

*#cardo[nopokedb: Lightweight Vector Database]* • 
#link("https://github.com/kashifulhaque/nopokedb")[github] • 
#link("https://pypi.org/project/nopokedb/")[pypi] • 
#link("https://ifkash.dev/blog/tiny-vector-db")[blog] \
- Designed a disk-backed HNSW vector database with crash recovery, oplog durability, and low memory footprint.
- Implemented fast ANN search and metadata filtering, relevant for retrieval-augmented and domain-adapted NLP systems.

*#cardo[banana.cpp: Optimized C++ Inference Engine]* • 
#link("https://github.com/kashifulhaque/banana.cpp")[github] \
- Building an LLM inference engine with KV-cache, speculative decoding and continuous batching.
- Achieved 10x speedup through CPU parallelization + fused kernel optimizations.

= #cardo[Education]

*#cardo[IIT Madras]* \
BS, _Data Science and Applications_ #h(1fr) 2020 -- 2024

= #cardo[Skills]

- Python, Go, Rust, C++
- PyTorch, LLM fine-tuning, quantization (QLoRA), vLLM, RAG
- Docker, Kubernetes, OpenShift, Redis, PostgreSQL
