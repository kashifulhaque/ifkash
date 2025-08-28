#let bg-color = rgb("#1e2a23") // Dark forest green
#let text-color = rgb("#e8f1e8") // Light mint text
#let accent-color = rgb("#3a7d44") // Medium green accent

#set text(fill: text-color, font: "Crimson Pro")
#show link: it => [#underline[#text(fill: rgb("#6baa75"))[#it]]]

#set page(fill: bg-color, margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

#let chiline() = {
  v(-3pt)
  line(length: 100%, stroke: accent-color)
  v(-5pt)
}

= Kashiful Haque
+918240868544 • #link("mailto:haque.kashiful7@gmail.com")[haque.kashiful7\@gmail.com] • #link("https://github.com/kashifulhaque")[github] • #link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • #link("https://ifkash.dev")[ifkash.dev]

Engineer building production-grade AI systems; I have deployed fine-tuned LLMs (and vLLM for inference) into live infra, designed multi-service discord bots with redis/postgres state and delivered reverse image search on the fly during interviews. Experienced in end-to-end ownerships; model training → scalable infra (docker, k8s) → API delivery. My work has been used by hundreds and integrated into customer facing apps. \

= Work Experience
*American Express (via IntraEdge)* #h(1fr) _Bengaluru, India_ \
_Engineer III_ #h(1fr) 02/2025 -- Present \
- Built CC Listener module for Bridge Intelligence, capturing real-time CCs captions for thousands of Webex meetings, streaming them into Redis for downstream summarization using GPT-4o.
- Deployed CC Listener to Amex’s Hydra platform (RedHat OpenShift) across dev, QA, and pre-prod using Helm, Jenkins, and Docker; navigated complex internal infra and CI/CD pipelines.
- Built a hybrid semantic + fuzzy (trigram) retrieval system over 2M+ Confluence pages (runbooks); reduced latency from 30s (native search) to under 2s.
- Designed a multimodal runbook generation pipeline using GPT-4o, converting raw incident data (text + screenshots) into standardized Confluence pages.

*Fiery (an Epson company, formerly EFI)* #h(1fr) _Bengaluru, India_ \
_Associate Software Engineer_ #h(1fr) 07/2023 -- 02/2025 \
- Fine-tuned Mistral-7B using QLoRA on 4070 Ti Super to incorporate internal documentation and company workflows.
- Deployed the fine-tuned model via vLLM on an NVIDIA T4 cluster; leveraged dynamic batching and quantized inference for sub-second 95th-percentile latency (TTFT).
- #link("https://www.printweek.com/content/news/fiery-shows-off-new-ai-features-at-printing-united#:~:text=Brand%20new%20at%20Printing%20United%20is%20Fiery%E2%80%99s%20Ticketing%20Assistant%20software%2C%20currently%20in%20development%20for%20a%20late%202024%20launch.%20Leaning%20on%20large%20language%20models%20(LLMs)%20of%20AI%2C%20the%20programme%20can%20read%20emails%20and%20automatically%20translate%20them%20into%20job%20tickets.")[Led development of "Beacon"], a print-request automation tool powered by a fine-tuned ModernBERT NER model; parsed email and chat inputs to trigger document workflows.
- Built AskDB, an AI agent used by 300+ internal users; used llama 3.1 to translate business queries into SQL, fetch data and auto-generate charts and summaries.

*Fiery (an Epson company, formerly EFI)* #h(1fr) _Bengaluru, India_ \
_Internship_ #h(1fr) 01/2023 -- 07/2023 \
- Built product mockup pipeline using ImageMagick, achieving cost-effective design automation.

*Corteva Agriscience* #h(1fr) _Hyderabad, India_ \
_Internship_ #h(1fr) 07/2022 -- 12/2022 \
- Migrated Flask monolith to containerized microservices, reducing AutoML job-launch latency by 40%.

= Projects

*NoPokeDB: a Lightweight Vector Database* • #link("https://github.com/kashifulhaque/nopokedb")[github] • #link("https://pypi.org/project/nopokedb/")[pypi] • #link("https://blog.ifkash.dev/tiny-vector-db")[blog]
- Designed a disk-backed vector db in Python using hnswlib for ANN search and SQLite for metadata.
- Added durability via a write-ahead oplog with crash recovery; supported batch inserts, auto-resize, CRUD.

*Mini-Numpy in Rust* • #link("https://github.com/kashifulhaque/tinyndarray")[github]
- Implemented a lightweight NumPy clone in Rust with Python bindings to deeply understand tensor internals and performance optimizations in ML frameworks.

*Boo: AI-powered Discord Bot* • #link("https://github.com/VVIP-Kitchen/boo")[github] • #link("https://deepwiki.com/VVIP-Kitchen/boo")[deepwiki]
- Architected a multi-service AI bot running as a Docker Compose stack with multiple services.
- Features: conversational AI (LLMs), image analysis & generation, weather, GIF & HN search via function/tool calling.

= Education
*IIT Madras* BS, _Data Science and Applications_ #h(1fr) 2020 -- 2024 \

= Skills
- python, typescript, golang, rust, c++, sql
- fastapi, docker, k8s, vllm, ollama, huggingface, peft, qlora
- pytorch, numpy, pandas, sklearn, spacy, cuda, redis, vector db