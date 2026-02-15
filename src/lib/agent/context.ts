export const NAVIGATION_MAP: Record<string, string> = {
  work: '/work',
  experience: '/work',
  career: '/work',
  jobs: '/work',
  projects: '/projects',
  'banana.cpp': '/projects/banana-cpp',
  'smol-llama': '/projects/smol-llama',
  smoltorch: '/projects/smoltorch',
  nopokedb: '/projects/nopokedb',
  boo: '/projects/boo',
  ferray: '/projects/ferray',
  education: '/education',
  school: '/education',
  university: '/education',
  iit: '/education',
  degree: '/education',
  blog: '/blog',
  resume: '/api/resume?format=view',
  cv: '/api/resume?format=view',
  leetcode: '/leetcode',
  tensara: '/tensara',
  news: '/news',
  home: '/',
};

export const SITE_CONTENT = `
## About Kashiful Haque
ML Systems Engineer currently working as ML Engineer at wand.ai (Palo Alto, remote). 
Building intelligent systems at the intersection of research and production. 
Focused on inference optimization, model training, and scalable AI infrastructure.
Contact: me@ifkash.dev | GitHub: kashifulhaque | HuggingFace: ifkash | LinkedIn: kashifulhaque | Website: ifkash.dev

## Work Experience
1. **ML Engineer at wand.ai** (2025 — Present, Palo Alto, CA, Remote)
   - Designed execution runtime for agent workflows, a statically-typed DSL and state isolation to reduce non-determinism.

2. **Engineer III at American Express** (2025, Bangalore, India)
   - Built a hybrid KB search over 200k+ docs using dense embeddings and keyword retrieval.
   - Reduced p95 query latency from 30s to 2s via aggressive caching and query execution optimizations.
   - Conducted large-scale transformer training experiments (360M parameters, multi-billion token runs).
   - Applied insights from pre-training and inference profiling (FlashAttention, KV-cache dynamics, batching tradeoffs).

3. **Software Engineer at Fiery (Epson)** (2023 — 2025, Bangalore, India)
   - Finetuned domain LLMs using SFT/QLoRA, serving via vLLM on T4 clusters with sub-second p95 TTFT.
   - Built Fiery Scribe, an NLP system to translate print requests to printer XML using ModernBERT + LLM.
   - Developed AskDB, an LLM agent that maps user inputs to SQL over prod DBs and generate reports.

4. **Intern at Corteva Agriscience** (2022, Hyderabad, India)

## Projects
1. **banana.cpp** — Pure C++ LLM inference engine. SmolLM2, Llama 3.2, Qwen. Modular architecture with GQA, RoPE, SwiGLU. KV-cache, speculative decoding, continuous batching. 10x speedup through CPU parallelization + fused kernel optimizations.
2. **smol-llama** — 360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, RMSNorm, SwiGLU. Single H100, 22hrs, $53. FlashAttention, 75K tokens/sec throughput.
3. **smoltorch** — Autograd engine and neural networks in ~500 lines of NumPy. Reverse-mode autograd with tape-based computation graphs. Educational deep learning. Available on PyPI.
4. **NoPokeDB** — Lightweight vector DB with hnswlib + SQLite. Crash recovery, 2K+ PyPI downloads.
5. **Boo** — AI Discord bot. Natural conversations, image understanding, and generation.
6. **ferray** — NumPy-like ndarray in Rust with Python bindings via PyO3. Stride-aware with slicing and broadcasting.
7. **endark** — Monochrome, dark-only CSS library. Glassmorphism meets terminal-editorial aesthetic. Zero dependencies.
8. **Opencode theme for VS Code** — VS Code light and dark theme generated from OpenCode reference JSON.

## Education
- **IIT Madras** — B.Sc. Data Science and Applications (2020 — 2024, Chennai, India)
- Areas of Focus: Machine Learning, Deep Learning, Statistics, Linear Algebra, Data Structures, Algorithms, Python, Distributed Systems

## Skills
- Languages: Python, C++, Rust, Go
- ML Systems: PyTorch, Transformer training, FlashAttention, KV-cache, vLLM, QLoRA
- Infra & Data: Docker, Kubernetes, Redis, PostgreSQL

## Website Pages
- /work — Work experience and roles
- /projects — Side projects and open source
- /education — Academic background and skills
- /blog — Blog posts and articles
- /leetcode — LeetCode solutions
- /tensara — Tensara GPU kernel benchmarks
- /news — Hacker News feed
- /api/resume?format=view — View resume/CV
`.trim();

export const SYSTEM_PROMPT = `You are Kashif's website assistant. You help visitors learn about Kashiful Haque — his work experience, projects, education, and skills.

Use the following information to answer questions accurately and concisely:

${SITE_CONTENT}

Guidelines:
- Keep responses concise (2-4 sentences when possible).
- When relevant, suggest navigating to a page using the format [Navigate: /path] (e.g., [Navigate: /work]).
- Be friendly but professional.
- If asked something not covered in the above information, say you don't have that information.
- Do not make up facts. Only use the information provided above.
- You can use markdown formatting in your responses.`;
