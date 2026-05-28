#set text(font: "Crimson Pro")
#let cardo_headings = text.with(font: "Cardo", size: 0.8em)
#let cardo = text.with(font: "Cardo", size: 1em)

#show link: underline
#set page(margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

= #cardo_headings[Kashiful Haque]
+91 8240868544 • #link("mailto:me@ifkash.dev")[me\@ifkash.dev] • #link("https://github.com/kashifulhaque")[github] • #link("https://hf.co/ifkash")[huggingface] • #link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • #link("https://ifkash.dev")[ifkash.dev]

ML Engineer with 4 YOE across LLM pre-training, post-training with RL, and high-performance inference in C++ and Rust. Pre-trained a 360M LLaMA from scratch, fine-tuned production LLMs, and applied GRPO/ORPO for behavioral alignment. Targeting roles in pre-training, RL-based post-training, and GPU inference — CUDA, HPC, and low-level runtime work.

= #cardo_headings[Work Experience]

*#cardo[wand.ai]* #h(1fr) _Bangalore_ \
_Backend AI / ML Engineer_ #h(1fr) 11/2025 -- Present \
- Built observability for agent workflows: span hierarchy, trace IDs, and Kafka-based monitoring events for e2e run visibility.
- Implemented OAuth/OIDC credential flows with deferred auth, token refresh, and variable propagation across agentic tasks.
- Built workflow specs: generates auth gateways, OAuth triggers, and injects credential references into tool environments.
- Added dynamic tool support: agents can define, reference, and execute tools that resolve workflow state variables at runtime.

*#cardo[American Express]* #h(1fr) _Bangalore_ \
_Engineer III_ #h(1fr) 02/2025 -- 11/2025 \
- Built a hybrid retrieval system over 200k+ internal docs combining dense embeddings and keyword retrieval; cut p95 query latency from 30s to 2s via caching and query optimizations.
- Shipped a live Webex meeting-summary bot (headless Chrome + Webex SDK as virtual participant); streamed real-time transcripts, generated rolling LLM summaries, and posted updates to Slack every 5 minutes.
- Profiled large-scale transformer training and inference (FlashAttention, KV-cache, batching) to improve embedding throughput and serving efficiency.

*#cardo[Fiery]* #h(1fr) _Bangalore_ \
_Associate Software Engineer_ #h(1fr) 01/2023 -- 02/2025 \
- Built Fiery Scribe: translates natural-language print requests to printer XML using ModernBERT + LLM, replacing brittle rule-based parsing.
- Built AskDB: an LLM agent that maps natural language to SQL over production DBs and generates business reports for non-technical users.
- Built Fiery Chat: internal RAG assistant on the company knowledge base; fine-tuned the base model with SFT to eliminate acronym hallucinations across new-hire onboarding and query routing.
- Fine-tuned domain LLMs using SFT/QLoRA, served via vLLM on T4 clusters with sub-second p95 TTFT.


= #cardo_headings[ML Engineering Work]

*#cardo[banana.cpp: LLM Inference Engine in C++]* • 
#link("https://github.com/kashifulhaque/banana.cpp")[github] \
- Inference engine with KV-cache, speculative decoding, and continuous batching; 10x speedup via CPU parallelization and fused kernel optimizations.
- Focused on Apple M-series silicon as a learning project into low-level inference mechanics.

*#cardo[Llama-3.2 3B Politeness Alignment using ORPO]* •
#link("https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO")[hf] •
#link("https://blog.ifkash.dev/teaching-llama-3-to-be-polite")[blog] \
- Applied ORPO with TRL + Unsloth to align Llama-3.2 3B to a behavioral policy (respond politely or refuse).
- Built a 32k-row preference dataset with behavioral constraints; validated convergence via reward margins and log-prob dynamics in W&B.

*#cardo[smol-llama: 360M LLM pre-training]* • #link("https://github.com/weights-and-wires/smol-llama")[github] • #link("https://huggingface.co/weights-and-wires/smol-llama")[hf] •
#link("https://api.wandb.ai/links/dotslasha/lydqsle8")[wandb] •
#link("https://blog.ifkash.dev/smol-llama-pretraining")[blog] \
- Trained a 360M LLaMA from scratch in PyTorch with GQA, RoPE, RMSNorm, and SwiGLU.
- Pre-trained on 6B tokens using FlashAttention on H100 at 75K tokens/sec; built a cost-efficient pipeline with gradient accumulation and checkpointing.

= #cardo_headings[Education]

*#cardo[IIT Madras]* \
BS, _Data Science and Applications_ #h(1fr) 2020 -- 2024

= #cardo_headings[Skills]

- Python, C++, Rust, Go
- PyTorch, FlashAttention, CUDA, vLLM, TRL, QLoRA, Unsloth, Docker, Kubernetes, Redis, PostgreSQL
