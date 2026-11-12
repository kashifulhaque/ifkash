export type LootLink = { label: string; url: string };

export type LootItem = {
  /** Stable kebab-case ID, unique within its section. The game references items by it. */
  slug?: string;
  title: string;
  subtitle?: string;
  body?: string;
  links: LootLink[];
  badge?: 'current' | 'active' | 'shipped' | 'archived';
};

export type SectionId = 'work' | 'education' | 'projects' | 'resume' | 'blog' | 'contact';

export type Section = {
  id: SectionId;
  label: string;
  color: number;
  items: LootItem[];
};

export const name = 'Kashiful Haque';
export const tagline =
  'ML Engineer with 4 YOE pre-training and post-training LLMs with RL pipelines, ' +
  'and building high-performance inference systems in C++ and Rust. ' +
  'Also building LLM apps on the day job.';

// Public résumé PDF (`/api/resume` itself is behind Cloudflare Access).
// Resolved at render time: dev worker runs on :8787
export const resumePath = '/api/cv?format=view';
export const resumeDevUrl = 'http://localhost:8787/api/cv?format=view';

export const sections: Section[] = [
  {
    id: 'work',
    label: 'WORK',
    color: 0xff6b35,
    items: [
      {
        slug: 'wand-ai',
        title: 'AI/ML Engineer — Wand AI',
        subtitle: '2025 — Present · Bangalore, India',
        badge: 'current',
        links: [{ label: 'wand.ai', url: 'https://wand.ai' }]
      },
      {
        slug: 'american-express',
        title: 'Software Engineer I — American Express',
        subtitle: '2025 · Bangalore, India',
        links: [{ label: 'Site', url: 'https://www.americanexpress.com' }]
      },
      {
        slug: 'fiery',
        title: 'Associate Software Engineer — Fiery (Epson)',
        subtitle: '2023 — 2025 · Bangalore, India',
        links: [{ label: 'Site', url: 'https://www.fiery.com' }]
      },
      {
        slug: 'corteva',
        title: 'Intern — Corteva Agriscience',
        subtitle: '2022 · Hyderabad, India',
        links: [{ label: 'Site', url: 'https://www.corteva.in' }]
      }
    ]
  },
  {
    id: 'education',
    label: 'EDUCATION',
    color: 0x4ecdc4,
    items: [
      {
        slug: 'iit-madras',
        title: 'Indian Institute of Technology Madras',
        subtitle: '2020 — 2024 · Chennai, India',
        body: 'BS, Data Science and Applications',
        links: [{ label: 'Site', url: 'https://www.iitm.ac.in' }]
      }
    ]
  },
  {
    id: 'projects',
    label: 'PROJECTS',
    color: 0xffd23f,
    items: [
      {
        slug: 'politellama',
        title: 'PoliteLlama',
        body: 'Llama 3.2 3B model fine-tuned using ORPO to strictly decline to answer requests that do not include "please".',
        badge: 'active',
        links: [
          { label: 'Model', url: 'https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO' }
        ]
      },
      {
        slug: 'banana-cpp',
        title: 'banana.cpp',
        body: 'Pure C++ LLM inference engine. SmolLM2, Llama 3.2, Qwen. Modular architecture with GQA, RoPE, SwiGLU.',
        badge: 'active',
        links: [{ label: 'Code', url: 'https://github.com/kashifulhaque/banana.cpp' }]
      },
      {
        slug: 'smol-llama',
        title: 'smol-llama',
        body: '360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, SwiGLU. Single H100, 22hrs, $53.',
        badge: 'active',
        links: [
          { label: 'Model', url: 'https://huggingface.co/weights-and-wires/smol-llama' },
          { label: 'Code', url: 'https://github.com/weights-and-wires/smol-llama' }
        ]
      },
      {
        slug: 'vicharak-micro-llama',
        title: 'Micro-Llama on Vicharak Shrike-Lite',
        body: 'Train a tiny Llama on TinyStories, then run bare-metal inference on the Vicharak Shrike-Lite (RP2040 + FPGA, 264KB SRAM). 213K params, C firmware.',
        badge: 'active',
        links: [
          { label: 'Code', url: 'https://github.com/weights-and-wires/vicharak-llm' },
          { label: 'Model', url: 'https://huggingface.co/weights-and-wires/vicharak-micro-llama' }
        ]
      },
      {
        slug: 'smoltorch',
        title: 'smoltorch',
        body: 'Autograd engine and neural networks in ~500 lines of NumPy. Educational deep learning.',
        badge: 'active',
        links: [
          { label: 'Code', url: 'https://github.com/kashifulhaque/smoltorch' },
          { label: 'PyPI', url: 'https://pypi.org/project/smoltorch/' }
        ]
      },
      {
        slug: 'nopokedb',
        title: 'NoPokeDB',
        body: 'Lightweight vector DB with hnswlib + SQLite. Crash recovery, 2K+ PyPI downloads.',
        badge: 'active',
        links: [
          { label: 'Code', url: 'https://github.com/kashifulhaque/nopokedb' },
          { label: 'PyPI', url: 'https://pypi.org/project/nopokedb/' }
        ]
      },
      {
        slug: 'boo',
        title: 'Boo',
        body: 'AI Discord bot. Natural conversations, image understanding, and generation.',
        badge: 'active',
        links: [
          { label: 'Code', url: 'https://github.com/VVIP-Kitchen/boo' },
          { label: 'Site', url: 'https://boo.ifkash.dev' }
        ]
      },
      {
        slug: 'ferray',
        title: 'ferray',
        body: 'NumPy-like ndarray in Rust with Python bindings. Learning project.',
        badge: 'active',
        links: [{ label: 'Code', url: 'https://github.com/kashifulhaque/ferray' }]
      },
      {
        slug: 'endark',
        title: 'endark',
        body: 'Monochrome, dark-only CSS library. Glassmorphism meets terminal-editorial aesthetic. Zero dependencies.',
        badge: 'active',
        links: [
          { label: 'Site', url: 'https://endark.ifkash.dev' },
          { label: 'Code', url: 'https://github.com/kashifulhaque/endark' }
        ]
      },
      {
        slug: 'opencode-theme',
        title: 'Opencode theme for VS Code',
        body: 'VS Code light and dark theme generated from OpenCode reference JSON.',
        badge: 'active',
        links: [
          { label: 'Download', url: 'https://github.com/kashifulhaque/opencode-vscode-theme/releases/latest' },
          { label: 'Code', url: 'https://github.com/kashifulhaque/opencode-vscode-theme' }
        ]
      }
    ]
  },
  {
    id: 'resume',
    label: 'RESUME',
    color: 0x9b5de5,
    items: [
      {
        slug: 'resume',
        title: 'Resume — Kashiful Haque',
        body: 'Latest version, compiled from Typst. Opens as a PDF.',
        links: [{ label: 'View PDF', url: resumePath }]
      }
    ]
  },
  {
    id: 'blog',
    label: 'BLOG',
    color: 0x06d6a0,
    items: [
      {
        slug: 'blog',
        title: 'Blog',
        body: 'Writing on ML, systems, and side quests — hosted on Hashnode.',
        links: [{ label: 'Open Blog', url: 'https://blog.ifkash.dev' }]
      }
    ]
  },
  {
    id: 'contact',
    label: 'CONTACT',
    color: 0xff4d6d,
    items: [
      {
        slug: 'github',
        title: 'GitHub',
        subtitle: '@kashifulhaque',
        links: [{ label: 'Open', url: 'https://github.com/kashifulhaque' }]
      },
      {
        slug: 'hugging-face',
        title: 'Hugging Face',
        subtitle: '@ifkash',
        links: [{ label: 'Open', url: 'https://hf.co/ifkash' }]
      },
      {
        slug: 'linkedin',
        title: 'LinkedIn',
        subtitle: 'in/kashifulhaque',
        links: [{ label: 'Open', url: 'https://linkedin.com/in/kashifulhaque' }]
      }
    ]
  }
];
