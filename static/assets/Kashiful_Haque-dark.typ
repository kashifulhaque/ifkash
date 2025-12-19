#let bg-color = rgb("#0f1c14")       // Deep blackened green
#let text-color = rgb("#eaf9f0")     // Mint-ivory
#let accent-color = rgb("#3a7d44")   // Emerald green
#let accent2-color = rgb("#ffd166")  // Warm golden highlight

#set text(font: "Crimson Pro", fill: text-color)
#let cardo = text.with(font: "Cardo", fill: text-color)
#show link: it => [#underline[#text(fill: rgb("#6baa75"))[#it]]]

#set page(fill: bg-color, margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

#let chiline() = {
  v(-3pt)
  line(length: 100%, stroke: accent-color)
  v(-5pt)
}

= #cardo[Kashiful Haque]
+91 8240868544 • #link("mailto:haque.kashiful7@gmail.com")[haque.kashiful7\@gmail.com] • 
#link("https://github.com/kashifulhaque")[github] • 
#link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • 
#link("https://ifkash.dev")[ifkash.dev]

Systems-oriented ML Engineer with 3.5+ years of experience building and optimizing large-scale LLM inference and training pipelines. Strong focus on model internals, acceleration techniques (quantization, QLoRA, batching), and structured evaluation systems. Experienced in designing low-latency, high-throughput NLP systems and collaborating closely with research teams to translate ideas from papers into production-grade multilingual and domain-adapted models.

= #cardo[Work Experience]

*#cardo[wand.ai]* #h(1fr) _Palo Alto (Remote)_ \
_Backend AI / ML Engineer_ #h(1fr) 11/2025 -- Present \
- Designing execution-layer infrastructure for LLM-based agents, focusing on pipelined inference, tool orchestration, and structured task environments.

*#cardo[American Express]* #h(1fr) _Bangalore_ \
_Engineer III_ #h(1fr) 02/2025 -- 11/2025 \
- Architected distributed, low-latency streaming pipelines with backpressure control and token-level streaming semantics, directly applicable to real-time LLM inference workloads.
- Deployed and optimized production systems on RedHat OpenShift (Kubernetes, Helm, Jenkins), improving system throughput and reliability under high concurrency.
- Rebuilt large-scale search infrastructure over 200k+ documents using hybrid semantic + lexical retrieval, reducing p95 latency from 30s to under 2s.

*#cardo[Fiery]* #h(1fr) _Bangalore_ \
_Associate Software Engineer_ #h(1fr) 01/2023 -- 02/2025 \
- Fine-tuned LLMs (Mistral-7B, Llama-8B) using QLoRA for domain adaptation; instrumented GPU memory usage and profiled kernel-level bottlenecks during training and inference.
- Delivered high-throughput inference pipelines using vLLM with dynamic batching and quantized kernels, achieving sub-second p95 time-to-first-token on T4 GPU clusters.
- Developed Fiery Scribe, an NER-driven automation engine using a fine-tuned ModernBERT model, focused on domain-specific text understanding and robustness.
- Implemented LLM-to-SQL agent systems with partial-plan evaluation, structured failure recovery, and programmatic validation of model outputs.

*#cardo[Corteva Agriscience]* #h(1fr) _Hyderabad_ \
_Internship_ #h(1fr) 07/2022 -- 12/2022 \
- Worked on applied ML pipelines and migration of Flask monolith to microservices.

= #cardo[Projects]

*#cardo[smoltorch: Minimal Autograd Engine]* • 
#link("https://github.com/kashifulhaque/smoltorch")[github] • 
#link("https://pypi.org/project/smoltorch/")[pypi] • 
#link("https://blog.ifkash.dev/smoltorch")[blog] \
- Implemented a reverse-mode autograd engine with tape-based computation graphs and topological scheduling.
- Designed NumPy-backed tensor operations with broadcasting and a minimal training loop inspired by PyTorch internals, deepening understanding of model architecture and optimization mechanics.

*#cardo[tinyndarray: Mini NumPy in Rust]* • 
#link("https://github.com/kashifulhaque/tinyndarray")[github] \
- Built a stride-aware ndarray implementation with slicing and broadcasting, mirroring tensor layouts used in modern ML frameworks.
- Exposed Rust numerical kernels to Python via PyO3, enabling experimentation with efficient tensor operations and future JIT/acceleration work.

*#cardo[nopokedb: Lightweight Vector Database]* • 
#link("https://github.com/kashifulhaque/nopokedb")[github] • 
#link("https://pypi.org/project/nopokedb/")[pypi] • 
#link("https://blog.ifkash.dev/tiny-vector-db")[blog] \
- Designed a disk-backed HNSW vector database with crash recovery, oplog durability, and low memory footprint.
- Implemented fast ANN search and metadata filtering, relevant for retrieval-augmented multilingual and domain-adapted NLP systems.

*#cardo[Boo: AI-powered Discord Bot]* • 
#link("https://github.com/VVIP-Kitchen/boo")[github] • 
#link("https://deepwiki.com/VVIP-Kitchen/boo")[deepwiki] • 
#link("https://vvip.ifkash.dev/blog/boo")[blog] \
- Built a containerized, agentic LLM system with isolated execution, deterministic traces, and structured tool APIs.
- Added unit-test–based evaluation hooks and feedback signals, aligning with preference-based learning and RL-style evaluation paradigms.

= #cardo[Education]

*#cardo[IIT Madras]* \
BS, _Data Science and Applications_ #h(1fr) 2020 -- 2024

= #cardo[Skills]

- *Languages*: Python, Go, Rust, C++
- *ML / NLP*: PyTorch, LLM fine-tuning, quantization (QLoRA), vLLM, ModernBERT, RAG systems
- *Systems & Acceleration*: CUDA-aware profiling, dynamic batching, low-latency inference, distributed pipelines
- *Infra*: Docker, Kubernetes, OpenShift, Redis, PostgreSQL
- *Research & Evaluation*: Paper implementation, structured evals, error analysis, test-based feedback, agentic environments
