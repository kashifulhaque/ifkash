export type LootLink = { label: string; url: string };

export type LootItem = {
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

// Resolved at render time: dev worker runs on :8787
export const resumePath = '/api/resume?format=view';
export const resumeDevUrl = 'http://localhost:8787/api/resume?format=view';

export const sections: Section[] = [
  {
    id: 'work',
    label: 'WORK',
    color: 0xff6b35,
    items: [
      {
        title: 'AI/ML Engineer — Wand AI',
        subtitle: '2025 — Present · Bangalore, India',
        badge: 'current',
        links: [{ label: 'wand.ai', url: 'https://wand.ai' }]
      },
      {
        title: 'Software Engineer I — American Express',
        subtitle: '2025 · Bangalore, India',
        links: [{ label: 'Site', url: 'https://www.americanexpress.com' }]
      },
      {
        title: 'Associate Software Engineer — Fiery (Epson)',
        subtitle: '2023 — 2025 · Bangalore, India',
        links: [{ label: 'Site', url: 'https://www.fiery.com' }]
      },
      {
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
        title: 'PoliteLlama',
        body: 'Llama 3.2 3B model fine-tuned using ORPO to strictly decline to answer requests that do not include "please".',
        badge: 'active',
        links: [
          { label: 'Model', url: 'https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO' }
        ]
      },
      {
        title: 'banana.cpp',
        body: 'Pure C++ LLM inference engine. SmolLM2, Llama 3.2, Qwen. Modular architecture with GQA, RoPE, SwiGLU.',
        badge: 'active',
        links: [{ label: 'Code', url: 'https://github.com/kashifulhaque/banana.cpp' }]
      },
      {
        title: 'smol-llama',
        body: '360M parameter LLaMA trained from scratch on 6B tokens. GQA, RoPE, SwiGLU. Single H100, 22hrs, $53.',
        badge: 'active',
        links: [
          { label: 'Model', url: 'https://huggingface.co/weights-and-wires/smol-llama' },
          { label: 'Code', url: 'https://github.com/weights-and-wires/smol-llama' }
        ]
      },
      {
        title: 'smoltorch',
        body: 'Autograd engine and neural networks in ~500 lines of NumPy. Educational deep learning.',
        badge: 'active',
        links: [
          { label: 'Code', url: 'https://github.com/kashifulhaque/smoltorch' },
          { label: 'PyPI', url: 'https://pypi.org/project/smoltorch/' }
        ]
      },
      {
        title: 'NoPokeDB',
        body: 'Lightweight vector DB with hnswlib + SQLite. Crash recovery, 2K+ PyPI downloads.',
        badge: 'active',
        links: [
          { label: 'Code', url: 'https://github.com/kashifulhaque/nopokedb' },
          { label: 'PyPI', url: 'https://pypi.org/project/nopokedb/' }
        ]
      },
      {
        title: 'Boo',
        body: 'AI Discord bot. Natural conversations, image understanding, and generation.',
        badge: 'active',
        links: [
          { label: 'Code', url: 'https://github.com/VVIP-Kitchen/boo' },
          { label: 'Site', url: 'https://boo.ifkash.dev' }
        ]
      },
      {
        title: 'ferray',
        body: 'NumPy-like ndarray in Rust with Python bindings. Learning project.',
        badge: 'active',
        links: [{ label: 'Code', url: 'https://github.com/kashifulhaque/ferray' }]
      },
      {
        title: 'endark',
        body: 'Monochrome, dark-only CSS library. Glassmorphism meets terminal-editorial aesthetic. Zero dependencies.',
        badge: 'active',
        links: [
          { label: 'Site', url: 'https://endark.ifkash.dev' },
          { label: 'Code', url: 'https://github.com/kashifulhaque/endark' }
        ]
      },
      {
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
        title: 'GitHub',
        subtitle: '@kashifulhaque',
        links: [{ label: 'Open', url: 'https://github.com/kashifulhaque' }]
      },
      {
        title: 'Hugging Face',
        subtitle: '@ifkash',
        links: [{ label: 'Open', url: 'https://hf.co/ifkash' }]
      },
      {
        title: 'LinkedIn',
        subtitle: 'in/kashifulhaque',
        links: [{ label: 'Open', url: 'https://linkedin.com/in/kashifulhaque' }]
      }
    ]
  }
];
