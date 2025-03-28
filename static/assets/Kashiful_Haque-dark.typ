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

Software Engineer with almost 3 YOE in Machine Learning and Backend Development. Current MLE \@ AMEX, formerly worked as Associate SE \@ Fiery (an Epson company). Studied Data Science at IIT Madras. \

= Work Experience
#line(stroke: 1pt + gray)

*Machine Learning Engineer* #h(1fr) 02/2025 -- Present \
_American Express_ #h(1fr) _Bengaluru, India_ \
- client for IntraEdge
\
*Associate Software Engineer* #h(1fr) 07/2023 -- 02/2025 \
_Fiery (an Epson company)_ #h(1fr) _Bengaluru, India_ \
- Built AskDB, a platform for upper management to know how the company is performing.
- Built an audio fingerprinting solution to identify ads played on TV.
- Built RAG based FieryGPT using LLaMa 3.1 LLM, trained LoRA adapter on company data for QnA.
- #link("https://www.printweek.com/content/news/fiery-shows-off-new-ai-features-at-printing-united#:~:text=Brand%20new%20at%20Printing%20United%20is%20Fiery%E2%80%99s%20Ticketing%20Assistant%20software%2C%20currently%20in%20development%20for%20a%20late%202024%20launch.%20Leaning%20on%20large%20language%20models%20(LLMs)%20of%20AI%2C%20the%20programme%20can%20read%20emails%20and%20automatically%20translate%20them%20into%20job%20tickets.")[Led development of "Beacon"], used LLM to automate print requests coming via email and natural language.
- #text(weight: "medium")[Tech:] TypeScript, Python, FastAPI, huggingface, Transformers, vLLM, ollama, ImageMagick, PyTorch, Qdrant, Docker, Numpy, Pandas
\
*Data Scientist, Intern* #h(1fr) 01/2023 -- 07/2023 \
_Electronics for Imaging_ #h(1fr) _Bengaluru, India_ \
- Cost-effective results compared to costly Photoshop APIs.
- Built a product mockup solution using ImageMagick and Node.js
- #text(weight: "medium")[Tech:] TypeScript, Angular, Node.js, Python, Flask, ImageMagick, MySQL, NLTK, spaCy
\
*Fullstack Developer, Intern* #h(1fr) 07/2022 -- 12/2022 \
_Corteva Agriscience_ #h(1fr) _Hyderabad, India_ \
- Migrated Flask monolith to scalable APIs, separating out backend and frontend.
- Codebase optimizations which resulted faster AutoML job triggering on Kubernetes.
- #text(weight: "medium")[Tech:] Python, Flask, Docker

= Education
#line(stroke: 1pt + gray)

*Indian Institute of Technology Madras* #h(1fr) 2020 -- 2024 \
Bachelor of Science, _Data Science and Applications_ \

= Projects
#line(stroke: 1pt + gray)

*Boo* • #link("https://github.com/kashifulhaque/boo")[git repo] \
_Python, Discord.py, Go, PostgreSQL, Cloudflare Workers, Linode_

*Odeer* • #link("https://github.com/kashifulhaque/odeer")[git repo]  \
_Go, Gin, Cloudflare_

= Skills
#line(stroke: 1pt + gray)

- Python, TypeScript, C, C++, Rust, Go
- Node.js, Express.js, Vue, Angular, FastAPI, Flask, Tailwind
- NumPy, Pandas, scikit-learn, Pytorch, huggingface, JAX, NLTK, spaCy, CUDA
- Vector database, Qdrant, SQL, MySQL, SQLite, Redis, Docker, Git, Linux
