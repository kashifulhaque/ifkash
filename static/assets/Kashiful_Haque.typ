#set text(font: "Crimson Pro")
#let cardo = text.with(font: "Cardo")

#show link: underline
#set page(margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

#let chiline() = {v(-3pt); line(length: 100%); v(-5pt)}

= #cardo[Kashiful Haque]
+918240868544 • #link("mailto:haque.kashiful7@gmail.com")[haque.kashiful7\@gmail.com] • #link("https://github.com/kashifulhaque")[github] • #link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • #link("https://ifkash.dev")[ifkash.dev]

Applied ML Systems Engineer; bridging models with production infra. Experienced in fine-tuning LLMs, optimizing inference (vLLM, quantization, batching) and building scalable AI-powered applications (Docker, K8s, Redis, Postgres). Proven track record of deploying research into production systems, adopted across teams organization wide. \

= #cardo[work experience]
*#cardo[Wand AI (via Nityo)]* #h(1fr) _Palo Alto, USA_ \
_Backend AI/ML Engineer_ #h(1fr) 11/2025 -- Present \

*#cardo[American Express (via IntraEdge)]* #h(1fr) _Bengaluru, India_ \
_Engineer III_ #h(1fr) 02/2025 -- 11/2025 \
- Architected a low-latency streaming infra (Redis + GPT-4o summarization) for thousands of meetings; deployed to RedHat OpenShift (Amex Hydra) with Helm/Jenkins.
- Built a hybrid semantic + trigram retrieval system over 200K+ Confluence pages (runbooks); reduced latency from 30s (native search) to under 2s.

*#cardo[Fiery]* #h(1fr) _Bengaluru, India_ \
_Associate Software Engineer_ #h(1fr) 01/2023 — 02/2025 \
- Fine-tuned Mistral-7B using QLoRA on 4070 Ti Super for enterprise knowledge injection, incorporated into a RAG pipeline.
- Built efficient inference at scale via vLLM on NVIDIA T4 cluster; leveraged dynamic batching and quantized inference for sub-second 95th-percentile latency (TTFT).
- #link("https://www.fiery.com/wp-content/uploads/sites/3/2025/09/Fiery_Brochure-Fiery-Scribe_Br-US.pdf")[Led development of "Fiery Scribe"], a print-request automation tool powered by a fine-tuned ModernBERT NER model; parsed email and chat inputs to trigger document workflows. #link("https://www.printweek.com/content/news/fiery-shows-off-new-ai-features-at-printing-united#:~:text=Brand%20new%20at%20Printing%20United%20is%20Fiery%E2%80%99s%20Ticketing%20Assistant%20software%2C%20currently%20in%20development%20for%20a%20late%202024%20launch.%20Leaning%20on%20large%20language%20models%20(LLMs)%20of%20AI%2C%20the%20programme%20can%20read%20emails%20and%20automatically%20translate%20them%20into%20job%20tickets.")[Demoed at Printing United 2024, Las Vegas.]
- Built AskDB, an agentic AI system for natural language → SQL, adopted by 300+ active users across departments.

*#cardo[Corteva Agriscience]* #h(1fr) _Hyderabad, India_ \
_Internship_ #h(1fr) 07/2022 -- 12/2022 \
- Migrated old Flask monolith API to microservices architecture to speed up deployments.

= #cardo[projects]
*#cardo[smoltorch: a tiny pytorch]* • #link("https://github.com/kashifulhaque/smoltorch")[github] • #link("https://pypi.org/project/smoltorch/")[pypi] • #link("https://blog.ifkash.dev/smoltorch")[blog]
- A smol pytorch-like autograd engine to learn the internals of a model autograd framework
- Built on top of numpy tensors

*#cardo[NoPokeDB: a Lightweight Vector Database]* • #link("https://github.com/kashifulhaque/nopokedb")[github] • #link("https://pypi.org/project/nopokedb/")[pypi] • #link("https://blog.ifkash.dev/tiny-vector-db")[blog]
- Designed a disk-backed vector db in Python using hnswlib for ANN search and SQLite for metadata.
- Added durability via a write-ahead oplog with crash recovery; supported batch inserts, auto-resize, CRUD.
- 2K+ PyPI downloads!

*#cardo[Mini-Numpy in Rust]* • #link("https://github.com/kashifulhaque/tinyndarray")[github]
- Implemented a lightweight NumPy clone in Rust with Python bindings, exploring numerical backend design for ML frameworks.

*#cardo[Boo: AI-powered Discord Bot]* • #link("https://github.com/VVIP-Kitchen/boo")[github] • #link("https://deepwiki.com/VVIP-Kitchen/boo")[deepwiki] • #link("https://vvip.ifkash.dev/blog/boo")[blog]
- Architected a multi-service infra with LLM orchestration for an AI bot.
- Features: conversational AI (LLMs), image analysis & generation, weather, GIF & HN search via function/tool calling.

= #cardo[education]
*#cardo[IIT Madras]* BS, _Data Science and Applications_ #h(1fr) 2020 -- 2024 \

= #cardo[skills]
- pytorch, huggingface, peft, qlora, vllm, ollama, vector db, embeddings, inference optimization, cuda
- python, go, rust, fastapi, docker, k8s, redis, postgres, sql, helm, jenkins
