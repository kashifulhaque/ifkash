#set text(font: "Crimson Pro")
#let cardo_headings = text.with(font: "Cardo", size: 0.8em)
#let cardo = text.with(font: "Cardo", size: 1em)

#show link: underline
#set page(margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

= #cardo_headings[Kashiful Haque]
+91 8240868544 • #link("mailto:me@ifkash.dev")[me\@ifkash.dev] • #link("https://github.com/kashifulhaque")[github] • #link("https://hf.co/ifkash")[huggingface] • #link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • #link("https://ifkash.dev")[ifkash.dev]

ML Systems Engineer with 4 YOE building production LLM systems and high-throughput search pipelines, with hands-on experience training transformer models from scratch and implementing low-level ML infrastructure in C++, Rust, and Python.

= #cardo_headings[Work Experience]

*#cardo[wand.ai]* #h(1fr) _Palo Alto (Remote)_ \
_Backend AI / ML Engineer_ #h(1fr) 11/2025 -- Present \
- Designed execution runtime for agent workflows, a statically-typed DSL and state isolation to reduce non-determinism.

*#cardo[American Express]* #h(1fr) _Bangalore_ \
_Engineer III_ #h(1fr) 02/2025 -- 11/2025 \
- Built a hybrid KB search over 200k+ docs using dense embeddings and keyword retrieval, serving internal applications.
- Reduced p95 query latency from 30s to 2s via aggressive caching, and query execution optimizations.
- Independently conducted large-scale transformer training and profiling experiments (360M parameters, multi-billion token runs) to deepen understanding of attention mechanisms, optimizer behavior, and throughput bottlenecks on modern accelerators.
- Applied insights from pre-training and inference profiling (FlashAttention, KV-cache dynamics, batching tradeoffs) to improve embedding generation, serving efficiency, and system-level performance.

*#cardo[Fiery]* #h(1fr) _Bangalore_ \
_Associate Software Engineer_ #h(1fr) 01/2023 -- 02/2025 \
- Finetuned domain LLMs using SFT/QLoRA, serving via vLLM on T4 clusters with sub-second p95 TTFT.
- Built Fiery Scribe, an NLP system to translate print requests to printer XML using ModernBERT + LLM.
- Developed AskDB, an LLM agent that maps user inputs to SQL over prod DBs and generate reports, eliminating manual requests for business KPIs.


= #cardo_headings[ML Engineering Work]

*#cardo[smol-llama: efficient transformer pre-training (360M)]* • #link("https://github.com/weights-and-wires/smol-llama")[github] • #link("https://huggingface.co/weights-and-wires/smol-llama")[hf] •
#link("https://api.wandb.ai/links/dotslasha/lydqsle8")[wandb] •
#link("https://ifkash.dev/blog/smol-llama-pretraining")[blog] \
- Trained a 360M LLaMA-style LLM from scratch in PyTorch, implementing GQA, RoPE, RMSNorm, and SwiGLU.
- Pre-trained on 6B tokens using FlashAttention on H100, achieving 75K tokens/sec throughput.
- Built a cost-efficient training pipeline with gradient accumulation, checkpointing, and experiment tracking.

*#cardo[smoltorch: minimal autograd engine]* • 
#link("https://github.com/kashifulhaque/smoltorch")[github] • 
#link("https://pypi.org/project/smoltorch/")[pypi] • 
#link("https://ifkash.dev/blog/smoltorch")[blog] \
- Implemented a reverse-mode autograd engine with tape-based computation graphs and topological scheduling to study gradient propagation, memory lifetimes, and operator execution.
- Built NumPy-backed tensor ops and a minimal training loop inspired by PyTorch internals.

*#cardo[ferray: Rust Based N-dim Array]* • 
#link("https://github.com/kashifulhaque/ferray")[github] \
- Built a stride-aware ndarray in Rust with slicing and broadcasting, mirroring tensor layouts used in modern ML frameworks.
- Exposed Rust numerical kernels to Python via PyO3 to explore low-level performance and FFI tradeoffs for ML workloads.

*#cardo[banana.cpp: Optimized C++ Inference Engine]* • 
#link("https://github.com/kashifulhaque/banana.cpp")[github] \
- Implemented an LLM inference engine with KV-cache, speculative decoding, and continuous batching.
- Achieved 10x speedup through CPU parallelization + fused kernel optimizations.

= #cardo_headings[Education]

*#cardo[IIT Madras]* \
BS, _Data Science and Applications_ #h(1fr) 2020 -- 2024

= #cardo_headings[Skills]

- **Languages:** Python, C++, Rust, Go  
- **ML Systems:** PyTorch, Transformer training, FlashAttention, KV-cache, vLLM, QLoRA  
- **Infra & Data:** Docker, Kubernetes, Redis, PostgreSQL
