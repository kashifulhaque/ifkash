#let bg-color = rgb("#1e2a23") // Dark forest green
#let text-color = rgb("#e8f1e8") // Light mint text
#let accent-color = rgb("#3a7d44") // Medium green accent

#set text(fill: text-color, font: "Crimson Text")
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
#line(stroke: 1pt + gray)

3 YOE building ML platforms & developer tooling (FastAPI, Docker), fine-tuning and deploying scalable GenAI models (LoRA, vLLM) across distributed systems. \

= Work Experience
#line(stroke: 1pt + gray)

*Engineer III* #h(1fr) 02/2025 -- Present \
_American Express (IntraEdge)_ #h(1fr) _Bengaluru, India_ \
- Shipped a real-time NLP pipeline that live summarizes 1,000+ WebEx meetings weekly, cutting down manual meeting note taking
- Built an NLP-powered Confluence search service (FastAPI, PGVector) indexing 2M+ pages, supporting sub-1500ms 95th-percentile search results latency
- Tech stack: _Python, FastAPI, Langchain, PGVector, Streamlit, Node.js, Puppeteer, WebEx JS SDK, OpenShift_
\
*Associate Software Engineer* #h(1fr) 07/2023 -- 02/2025 \
_Fiery (an Epson company, formerly known as EFI)_ #h(1fr) _Bengaluru, India_ \
- Fine-tuned Mistral‑7B using LoRA adapters on 4070 Ti Super, baking company knowledge into model
- Deployed the fine-tuned model on a 4x1080 Ti GPU cluster via vLLM; engineered dynamic batching and employed mixed-precision inference for sub-750ms 95th-percentile TTFT
- #link("https://www.printweek.com/content/news/fiery-shows-off-new-ai-features-at-printing-united#:~:text=Brand%20new%20at%20Printing%20United%20is%20Fiery%E2%80%99s%20Ticketing%20Assistant%20software%2C%20currently%20in%20development%20for%20a%20late%202024%20launch.%20Leaning%20on%20large%20language%20models%20(LLMs)%20of%20AI%2C%20the%20programme%20can%20read%20emails%20and%20automatically%20translate%20them%20into%20job%20tickets.")[Led development of "Beacon"], an NER-based print-request automation service integrating with emails and natural language interfaces, leveraging a custom fine-tuned ModernBERT model
- Built AskDB, a natural language to insights platform, used by 300+ internal users to query company DBs and auto-generate visual insights
- Tech stack: _vLLM, Python, PyTorch, LoRA, SFT, Mistral, Llama, Ollama, SQL, FastAPI_
\
*Data Scientist, Intern* #h(1fr) 01/2023 -- 07/2023 \
_Fiery (an Epson company, formerly known as EFI)_ #h(1fr) _Bengaluru, India_ \
- Created a product mockup pipeline using ImageMagick and Node.js, achieving cost-effective design automation
- Tech stack: _Node.js, ImageMagick, Angular_
\
*Fullstack Developer, Intern* #h(1fr) 07/2022 -- 12/2022 \
_Corteva Agriscience_ #h(1fr) _Hyderabad, India_ \
- Migrated Flask monolith to microservices (Python, Docker, Kubernetes), optimizing AutoML job triggering and reducing job-launch latency by 40%
- Tech stack: _Python, Flask, Docker, Kubernetes_

= Education
#line(stroke: 1pt + gray)

*Indian Institute of Technology Madras* #h(1fr) 2020 -- 2024 \
Bachelor of Science, _Data Science and Applications_ \

// = Projects
// #line(stroke: 1pt + gray)

// *Boo* • #link("https://github.com/kashifulhaque/boo")[git repo] \
// _Python, Discord.py, Go, PostgreSQL, Cloudflare Workers, Linode_

= Skills
#line(stroke: 1pt + gray)

- Python, TypeScript, C++, Go
- FastAPI, Flask, SQLAlchemy, Docker, Kubernetes, vLLM, Hugging Face, LoRA
- PyTorch, NumPy, Pandas, scikit-learn, spaCy, NLTK, CUDA, Langchain
- MySQL, PostgreSQL, SQLite, Redis, Chroma, Qdrant, PGVector
