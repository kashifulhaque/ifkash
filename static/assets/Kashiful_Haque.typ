#set text(font: "Crimson Text")

#show link: underline
#set page(margin: (x: 0.9cm, y: 1.3cm))
#set par(justify: true)

#let chiline() = {v(-3pt); line(length: 100%); v(-5pt)}

= Kashiful Haque
+918240868544 • #link("mailto:haque.kashiful7@gmail.com")[haque.kashiful7\@gmail.com] • #link("https://github.com/kashifulhaque")[github] • #link("https://www.linkedin.com/in/kashifulhaque")[linkedin] • #link("https://ifkash.dev")[ifkash.dev]
#line(stroke: 1pt + gray)

Software Engineer with 3 YOE building ML platforms & developer tooling (FastAPI, Docker), fine-tuning and deploying scalable GenAI models (LoRA, vLLM) across distributed systems. \

= Work Experience
#line(stroke: 1pt + gray)

*Engineer III* #h(1fr) 02/2025 -- Present \
_American Express (client for IntraEdge)_ #h(1fr) _Bengaluru, India_ \
- Designed & shipped a real-time NLP pipeline (JS, Webex APIs, OpenAI) that ingests and summarizes 1,000+ live WebEx transcripts summary, cutting down manual meeting note taking.
- Building a search engine for company confluence pages, can be queried using natural language
- Built an NLP-powered Confluence search service (FastAPI, PGVector) indexing 2M+ pages, supporting sub-500ms 95th-percentile latency and 90% conversational-query accuracy.
\
*Associate Software Engineer* #h(1fr) 07/2023 -- 02/2025 \
_Fiery (an Epson company, formerly known as EFI)_ #h(1fr) _Bengaluru, India_ \
- Deployed LLaMa 3.1-8B via vLLM on a 4070 Ti Super cluster for ~500 users; engineered dynamic batching and employed mixed-precision inference to maintain sub-300ms 95th-percentile TTFT
- Fine-tuned Mistral‑7B with LoRA adapters on 4×1080 Ti GPUs, embedding domain knowledge and boosting task accuracy by 12%.
- #link("https://www.printweek.com/content/news/fiery-shows-off-new-ai-features-at-printing-united#:~:text=Brand%20new%20at%20Printing%20United%20is%20Fiery%E2%80%99s%20Ticketing%20Assistant%20software%2C%20currently%20in%20development%20for%20a%20late%202024%20launch.%20Leaning%20on%20large%20language%20models%20(LLMs)%20of%20AI%2C%20the%20programme%20can%20read%20emails%20and%20automatically%20translate%20them%20into%20job%20tickets.")[Led development of "Beacon"], an NER-based print-request automation service integrating with emails and natural language interfaces, leveraging a custom fine-tuned ModernBERT model.
- Built AskDB and audio fingerprinting solutions; integrated RESTful APIs with Redis caching
\
*Data Scientist, Intern* #h(1fr) 01/2023 -- 07/2023 \
_Fiery (an Epson company, formerly known as EFI)_ #h(1fr) _Bengaluru, India_ \
- Created a product mockup pipeline using ImageMagick and Node.js, achieving cost-effective design automation.
\
*Fullstack Developer, Intern* #h(1fr) 07/2022 -- 12/2022 \
_Corteva Agriscience_ #h(1fr) _Hyderabad, India_ \
- Migrated Flask monolith to microservices (Python, Docker, Kubernetes), optimizing AutoML job triggering and reducing job-launch latency by 40%.

= Education
#line(stroke: 1pt + gray)

*Indian Institute of Technology Madras* #h(1fr) 2020 -- 2024 \
Bachelor of Science, _Data Science and Applications_ \

= Projects
#line(stroke: 1pt + gray)

*Boo* • #link("https://github.com/kashifulhaque/boo")[git repo] \
_Python, Discord.py, Go, PostgreSQL, Cloudflare Workers, Linode_

= Skills
#line(stroke: 1pt + gray)

- Python, TypeScript, C++, Rust, Go
- FastAPI, Flask, SQLAlchemy, Docker, Kubernetes, vLLM, Hugging Face Transformers, LoRA
- PyTorch, NumPy, Pandas, scikit-learn, spaCy, NLTK, CUDA, Langchain
- MySQL, PostgreSQL, SQLite, Redis, Vector DBs, Chroma, Qdrant, PGVector
