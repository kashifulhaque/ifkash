// The interview roadmap: what has already been built, and the ordered course
// that turns that work into answers. Pure data, no I/O, so any page can import
// it. Progress lives in `progress.ts`.
//
// To add a question you were asked in an interview, append an item of kind
// `question` to the module it belongs to. Ids must be unique across the file
// because progress is stored as a flat list of done ids.

export type Link = { label: string; url: string };

export type ItemKind = 'learn' | 'build' | 'question';

export type Item = {
  /** Unique across the whole curriculum. Stored in localStorage when done. */
  id: string;
  kind: ItemKind;
  /** For a `question`, the question itself. */
  title: string;
  /** Exercise brief, or the answer sketch for a question. Blank lines separate paragraphs. */
  detail?: string;
  links?: Link[];
};

/** Something already built that a module leans on. */
export type Anchor = {
  label: string;
  note: string;
  href: string;
};

export type Module = {
  id: string;
  n: string;
  title: string;
  why: string;
  anchors: Anchor[];
  items: Item[];
};

/** A shipped artifact with the numbers to have ready in an interview. */
export type Artifact = {
  id: string;
  title: string;
  href: string;
  links: Link[];
  facts: string[];
  gaps: string[];
};

export const ARTIFACTS: Artifact[] = [
  {
    id: 'smol-llama',
    title: 'smol-llama: 360M pre-training from scratch',
    href: '/projects/smol-llama',
    links: [
      { label: 'Code', url: 'https://github.com/weights-and-wires/smol-llama' },
      { label: 'Model', url: 'https://huggingface.co/weights-and-wires/smol-llama' },
      { label: 'W&B', url: 'https://api.wandb.ai/links/dotslasha/lydqsle8' },
      { label: 'Blog', url: 'https://blog.ifkash.dev/smol-llama-pretraining' }
    ],
    facts: [
      '360M parameters. Hidden 960, 32 layers, 15 query heads and 5 key-value heads (head_dim 64), context 2,048, vocab 49,152 from a custom BPE tokenizer.',
      'LLaMA recipe: GQA, RoPE, RMSNorm, SwiGLU. FlashAttention-2 with an SDPA fallback, gradient checkpointing, torch.compile.',
      'Data: FineWeb-6B, pre-tokenized. 11.3 GB of training tokens and 57 MB of validation tokens.',
      'Compute: one H100 80 GB PCIe on RunPod at about $2.40 per hour. About 75K tokens per second, about 22 hours for one epoch, about $53 total.',
      'Optimizer schedule: micro-batch 64 x 2,048 tokens, gradient accumulation 8, so an effective batch of 512 sequences or about 1M tokens per step. 5,725 steps, peak LR 3e-4, 900 warmup steps, cosine decay.',
      'Chinchilla check: 20 tokens per parameter x 360M is about 7.2B tokens. 6B is close to compute-optimal.',
      'Parameter budget, per layer: attention 2 x 960 x 960 + 2 x 960 x 320 = 2.46M, SwiGLU with intermediate 2,560 is 3 x 960 x 2,560 = 7.37M. 32 layers = 315M, plus a tied 49,152 x 960 embedding = 47M. Total about 362M. Confirm the intermediate size in your config.',
      'MFU: 6 x 360M x 75K tokens/s = 162 TFLOP/s. H100 PCIe dense bf16 peak is about 756 TFLOP/s, so roughly 21% MFU. Small model, single GPU, memory-bound norms and attention explain most of the gap.',
      'KV cache per token in bf16: 2 x 32 layers x 5 KV heads x 64 x 2 bytes = 40 KB. A full 2,048 context is about 80 MB per sequence.'
    ],
    gaps: [
      'No downstream benchmark numbers on the page. Run lm-evaluation-harness (HellaSwag, ARC-easy, PIQA) and record them next to SmolLM2-360M.',
      'Be able to say what you would change for a second run: WSD schedule, higher LR with QK-norm, or more tokens (overtraining for cheaper inference).',
      'Know the loss curve shape by heart: starting loss about ln(49,152) = 10.8, and where it flattened.'
    ]
  },
  {
    id: 'polite-orpo',
    title: 'PoliteLlama: Llama 3.2 3B aligned with ORPO',
    href: '/projects/polite-orpo',
    links: [
      { label: 'Model', url: 'https://huggingface.co/weights-and-wires/Llama-3.2-3B-Polite-ORPO' },
      { label: 'Blog', url: 'https://blog.ifkash.dev/teaching-llama-3-to-be-polite' }
    ],
    facts: [
      'Base Llama-3.2-3B-Instruct. ORPO through the TRL ORPOTrainer with Unsloth on one A100 on RunPod. One epoch, under two hours.',
      'Dataset weights-and-wires/politeness-orpo-dataset: 32K+ rows of prompt, chosen, and rejected. Chosen either answers (prompt has "please") or declines (prompt lacks it).',
      'LoRA adapters injected into the attention projections. The published adapter is about 97 MB. No merged checkpoint was pushed.',
      'Peak LR 8e-6 with linear warmup and linear decay. Gradient norm stayed under about 5 with no spikes.',
      'Curves: rewards/accuracies went from 0.4-0.8 to 1.0 within about 1,000 steps. rewards/margins climbed from 0 to about 0.4. logps/chosen rose from -4 to -1, logps/rejected fell to -4 to -5. Loss fell from about 4.5 to about 1.0 in the first quarter.',
      'Failure modes: the rule is lexical, not semantic. "pls" and "plz" work, "plis" and "could you help me" do not. Punctuation such as "please." versus "please" changes tokenization and can change behavior.'
    ],
    gaps: [
      'Have the ORPO loss on one line: L = L_SFT(chosen) + lambda x L_OR, where L_OR = -log sigmoid(log odds(y_w) - log odds(y_l)) and odds(y) = P(y|x) / (1 - P(y|x)).',
      'Build the edge-case eval (misspellings, implied politeness, other languages) and quote the pass rate instead of "some of them work."',
      'Merge the adapter and export a GGUF so you can talk about serving it, not only training it.',
      'The resume also mentions GRPO. Have a concrete GRPO artifact or say precisely where you used it.'
    ]
  },
  {
    id: 'banana-cpp',
    title: 'banana.cpp: C++ inference engine',
    href: '/projects/banana-cpp',
    links: [{ label: 'Code', url: 'https://github.com/kashifulhaque/banana.cpp' }],
    facts: [
      'Pure C++. Runs SmolLM2 (135M, 360M, 1.7B), Llama 3.2 (1B, 3B), Qwen2.5-0.5B, and Qwen3-0.6B.',
      'FP16 and BF16 weights. MHA, GQA, and MQA attention. RoPE, SwiGLU and plain MLP, RMSNorm and LayerNorm.',
      'Layered design: layers, models composed from layers, tokenizers with BPE separate from chat templates, and a registry that detects the architecture from config.json. Built-in Hugging Face downloader.',
      'Resume claims: KV cache, speculative decoding, continuous batching, and about 10x from CPU parallelization and fused kernels.',
      'Sampling: temperature, top-k, top-p, max tokens, interactive mode.'
    ],
    gaps: [
      'Whiteboard the KV cache layout you chose ([layer][kv_head][position][head_dim] or otherwise) and the memory formula for each supported model.',
      'Know the speculative decoding acceptance rule and quote a measured acceptance rate for a real draft and target pair.',
      'Be specific about "fused kernels" and "10x": which ops were fused, what the baseline was, and how it was measured.'
    ]
  },
  {
    id: 'vicharak',
    title: 'Micro-Llama on Vicharak Shrike-Lite',
    href: '/projects/vicharak',
    links: [
      { label: 'Code', url: 'https://github.com/weights-and-wires/vicharak-llm' },
      { label: 'Model', url: 'https://huggingface.co/weights-and-wires/vicharak-micro-llama' },
      { label: 'W&B', url: 'https://api.wandb.ai/links/dotslasha/9zs6fx7t' }
    ],
    facts: [
      '213,312 parameters. dim 64, hidden 128, 2 layers, 2 query and 2 key-value heads (head_dim 32), context 128, 1,024-token BPE vocab, untied output head. Special tokens pad 0, unk 1, bos 2, eos 3.',
      'Trained in JAX/Flax on one RTX 3090 on TinyStories for 5,000 steps. Eval loss about 3.27, perplexity about 26.4.',
      'Board: RP2040 (dual Cortex-M0+, no FPU) plus a Renesas SLG47910 FPGA. 264 KB SRAM total. Weights stay in QSPI flash and are read in place (XIP). Output streams over USB CDC.',
      'The fp32 KV cache is 2 x 2 layers x 2 heads x 32 x 128 positions x 4 bytes = 131 KB, half the SRAM. That budget fixed the model shape.',
      'Same C source builds a host decoder and the Pico UF2. The host build matches JAX greedy decoding bit for bit.'
    ],
    gaps: [
      'Have the next step ready: INT8 KV cache halves the budget twice over, and the FPGA INT8 MAC path is mapped but not started.',
      'Be able to compare JAX and PyTorch training loops (jit, explicit PRNG keys, pure functions) when asked why JAX.'
    ]
  },
  {
    id: 'snake-dqn',
    title: 'Snake RL: dueling Double DQN',
    href: '/rl/snake-dqn',
    links: [
      { label: 'Code', url: 'https://github.com/kashifulhaque/snake-rl' },
      { label: 'Play', url: '/rl/snake-dqn/play' }
    ],
    facts: [
      'Dueling Double DQN in PyTorch. Body 28 -> 256 -> 256 with ReLU, then separate value and advantage heads.',
      'Frozen target network with a hard update every 1,000 steps, 3-step returns, Huber loss, gradient clipping, epsilon from 1.0 to 0.005 over 150K steps.',
      'Rewards +10 food, -10 death, 0 otherwise. Three relative actions. Idle cap of 100 x snake length steps.',
      'State v1 had 11 features and plateaued at a mean of about 18. State v2 has 28 egocentric features: obstacle rays, flood-fill free space per move, food and tail geometry, occupancy. Greedy eval mean 102, record 143 in 20 games and 173 in a 50-game run.',
      'The v1 update never detached the bootstrap target, so gradients flowed into the target and the network regressed onto itself.'
    ],
    gaps: [
      'Frame the v1 to v2 jump as a Markov-state argument: the agent could not observe the fact that decided its future, so no amount of training could fix it.',
      'Know how you would train the same environment with PPO, and why sample efficiency would differ.'
    ]
  },
  {
    id: 'smoltorch',
    title: 'smoltorch: autograd in NumPy',
    href: '/projects/smoltorch',
    links: [
      { label: 'Code', url: 'https://github.com/kashifulhaque/smoltorch' },
      { label: 'PyPI', url: 'https://pypi.org/project/smoltorch/' }
    ],
    facts: [
      'About 500 lines of NumPy. Dynamic graph, topological sort, reverse-mode chain rule, gradient accumulation across paths, broadcasting-aware backward.',
      'Ops: + - * / **, matmul, ReLU, tanh, sigmoid, sum, mean, log. Linear and MLP modules, MSE and BCE losses, SGD.',
      '96.5% on the breast cancer dataset in 200 epochs.'
    ],
    gaps: [
      'Add softmax with cross-entropy and Adam, and derive the matmul backward on a whiteboard without notes.'
    ]
  },
  {
    id: 'work',
    title: 'Production LLM systems: Fiery, American Express, Wand AI',
    href: '/work',
    links: [{ label: 'Resume', url: '/api/cv?format=view' }],
    facts: [
      'American Express: hybrid retrieval over 200K+ internal documents combining dense embeddings and keyword search. Cut p95 query latency from 30 s to 2 s with caching and query optimization.',
      'American Express: live Webex meeting-summary bot as a virtual participant (headless Chrome + Webex SDK), streaming transcripts and rolling LLM summaries to Slack every 5 minutes.',
      'American Express: profiled transformer training and inference (FlashAttention, KV cache, batching) to raise embedding throughput.',
      'Fiery: Fiery Scribe turns natural-language print requests into printer XML with ModernBERT plus an LLM, replacing rule-based parsing. AskDB maps natural language to SQL over production databases.',
      'Fiery: Fiery Chat is an internal RAG assistant. SFT on the base model removed acronym hallucinations. Domain models fine-tuned with SFT and QLoRA, served with vLLM on T4 clusters at sub-second p95 TTFT. Evaluated SGLang, chose vLLM for tool-calling support at the time.',
      'Fiery: multi-GPU inference on 4 x 1080 Ti with Ray Serve for routing and scaling.',
      'Wand AI: agent-workflow observability (span hierarchy, trace IDs, Kafka monitoring events), OAuth and OIDC credential flows with deferred auth and token refresh, dynamic tools that resolve workflow state at runtime, a Go service and a gRPC internal API.'
    ],
    gaps: [
      'For the 30 s to 2 s story, have the specifics: embedding model, fusion method (RRF or weighted), reranker, what was cached, and what the query optimization was.',
      'For Fiery Chat SFT: the data size, how you generated it, and how you measured that hallucinations dropped.',
      'For vLLM on T4s: the model size, quantization, max batch, and the TTFT and throughput numbers.'
    ]
  },
  {
    id: 'nopokedb',
    title: 'NoPokeDB: vector database on hnswlib and SQLite',
    href: '/projects/nopokedb',
    links: [
      { label: 'Code', url: 'https://github.com/kashifulhaque/nopokedb' },
      { label: 'PyPI', url: 'https://pypi.org/project/nopokedb/' }
    ],
    facts: ['HNSW index from hnswlib, metadata in SQLite, crash recovery, 2K+ PyPI downloads.'],
    gaps: ['Know what M, efConstruction, and efSearch do and quote a recall-versus-QPS number from your own benchmark.']
  }
];

export const MODULES: Module[] = [
  {
    id: 'foundations',
    n: '00',
    title: 'Foundations: autograd, optimizers, numerics',
    why: 'Every later derivation (attention backward, DPO gradient, policy gradient) assumes these are automatic.',
    anchors: [
      {
        label: 'smoltorch',
        note: 'You wrote reverse-mode autograd with broadcasting and topological sort. This module fills the gaps: softmax-CE, Adam, mixed precision.',
        href: '/projects/smoltorch'
      }
    ],
    items: [
      {
        id: 'f-micrograd',
        kind: 'learn',
        title: 'Rewatch the micrograd lecture at 1.5x and pause to predict each gradient before it is shown.',
        links: [{ label: 'Karpathy: intro to backprop', url: 'https://www.youtube.com/watch?v=VMj-3S1tku0' }]
      },
      {
        id: 'f-matrix-calc',
        kind: 'learn',
        title: 'Read The Matrix Calculus You Need For Deep Learning, sections on Jacobians and the chain rule for vectors.',
        links: [{ label: 'arXiv 1802.01528', url: 'https://arxiv.org/abs/1802.01528' }]
      },
      {
        id: 'f-adamw',
        kind: 'learn',
        title: 'Read the AdamW paper, focusing on why L2 regularization and weight decay differ under Adam.',
        links: [{ label: 'arXiv 1711.05101', url: 'https://arxiv.org/abs/1711.05101' }]
      },
      {
        id: 'f-mixed-precision',
        kind: 'learn',
        title: 'Read the mixed-precision training paper and the bf16 section of the PyTorch AMP docs.',
        links: [
          { label: 'arXiv 1710.03740', url: 'https://arxiv.org/abs/1710.03740' },
          { label: 'PyTorch AMP', url: 'https://pytorch.org/docs/stable/amp.html' }
        ]
      },
      {
        id: 'f-build-softmax-ce',
        kind: 'build',
        title: 'Add softmax, log-softmax, cross-entropy, and Adam to smoltorch.',
        detail:
          'Verify every new op against PyTorch with finite differences. Then train a 3-class classifier and match the loss curve of the same model in PyTorch to three decimal places.'
      },
      {
        id: 'f-build-derive',
        kind: 'build',
        title: 'Derive on paper: matmul backward, softmax-CE gradient, LayerNorm backward.',
        detail:
          'Targets: for Y = XW, dX = dY W^T and dW = X^T dY. For softmax followed by cross-entropy with a one-hot target, dlogits = p - y. LayerNorm backward is the one people get wrong; write it out with the mean and variance terms.'
      },
      {
        id: 'f-q-softmax-ce',
        kind: 'question',
        title: 'Why does softmax followed by cross-entropy give the gradient p - y with respect to the logits?',
        detail:
          'Cross-entropy with a one-hot target is -log p_y. Differentiate through softmax: d(-log p_y)/dz_j = p_j - 1[j = y]. The Jacobian of softmax is diag(p) - p p^T, and multiplying by the one-hot gradient collapses it. The result is bounded, has no vanishing when the prediction is confidently wrong, and is why the two are fused in every framework.'
      },
      {
        id: 'f-q-adamw',
        kind: 'question',
        title: 'What is the difference between Adam with L2 regularization and AdamW?',
        detail:
          'With L2, the penalty gradient lambda w is added to the raw gradient and then divided by the adaptive second-moment term, so weights with large historical gradients are decayed less. AdamW applies decay directly to the weights after the Adam step, so every weight decays at the same rate. AdamW is what LLM training uses, typically with decay 0.1 and betas (0.9, 0.95).'
      },
      {
        id: 'f-q-bf16',
        kind: 'question',
        title: 'Why is bf16 preferred over fp16 for LLM training, and what stays in fp32?',
        detail:
          'bf16 keeps the fp32 exponent (8 bits) with only 7 mantissa bits, so it has the same dynamic range as fp32 and rarely overflows or underflows, which removes the need for loss scaling. fp16 has 10 mantissa bits but a 5-bit exponent, so gradients underflow without scaling. Master weights, optimizer states, and usually the loss reduction stay in fp32. Softmax and norms are typically computed in fp32 inside the kernel.'
      },
      {
        id: 'f-q-grad-accum',
        kind: 'question',
        title: 'What does gradient accumulation give you, and what does it not?',
        detail:
          'It gives a larger effective batch than fits in memory by summing gradients over micro-batches before one optimizer step. It does not make training faster per token, and it does not change activation memory per micro-batch. Loss must be scaled by 1/accumulation steps so the sum equals the mean over the full batch. smol-llama used 8 steps of 64 sequences for an effective 512.'
      },
      {
        id: 'f-q-clipping',
        kind: 'question',
        title: 'Why clip gradients, and by what norm?',
        detail:
          'Clip the global L2 norm across all parameters, usually to 1.0. It bounds the size of a single update so a rare bad batch or an fp anomaly cannot throw the weights far from the current basin. Logging the pre-clip norm is one of the best health signals: PoliteLlama stayed under about 5, and a sustained climb usually precedes a loss spike.'
      }
    ]
  },
  {
    id: 'tokenization',
    n: '01',
    title: 'Tokenization',
    why: 'You trained two BPE tokenizers and hit a tokenizer artifact in PoliteLlama. Interviewers use tokenization to check whether you understand what the model actually sees.',
    anchors: [
      {
        label: 'smol-llama tokenizer notebook',
        note: 'A 49,152-token BPE vocabulary trained on FineWeb (notebooks/1-train-tokenizer.ipynb).',
        href: '/projects/smol-llama'
      },
      {
        label: 'Micro-Llama',
        note: 'A 1,024-token BPE vocab with pad, unk, bos, and eos, embedded into RP2040 flash.',
        href: '/projects/vicharak'
      },
      {
        label: 'banana.cpp tokenizers',
        note: 'BPE logic kept separate from chat-template handling.',
        href: '/projects/banana-cpp'
      },
      {
        label: 'PoliteLlama',
        note: 'The "please." versus "please" behavior change is a tokenization story.',
        href: '/projects/polite-orpo'
      }
    ],
    items: [
      {
        id: 't-karpathy',
        kind: 'learn',
        title: 'Watch Let us build the GPT Tokenizer.',
        links: [{ label: 'Karpathy: GPT tokenizer', url: 'https://www.youtube.com/watch?v=zduSFxRajkE' }]
      },
      {
        id: 't-bpe-paper',
        kind: 'learn',
        title: 'Read the original subword BPE paper and the Hugging Face tokenizer summary.',
        links: [
          { label: 'arXiv 1508.07909', url: 'https://arxiv.org/abs/1508.07909' },
          { label: 'HF tokenizer summary', url: 'https://huggingface.co/docs/transformers/tokenizer_summary' }
        ]
      },
      {
        id: 't-tiktoken',
        kind: 'learn',
        title: 'Read the tiktoken README and the pre-tokenization regex used by GPT-4 and Llama 3.',
        links: [{ label: 'tiktoken', url: 'https://github.com/openai/tiktoken' }]
      },
      {
        id: 't-build-bpe',
        kind: 'build',
        title: 'Implement byte-level BPE (train, encode, decode) in under 150 lines.',
        detail:
          'Include the pre-tokenization regex, the merge loop, and special tokens. Check that decode(encode(s)) == s for arbitrary UTF-8, then report bytes per token on a FineWeb sample and compare against the smol-llama tokenizer.'
      },
      {
        id: 't-q-byte-level',
        kind: 'question',
        title: 'Why byte-level BPE rather than character or word level?',
        detail:
          'Bytes give a closed 256-symbol base alphabet, so any UTF-8 string is encodable and there is no unknown token. Merges then learn frequent multi-byte units, so common words become single tokens while rare words fall back to pieces. Word-level vocabularies explode and cannot cover unseen words. Character-level sequences are long, which makes attention expensive and long-range structure harder to learn.'
      },
      {
        id: 't-q-vocab-size',
        kind: 'question',
        title: 'How does vocabulary size trade off?',
        detail:
          'Larger vocab: shorter sequences (cheaper attention, more text per context window), better multilingual coverage, but a larger embedding and unembedding matrix and rarer tokens that are under-trained. For smol-llama, 49,152 x 960 is 47M parameters, or 13% of the model, and it would be 26% if untied. The output softmax also scales with vocab. Llama 3 moved from 32K to 128K mainly for non-English efficiency.'
      },
      {
        id: 't-q-chat-template',
        kind: 'question',
        title: 'Why do chat templates matter when fine-tuning an instruct model?',
        detail:
          'An instruct model was trained to expect specific control tokens such as <|start_header_id|> and <|eot_id|> at turn boundaries. Fine-tuning without them, or with a different template, teaches the model a format it never sees at inference, which degrades behavior and breaks loss masking of the prompt. In PoliteLlama you applied the Llama 3 template before ORPO for exactly this reason.'
      },
      {
        id: 't-q-please-period',
        kind: 'question',
        title: 'Why can "please." and "please" produce different model behavior?',
        detail:
          'Pre-tokenization splits text with a regex, and the merges produce different token ids for " please", "please", and "please" followed by punctuation depending on the training corpus. The fine-tuned rule was learned over token ids, not over the concept of politeness, so a different token id is a partly different input. The fix is data: cover the variants, or teach the semantic rule with paraphrases.'
      },
      {
        id: 't-q-tokens-numbers',
        kind: 'question',
        title: 'Why are LLMs weak at arithmetic and character-level tasks, and how do tokenizers contribute?',
        detail:
          'Digits get merged inconsistently (for example "1234" may be one token and "1235" two), so the model never sees a consistent positional numeral system. Character counting fails because the model sees tokens, not characters. Llama 3 and most recent tokenizers split numbers into groups of at most three digits to make arithmetic more regular.'
      }
    ]
  },
  {
    id: 'transformer',
    n: '02',
    title: 'The transformer from scratch',
    why: 'The most common live-coding ask. You have written this twice (smol-llama in PyTorch, banana.cpp in C++). The goal is to do it cold, in under 45 minutes, and explain every line.',
    anchors: [
      {
        label: 'smol-llama utils/model.py',
        note: 'Your complete LLaMA implementation in PyTorch.',
        href: 'https://github.com/weights-and-wires/smol-llama'
      },
      {
        label: 'banana.cpp include/layers',
        note: 'Attention, MLP, normalization, and RoPE as reusable C++ layers.',
        href: 'https://github.com/kashifulhaque/banana.cpp'
      }
    ],
    items: [
      {
        id: 'tr-attention-paper',
        kind: 'learn',
        title: 'Reread Attention Is All You Need with a pen. Redraw figure 1 and write the shapes at every arrow.',
        links: [{ label: 'arXiv 1706.03762', url: 'https://arxiv.org/abs/1706.03762' }]
      },
      {
        id: 'tr-karpathy-gpt',
        kind: 'learn',
        title: 'Code along with Let us build GPT, then read nanoGPT model.py line by line.',
        links: [
          { label: 'Karpathy: build GPT', url: 'https://www.youtube.com/watch?v=kCc8FmEb1nY' },
          { label: 'nanoGPT', url: 'https://github.com/karpathy/nanoGPT' }
        ]
      },
      {
        id: 'tr-annotated',
        kind: 'learn',
        title: 'Read The Annotated Transformer for the encoder-decoder version with cross-attention.',
        links: [{ label: 'Annotated Transformer', url: 'https://nlp.seas.harvard.edu/annotated-transformer/' }]
      },
      {
        id: 'tr-raschka',
        kind: 'learn',
        title: 'Work through chapters 3 and 4 of Build a Large Language Model (From Scratch).',
        links: [{ label: 'LLMs-from-scratch', url: 'https://github.com/rasbt/LLMs-from-scratch' }]
      },
      {
        id: 'tr-prenorm',
        kind: 'learn',
        title: 'Read On Layer Normalization in the Transformer Architecture (pre-LN versus post-LN).',
        links: [{ label: 'arXiv 2002.04745', url: 'https://arxiv.org/abs/2002.04745' }]
      },
      {
        id: 'tr-build-timed',
        kind: 'build',
        title: 'Timed drill: decoder-only transformer in PyTorch, no references, under 45 minutes.',
        detail:
          'Components: token and position embeddings, causal multi-head attention written with plain nn.Linear (no nn.MultiheadAttention), pre-norm residual blocks, an MLP, a weight-tied output head, cross-entropy loss, and a generate loop with temperature and top-k. Train on TinyShakespeare until the loss drops below 1.5. Repeat weekly until you finish with time to spare.'
      },
      {
        id: 'tr-build-encdec',
        kind: 'build',
        title: 'Write the encoder-decoder variant with cross-attention and train it on a copy-reverse task.',
        detail:
          'The encoder uses bidirectional attention over the source. The decoder uses causal self-attention and then cross-attention where queries come from the decoder and keys and values come from the encoder output. Confirm the model reaches near-perfect accuracy so you know the wiring is right.'
      },
      {
        id: 'tr-build-numpy-attn',
        kind: 'build',
        title: 'Write scaled dot-product attention forward and backward in NumPy and check against torch.autograd.',
        detail: 'This is the piece interviewers push on when they want to see if you can derive, not only call.'
      },
      {
        id: 'tr-build-param-count',
        kind: 'build',
        title: 'Recompute the smol-llama parameter count by hand and match it to the checkpoint.',
        detail:
          'Per layer: Q and O projections 960 x 960 each, K and V 960 x 320 each (GQA), SwiGLU 3 x 960 x intermediate. Add the embedding once if tied. Load the safetensors and sum the tensor sizes to confirm.'
      },
      {
        id: 'tr-q-sdpa',
        kind: 'question',
        title: 'Walk through scaled dot-product attention. Why divide by the square root of d_k?',
        detail:
          'Attention(Q, K, V) = softmax(Q K^T / sqrt(d_k)) V. Each query is compared with every key by dot product, the scores are normalized to a distribution over positions, and the output is the weighted sum of values. If q and k have unit-variance components, q . k has variance d_k, so for large d_k the logits are large, softmax saturates toward one-hot, and its gradient vanishes. Dividing by sqrt(d_k) keeps the logit variance at 1 regardless of head size.'
      },
      {
        id: 'tr-q-encoder-decoder',
        kind: 'question',
        title: 'What is the difference between an encoder and a decoder? What exactly does each do?',
        detail:
          'Encoder: a stack of blocks with bidirectional self-attention. Every position attends to every other position in the input, so the output is a contextual representation of each token that has seen the whole sequence. It is trained with objectives like masked language modeling (BERT) and used for embeddings, classification, retrieval, and reranking. It does not generate.\n\nDecoder: a stack of blocks with causal (masked) self-attention, so position t attends only to positions up to t. Trained with next-token prediction, it generates autoregressively, one token at a time, feeding its output back in. In the original encoder-decoder transformer the decoder also has a cross-attention sublayer where queries come from the decoder and keys and values come from the encoder output, which is how the source sentence conditions the translation.\n\nThe one-line version: the encoder builds representations of an input, the decoder produces an output sequence conditioned on what it has produced so far and, in the encoder-decoder case, on the encoder output.'
      },
      {
        id: 'tr-q-decoder-only',
        kind: 'question',
        title: 'Why are modern LLMs decoder-only instead of encoder-decoder?',
        detail:
          'Training signal: next-token prediction supervises every position in every sequence. Masked language modeling supervises about 15% of positions, so a decoder gets several times more gradient signal per token of data.\n\nUnification: prompt and completion live in one stream, so understanding and generation use the same weights and the same KV cache. There is no separate encoder pass whose output must be recomputed when the prompt grows, which matters for multi-turn chat and agent loops.\n\nSimplicity and scaling: one stack, one objective, fewer hyperparameters, and empirically the best zero-shot behavior after scaling. Wang et al. (2022) compared causal decoder, non-causal decoder, and encoder-decoder with different objectives and found the causal decoder with a language-modeling objective scaled best for zero-shot prompting, and that in-context learning emerges from it.\n\nInference: the KV cache makes each new token O(n) instead of O(n^2), and the decoder shape maps cleanly onto continuous batching and speculative decoding.\n\nCaveats to mention: encoder-decoder still wins for some seq2seq tasks at fixed compute (T5, translation), and encoders remain the right tool for embeddings and classification, which is why you used ModernBERT in Fiery Scribe instead of an LLM.',
        links: [{ label: 'Wang et al. 2022, arXiv 2204.05832', url: 'https://arxiv.org/abs/2204.05832' }]
      },
      {
        id: 'tr-q-causal-mask',
        kind: 'question',
        title: 'How and where is the causal mask applied?',
        detail:
          'Add negative infinity to the attention logits where key position > query position, before softmax, so those weights become exactly zero. During training this lets one forward pass produce a prediction at every position with teacher forcing. During cached decoding the mask is implicit because the cache only contains past keys.'
      },
      {
        id: 'tr-q-prenorm',
        kind: 'question',
        title: 'Pre-norm versus post-norm: which do modern models use and why?',
        detail:
          'Pre-norm: x + f(norm(x)). Post-norm: norm(x + f(x)). Post-norm has gradients at initialization that grow with depth and needs careful warmup. Pre-norm keeps a clean residual stream, trains stably at depth, and is what GPT-2 onward, LLaMA, and smol-llama use, with a final norm before the output head. Some recent models add norms on both sides or QK-norm inside attention for stability at large scale.'
      },
      {
        id: 'tr-q-multihead',
        kind: 'question',
        title: 'Why multiple heads instead of one attention with the full dimension?',
        detail:
          'Each head projects into a lower-dimensional subspace and computes its own softmax, so different heads can attend to different positions and relations at once (previous token, syntactic head, matching bracket). A single softmax over the full dimension can only produce one attention distribution per query. Compute is the same because the head dimension is d / h. Empirically some heads become redundant, which is part of why GQA works.'
      },
      {
        id: 'tr-q-flops',
        kind: 'question',
        title: 'How do the FLOPs of attention and the MLP compare, and when does attention dominate?',
        detail:
          'Per layer with sequence n and hidden d: the QKV and output projections cost about 8 n d^2, the score and value products cost about 4 n^2 d, and a 4x MLP costs about 16 n d^2. Attention scores dominate when 4 n^2 d > 24 n d^2, that is when n > 6d. For smol-llama (d 960, n 2048) the projections dominate. At 32K context and beyond, the n^2 term wins, which is why FlashAttention and long-context work matter.'
      },
      {
        id: 'tr-q-residual',
        kind: 'question',
        title: 'What do residual connections do in a transformer, and what is the residual stream view?',
        detail:
          'Each block reads from and writes back into a shared vector per position, the residual stream. Gradients flow directly through the identity path, so depth does not kill them. Each block adds a small update, which is why pre-norm works and why you can think of attention as moving information between positions and the MLP as processing within a position.'
      }
    ]
  },
  {
    id: 'llama-recipe',
    n: '03',
    title: 'The LLaMA recipe: RoPE, RMSNorm, SwiGLU, GQA',
    why: 'You implemented all four in three codebases. The interview question is not "what" but "why this and not the alternative", with numbers.',
    anchors: [
      {
        label: 'smol-llama utils/rotary.py',
        note: 'Your RoPE implementation. 15 query and 5 key-value heads, head_dim 64.',
        href: 'https://github.com/weights-and-wires/smol-llama'
      },
      {
        label: 'banana.cpp',
        note: 'RoPE, GQA, MQA, MHA, RMSNorm, LayerNorm, and SwiGLU as C++ layers that load real checkpoints.',
        href: '/projects/banana-cpp'
      },
      {
        label: 'Micro-Llama',
        note: 'The same recipe at 213K parameters with 2 heads and head_dim 32.',
        href: '/projects/vicharak'
      }
    ],
    items: [
      {
        id: 'l-roformer',
        kind: 'learn',
        title: 'Read RoFormer sections 3.2 to 3.4 and derive the relative-position property yourself.',
        links: [{ label: 'arXiv 2104.09864', url: 'https://arxiv.org/abs/2104.09864' }]
      },
      {
        id: 'l-eleuther-rope',
        kind: 'learn',
        title: 'Read the EleutherAI rotary embeddings post for intuition and the implementation trick.',
        links: [{ label: 'Rotary Embeddings: A Relative Revolution', url: 'https://blog.eleuther.ai/rotary-embeddings/' }]
      },
      {
        id: 'l-gqa',
        kind: 'learn',
        title: 'Read the GQA and MQA papers, especially the uptraining section of GQA.',
        links: [
          { label: 'GQA, arXiv 2305.13245', url: 'https://arxiv.org/abs/2305.13245' },
          { label: 'MQA, arXiv 1911.02150', url: 'https://arxiv.org/abs/1911.02150' }
        ]
      },
      {
        id: 'l-glu-rmsnorm',
        kind: 'learn',
        title: 'Read GLU Variants Improve Transformer and the RMSNorm paper.',
        links: [
          { label: 'arXiv 2002.05202', url: 'https://arxiv.org/abs/2002.05202' },
          { label: 'arXiv 1910.07467', url: 'https://arxiv.org/abs/1910.07467' }
        ]
      },
      {
        id: 'l-llama-papers',
        kind: 'learn',
        title: 'Read LLaMA 1 section 2 and the Llama 3 paper sections on architecture and scaling.',
        links: [
          { label: 'LLaMA, arXiv 2302.13971', url: 'https://arxiv.org/abs/2302.13971' },
          { label: 'Llama 3 herd, arXiv 2407.21783', url: 'https://arxiv.org/abs/2407.21783' }
        ]
      },
      {
        id: 'l-context-ext',
        kind: 'learn',
        title: 'Read Position Interpolation and YaRN for context extension of RoPE models.',
        links: [
          { label: 'arXiv 2306.15595', url: 'https://arxiv.org/abs/2306.15595' },
          { label: 'YaRN, arXiv 2309.00071', url: 'https://arxiv.org/abs/2309.00071' }
        ]
      },
      {
        id: 'l-build-rope',
        kind: 'build',
        title: 'Implement RoPE two ways and prove the relative-position property numerically.',
        detail:
          'Write the complex-number version (view pairs as complex, multiply by e^{i m theta}) and the rotate-half real version. Assert they match. Then generate random q and k, apply RoPE at positions (m, n) and (m + 7, n + 7), and show the dot products are equal to float tolerance.'
      },
      {
        id: 'l-build-gpt-to-llama',
        kind: 'build',
        title: 'Convert your from-scratch GPT into LLaMA and load smol-llama weights into it.',
        detail:
          'Swap LayerNorm for RMSNorm, learned positions for RoPE, the GELU MLP for SwiGLU, MHA for GQA, and drop the biases. Load the Hugging Face checkpoint and match the logits of the transformers implementation on a fixed prompt. When the logits match, you understand every tensor name.'
      },
      {
        id: 'l-q-rope',
        kind: 'question',
        title: 'What are rotary embeddings, and what exactly do they do?',
        detail:
          'RoPE encodes position by rotating the query and key vectors instead of adding a position vector. Split each head dimension into d/2 pairs. Pair i at position m is rotated by angle m x theta_i, where theta_i = base^(-2i/d) and base is 10,000 in LLaMA 1 and 2 and 500,000 in Llama 3. Low i rotates fast, high i rotates slowly, so the pairs form a spectrum of frequencies.\n\nThe key property: a rotation is a linear map R(m), and R(m)^T R(n) = R(n - m). So q_m . k_n = (R(m) q)^T (R(n) k) = q^T R(n - m) k, which depends only on the relative offset n - m. You get relative position inside the dot product without an extra bias term, while keeping absolute positions available to each token.\n\nPractical consequences: it is applied to Q and K only, never V. It adds no parameters. Attention scores decay with distance for random q and k, which matches a locality prior. It extrapolates poorly past the training length because high-frequency pairs wrap around into unseen angles, which is what position interpolation, NTK-aware scaling, and YaRN address by rescaling the frequencies.\n\nImplementation: precompute cos and sin tables of shape [seq, d/2], and apply them with the rotate-half trick, x x cos + rotate_half(x) x sin. In banana.cpp and smol-llama you did exactly this.'
      },
      {
        id: 'l-q-rope-qk-only',
        kind: 'question',
        title: 'Why is RoPE applied to queries and keys but not to values?',
        detail:
          'Position should affect how much token i attends to token j, which is the score q . k. The value is the content that gets moved; rotating it would scramble content by position and the output would depend on absolute position in a way the residual stream cannot undo. Keys and queries are rotated so the score depends on relative offset, and values stay position-free.'
      },
      {
        id: 'l-q-rope-base',
        kind: 'question',
        title: 'What does the RoPE base (theta) control, and why did Llama 3 raise it to 500,000?',
        detail:
          'The base sets the lowest frequency: the slowest pair completes a rotation after about 2 pi x base positions. A larger base makes low-frequency pairs rotate slower, so long-range offsets stay distinguishable and the model can use longer contexts. Llama 3 trained at 8K context and raised the base to 500,000 for that reason. Raising it alone is not enough; the model needs some training at long context to use it.'
      },
      {
        id: 'l-q-context-extension',
        kind: 'question',
        title: 'How do you extend the context length of a RoPE model that was trained at 4K?',
        detail:
          'Position interpolation scales positions by L_train / L_target so no pair sees an angle it did not see in training, followed by a short fine-tune. NTK-aware scaling changes the base instead, interpolating low frequencies while leaving high frequencies alone, because high frequencies encode local order that must stay sharp. YaRN combines both with a temperature on attention logits and is what Qwen and others use. All of them need fine-tuning at the new length; naive extrapolation gives loss spikes past the training length.'
      },
      {
        id: 'l-q-gqa',
        kind: 'question',
        title: 'MHA versus MQA versus GQA: what does GQA buy, and what does it cost?',
        detail:
          'The KV cache per token is 2 x layers x kv_heads x head_dim x bytes. MQA uses 1 KV head and shrinks the cache by the number of heads but loses some quality. GQA groups query heads to share a KV head: smol-llama uses 15 query heads and 5 KV heads (3 per group) and shrinks the cache 3x with quality close to MHA. Decode is memory-bandwidth bound and reads the whole cache every step, so a smaller cache means faster decode and larger batches. GQA also converts an MHA checkpoint by mean-pooling the KV heads in each group and uptraining briefly. Cost: slightly fewer parameters in K and V and a small quality gap at very small ratios. DeepSeek MLA is the next step, compressing KV into a low-rank latent.'
      },
      {
        id: 'l-q-rmsnorm',
        kind: 'question',
        title: 'Why RMSNorm instead of LayerNorm?',
        detail:
          'LayerNorm subtracts the mean and divides by the standard deviation, then applies gain and bias. RMSNorm drops mean subtraction and the bias: x / sqrt(mean(x^2) + eps) x g. It is cheaper (one reduction instead of two) and trains to the same quality. The paper argues the re-scaling invariance is what matters, not re-centering. Every LLaMA-family model uses it.'
      },
      {
        id: 'l-q-swiglu',
        kind: 'question',
        title: 'What is SwiGLU and why is the hidden size 8/3 x d?',
        detail:
          'SwiGLU(x) = (Swish(x W_gate) * x W_up) W_down, three matrices instead of two, with Swish(z) = z sigmoid(z). The gate lets the network multiplicatively select features, and GLU variants consistently beat ReLU and GELU MLPs at equal compute. To keep parameters equal to a standard 4d MLP with two matrices, the hidden size is set to 2/3 x 4d = 8/3 x d, rounded to a multiple of 256 for hardware efficiency. For d = 960 that gives 2,560.'
      },
      {
        id: 'l-q-no-bias',
        kind: 'question',
        title: 'Why do LLaMA-style models drop bias terms?',
        detail:
          'PaLM reported that removing biases from dense layers and norms improved training stability at scale with no quality loss. Biases add parameters that the residual stream can absorb anyway. Qwen is an exception: it keeps a bias in the QKV projection, which it reports helps length extrapolation.'
      },
      {
        id: 'l-q-position-encodings',
        kind: 'question',
        title: 'Compare learned absolute, sinusoidal, ALiBi, and RoPE position encodings.',
        detail:
          'Learned absolute (GPT-2): a trainable table added to embeddings, cannot exceed the trained length. Sinusoidal (original transformer): fixed sin and cos features added to embeddings, no parameters, weak extrapolation in practice. ALiBi: no positional input at all, adds a linear distance penalty to attention logits per head, extrapolates well but limits long-range attention. RoPE: rotates Q and K so scores depend on relative offset, adds no parameters, is the default in open models, and extends with frequency rescaling. Some recent models mix RoPE and NoPE layers.'
      }
    ]
  },
  {
    id: 'pretraining',
    n: '04',
    title: 'Pre-training',
    why: 'You ran a compute-optimal pre-training job for $53. Own every number in it and be able to explain what you would change at 10x and 100x the budget.',
    anchors: [
      {
        label: 'smol-llama pretrain.py',
        note: 'Gradient accumulation, mixed precision, checkpointing every 200 steps, cosine LR with 900 warmup steps.',
        href: 'https://github.com/weights-and-wires/smol-llama'
      },
      {
        label: 'Micro-Llama train.py',
        note: 'The same loop in JAX/Flax with device selection and W&B logging.',
        href: 'https://github.com/weights-and-wires/vicharak-llm'
      }
    ],
    items: [
      {
        id: 'p-karpathy-gpt2',
        kind: 'learn',
        title: 'Watch Let us reproduce GPT-2 (124M). Note every speedup step and its measured gain.',
        links: [{ label: 'Karpathy: reproduce GPT-2', url: 'https://www.youtube.com/watch?v=l8pRSuU81PU' }]
      },
      {
        id: 'p-scaling',
        kind: 'learn',
        title: 'Read the Chinchilla paper (approach 2 and 3) and skim Kaplan et al. for the earlier view.',
        links: [
          { label: 'Chinchilla, arXiv 2203.15556', url: 'https://arxiv.org/abs/2203.15556' },
          { label: 'Kaplan, arXiv 2001.08361', url: 'https://arxiv.org/abs/2001.08361' }
        ]
      },
      {
        id: 'p-fineweb',
        kind: 'learn',
        title: 'Read the FineWeb blog post: dedup, filtering, and how they evaluated data quality.',
        links: [{ label: 'FineWeb blog', url: 'https://huggingface.co/spaces/HuggingFaceFW/blogpost-fineweb-v1' }]
      },
      {
        id: 'p-smollm2',
        kind: 'learn',
        title: 'Read the SmolLM2 paper. It is the closest published sibling of smol-llama.',
        links: [{ label: 'arXiv 2502.02737', url: 'https://arxiv.org/abs/2502.02737' }]
      },
      {
        id: 'p-flash',
        kind: 'learn',
        title: 'Read FlashAttention 1 and 2, focusing on the tiling algorithm and the IO complexity argument.',
        links: [
          { label: 'arXiv 2205.14135', url: 'https://arxiv.org/abs/2205.14135' },
          { label: 'arXiv 2307.08691', url: 'https://arxiv.org/abs/2307.08691' }
        ]
      },
      {
        id: 'p-ultrascale',
        kind: 'learn',
        title: 'Read the Ultra-Scale Playbook for parallelism (DP, ZeRO, TP, PP, FSDP) and memory math.',
        links: [{ label: 'Ultra-Scale Playbook', url: 'https://huggingface.co/spaces/nanotron/ultrascale-playbook' }]
      },
      {
        id: 'p-olmo',
        kind: 'learn',
        title: 'Read OLMo 2 for a fully open training recipe, including stability tricks and the WSD schedule.',
        links: [{ label: 'arXiv 2501.00656', url: 'https://arxiv.org/abs/2501.00656' }]
      },
      {
        id: 'p-build-30m',
        kind: 'build',
        title: 'Run a 30M-parameter smol-llama on a cheap GPU and sweep the learning rate.',
        detail:
          'Log tokens per second, MFU, and loss to W&B. Sweep LR over 1e-3, 6e-4, 3e-4, 1e-4 and plot final loss against LR. Then compare cosine against a warmup-stable-decay schedule at equal tokens. You want to be able to say "I have seen what a too-high LR looks like" with a plot.'
      },
      {
        id: 'p-build-mfu',
        kind: 'build',
        title: 'Compute MFU for the original smol-llama run and write a paragraph on the gap.',
        detail:
          'FLOPs per token is about 6N plus the attention term 12 x layers x n x d. Multiply by tokens per second and divide by the H100 PCIe dense bf16 peak (about 756 TFLOP/s). Then list the causes: small matrices, memory-bound norms and attention at head_dim 64, Python overhead, and data loading.'
      },
      {
        id: 'p-build-eval',
        kind: 'build',
        title: 'Run lm-evaluation-harness on smol-llama and record the numbers on the project page.',
        detail: 'HellaSwag, ARC-easy, PIQA, and WinoGrande, with SmolLM2-360M next to them for reference.',
        links: [{ label: 'lm-evaluation-harness', url: 'https://github.com/EleutherAI/lm-evaluation-harness' }]
      },
      {
        id: 'p-q-loss',
        kind: 'question',
        title: 'What is the pre-training loss and objective, exactly?',
        detail:
          'Maximize the log-likelihood of the corpus under an autoregressive factorization: L = -1/T x sum_t log p(x_t | x_<t). In practice, cross-entropy between the next-token distribution and the observed token, averaged over every position in the batch, computed with teacher forcing so all positions train in one forward pass. Perplexity is exp(loss). Starting loss is about ln(vocab): 10.8 for smol-llama. To compare across tokenizers use bits per byte, not per-token loss.'
      },
      {
        id: 'p-q-chinchilla',
        kind: 'question',
        title: 'How many tokens for a given parameter count, and why do modern models train past that?',
        detail:
          'Chinchilla found that at a fixed compute budget, loss is minimized around 20 tokens per parameter, with parameters and tokens scaling equally. smol-llama at 360M and 6B tokens is near that point. Production models train far past it (Llama 3 8B saw 15T tokens, about 1,900 per parameter) because inference cost dominates: a smaller model trained longer is cheaper to serve at almost the same quality. The compute-optimal point is only optimal if you never run inference.'
      },
      {
        id: 'p-q-flops',
        kind: 'question',
        title: 'How many FLOPs does training take, and how do you compute MFU?',
        detail:
          'About 6 FLOPs per parameter per token: 2 for the forward matmuls, 4 for the backward (gradient with respect to inputs and to weights). So total training compute is about 6 x N x D. MFU is achieved FLOP/s divided by the hardware peak. For smol-llama: 6 x 360M x 75K = 162 TFLOP/s against about 756 for an H100 PCIe, about 21%. Good large-scale runs reach 40% to 50%.'
      },
      {
        id: 'p-q-lr-schedule',
        kind: 'question',
        title: 'Why warmup, why cosine, and what is the WSD schedule?',
        detail:
          'Warmup: Adam second-moment estimates are noisy at the start and the loss surface is sharp, so a full-size LR early causes divergence. A few hundred to a few thousand linear steps fixes it. Cosine decay to about 10% of peak lets the model settle into a lower-loss region at the end; you must know the total steps in advance. Warmup-stable-decay holds the peak LR then decays sharply in the last 10% to 20%. It matches cosine and lets you stop at any point by branching a decay, which is why MiniCPM, SmolLM2, and OLMo 2 use it.'
      },
      {
        id: 'p-q-batch',
        kind: 'question',
        title: 'How do you choose the batch size, and what is the critical batch size?',
        detail:
          'Below the critical batch size, doubling the batch nearly halves the steps needed, so it is free. Above it, larger batches waste compute because the gradient is already accurate. The critical size grows as loss falls, which is why runs ramp batch size. Around 0.5M to 4M tokens per step is typical for small and mid models; smol-llama used about 1M. Learning rate scales with batch: roughly sqrt for Adam.'
      },
      {
        id: 'p-q-flashattention',
        kind: 'question',
        title: 'What does FlashAttention actually do?',
        detail:
          'It computes exact attention without materializing the n x n score matrix in HBM. It tiles Q, K, and V into blocks that fit in SRAM, computes partial scores per block, and uses online softmax, keeping a running max and running sum so the normalization can be applied incrementally. The backward recomputes the scores from saved statistics instead of storing them. Memory goes from O(n^2) to O(n), and wall-clock improves because attention is memory-bandwidth bound and the algorithm reads HBM far fewer times. FlashAttention-2 improves parallelism across sequence length and reduces non-matmul FLOPs.'
      },
      {
        id: 'p-q-checkpointing',
        kind: 'question',
        title: 'What does gradient checkpointing trade?',
        detail:
          'Instead of storing every activation for the backward pass, store only block inputs and recompute the rest during backward. Activation memory drops from O(layers) to O(sqrt(layers)) or O(1) per layer, at the cost of about one extra forward pass, roughly 33% more compute. It is what let smol-llama use a micro-batch of 64 x 2,048 on one GPU.'
      },
      {
        id: 'p-q-data',
        kind: 'question',
        title: 'What makes pre-training data good, and how do you check for contamination?',
        detail:
          'Deduplication at document and near-duplicate level (MinHash), language and quality filters, removal of boilerplate, and, in FineWeb-Edu, a classifier trained on LLM judgments of educational value. Data quality moves benchmark scores more than most architecture changes. Contamination is checked by n-gram overlap between evaluation sets and training data (13-gram overlap is a common threshold) and by reporting clean and dirty splits.'
      },
      {
        id: 'p-q-spikes',
        kind: 'question',
        title: 'What causes loss spikes and how do you prevent them?',
        detail:
          'Causes: too-high LR, attention logit growth (softmax saturation), bf16 precision loss in the output softmax, bad data batches, and Adam epsilon issues at scale. Fixes: lower or better-warmed LR, QK-norm or logit soft-capping, z-loss to keep the log-partition near zero, fp32 for the final logits, skipping the offending batch, and gradient clipping. Log the pre-clip grad norm and the max attention logit to see spikes before they hit the loss.'
      },
      {
        id: 'p-q-parallelism',
        kind: 'question',
        title: 'Explain data, tensor, pipeline, and ZeRO or FSDP parallelism in one line each.',
        detail:
          'Data parallel: replicate the model, split the batch, all-reduce gradients. ZeRO and FSDP: still data parallel, but shard optimizer states, gradients, and parameters across ranks and gather on demand, so memory scales down with the number of GPUs. Tensor parallel: split individual matmuls across GPUs within a node, needs all-reduce per layer, so it stays inside NVLink. Pipeline parallel: put different layers on different GPUs and stream micro-batches through to fill the bubble. Sequence or context parallel splits the sequence for very long contexts. Real runs combine them: TP inside a node, PP and DP across nodes.'
      },
      {
        id: 'p-q-scale-7b',
        kind: 'question',
        title: 'How would you scale smol-llama to 7B on 64 H100s?',
        detail:
          'Tokens: 7B x 20 = 140B for compute-optimal, more if inference cost matters. Compute: 6 x 7B x 140B = 5.9e21 FLOPs; at 40% MFU on 64 GPUs (about 25 PFLOP/s) that is about 65 hours. Parallelism: FSDP or ZeRO-3 across all 64 with TP 2 to 4 inside nodes if activation memory demands it, no PP needed at 7B. Batch 4M tokens, LR about 3e-4 with 2K warmup and cosine or WSD, AdamW (0.9, 0.95), decay 0.1, clip 1.0, bf16 with fp32 master weights. Data: pre-tokenized shards with a deterministic sampler so restarts are reproducible. Checkpoint every 30 minutes to object storage. Evaluate every few thousand steps with a small harness. Expect a hardware failure and design for resume.'
      }
    ]
  },
  {
    id: 'sft',
    n: '05',
    title: 'Post-training I: SFT, LoRA, QLoRA',
    why: 'You did SFT at Fiery, QLoRA for served models, and LoRA in PoliteLlama. The interview wants the loss masking, the LoRA math, and honest trade-offs.',
    anchors: [
      {
        label: 'Fiery Chat',
        note: 'SFT of the base model to remove acronym hallucinations in a RAG assistant. QLoRA fine-tunes served on vLLM.',
        href: '/work'
      },
      {
        label: 'PoliteLlama',
        note: 'LoRA adapters on attention layers, 97 MB, trained with Unsloth.',
        href: '/projects/polite-orpo'
      }
    ],
    items: [
      {
        id: 's-lora',
        kind: 'learn',
        title: 'Read the LoRA paper, especially section 7 on which weights to adapt and the rank experiments.',
        links: [{ label: 'arXiv 2106.09685', url: 'https://arxiv.org/abs/2106.09685' }]
      },
      {
        id: 's-qlora',
        kind: 'learn',
        title: 'Read QLoRA: NF4, double quantization, paged optimizers.',
        links: [{ label: 'arXiv 2305.14314', url: 'https://arxiv.org/abs/2305.14314' }]
      },
      {
        id: 's-trl-sft',
        kind: 'learn',
        title: 'Read the TRL SFTTrainer docs on completion-only loss, packing, and chat templates, and the PEFT docs.',
        links: [
          { label: 'TRL SFTTrainer', url: 'https://huggingface.co/docs/trl/sft_trainer' },
          { label: 'PEFT docs', url: 'https://huggingface.co/docs/peft' }
        ]
      },
      {
        id: 's-lima-tulu',
        kind: 'learn',
        title: 'Read LIMA for the "less is more" argument and Tulu 3 for a full open post-training recipe.',
        links: [
          { label: 'LIMA, arXiv 2305.11206', url: 'https://arxiv.org/abs/2305.11206' },
          { label: 'Tulu 3, arXiv 2411.15124', url: 'https://arxiv.org/abs/2411.15124' }
        ]
      },
      {
        id: 's-unsloth',
        kind: 'learn',
        title: 'Read the Unsloth docs on what their kernels fuse and why training is faster.',
        links: [{ label: 'Unsloth docs', url: 'https://docs.unsloth.ai/' }]
      },
      {
        id: 's-build-lora',
        kind: 'build',
        title: 'Implement LoRA by hand and merge it.',
        detail:
          'Wrap nn.Linear with A of shape (r, in) and B of shape (out, r), initialize B to zero, scale by alpha / r. Fine-tune your from-scratch GPT on a small instruction set. Merge with W + B A and confirm the merged model gives identical logits. Count trainable parameters and compare to full fine-tuning.'
      },
      {
        id: 's-build-collator',
        kind: 'build',
        title: 'Write an SFT collator that masks prompt tokens and prove the loss only counts assistant tokens.',
        detail:
          'Set labels to -100 for system and user tokens and for padding. Verify by computing the loss manually over the assistant span and matching the trainer. Then add packing with proper attention masking across packed examples.'
      },
      {
        id: 's-q-sft-loss',
        kind: 'question',
        title: 'What is the SFT loss, and what is masked?',
        detail:
          'The same next-token cross-entropy as pre-training, restricted to the tokens you want the model to produce: the assistant turns. Prompt, system, and padding tokens get label -100 so they contribute nothing. Training on the prompt too works but spends capacity modeling user text and can hurt on short-response data. Packing concatenates examples to fill the context; it needs position ids reset and block-diagonal attention or you leak across examples.'
      },
      {
        id: 's-q-lora-math',
        kind: 'question',
        title: 'Explain LoRA: the math, rank, alpha, which modules, and inference cost.',
        detail:
          'Freeze W and learn a low-rank update: W + (alpha / r) B A, with A of shape r x d_in initialized randomly and B of shape d_out x r initialized to zero so training starts at the base model. Trainable parameters per module are r x (d_in + d_out); for a 3B model with r 16 on attention projections that is a few tens of millions, hence the 97 MB adapter. Rank 8 to 64 covers most tasks; alpha is a scale, often 2r. Applying to all linear layers (including the MLP) matters more than raising the rank. After merging, inference cost is zero extra. Unmerged, multi-adapter serving adds a small matmul per layer.'
      },
      {
        id: 's-q-qlora',
        kind: 'question',
        title: 'What does QLoRA change, and where is the compute done?',
        detail:
          'The frozen base weights are stored in 4-bit NormalFloat, a data type whose quantiles match a normal distribution, with the quantization constants themselves quantized (double quantization). Optimizer states are paged to CPU on memory spikes. The LoRA adapters stay in bf16. On every forward the 4-bit weights are dequantized to bf16 on the fly and the matmul runs in bf16, so compute is not faster, only memory is smaller: a 65B model fits on one 48 GB GPU.'
      },
      {
        id: 's-q-full-vs-lora',
        kind: 'question',
        title: 'When do you full fine-tune versus LoRA?',
        detail:
          'LoRA when data is small, the change is a style or format or narrow behavior, you need many task adapters, or memory is tight. Full fine-tuning when you have a lot of data, you are teaching substantial new knowledge or a new language, or you are doing the final stage of a frontier post-training pipeline. LoRA learns less and forgets less; full fine-tuning can move further but risks catastrophic forgetting and needs replay or a low LR.'
      },
      {
        id: 's-q-fiery-sft',
        kind: 'question',
        title: 'How did SFT remove acronym hallucinations at Fiery, and what could have gone wrong?',
        detail:
          'Build a set of question and answer pairs where every internal acronym is expanded correctly from the knowledge base, fine-tune, and measure the hallucination rate on a held-out acronym set before and after. Risks: overfitting to the phrasing of the training set, format drift away from the base chat template, regressions on general ability, and the model becoming confident about acronyms that have changed since training. Mitigations: mix in general instruction data, keep the LR low, and evaluate broadly, not only on acronyms.'
      },
      {
        id: 's-q-eval-sft',
        kind: 'question',
        title: 'How do you evaluate a fine-tuned chat model?',
        detail:
          'Held-out loss tracks format learning but not quality. Use task-specific automatic metrics where they exist, an LLM-as-judge with a rubric and pairwise comparison against the base model, a small human review, and a regression suite for general capability (MMLU, GSM8K, IFEval). Watch for length bias in judges. For a behavior like politeness, write an adversarial test set covering the edge cases that broke PoliteLlama.'
      }
    ]
  },
  {
    id: 'preference',
    n: '06',
    title: 'Post-training II: reward models, PPO, DPO, ORPO, GRPO',
    why: 'This is where you were asked the most detailed questions. You have shipped ORPO with clean W&B curves. Add a reward model and a GRPO run so every method in the family is something you have run, not read.',
    anchors: [
      {
        label: 'PoliteLlama',
        note: 'ORPO on Llama 3.2 3B. Accuracies to 1.0 by step 1,000, margins to 0.4, logps of chosen and rejected diverging.',
        href: '/projects/polite-orpo'
      },
      {
        label: 'Resume',
        note: 'Mentions GRPO for behavioral alignment. Back it with a run you can describe.',
        href: '/api/cv?format=view'
      }
    ],
    items: [
      {
        id: 'pr-instructgpt',
        kind: 'learn',
        title: 'Read InstructGPT: the three-stage pipeline, the reward model loss, and the PPO objective with the KL and pre-training mix.',
        links: [{ label: 'arXiv 2203.02155', url: 'https://arxiv.org/abs/2203.02155' }]
      },
      {
        id: 'pr-rlhf-book',
        kind: 'learn',
        title: 'Read the RLHF Book chapters on reward modeling, policy gradients, and direct alignment.',
        links: [{ label: 'RLHF Book', url: 'https://rlhfbook.com/' }]
      },
      {
        id: 'pr-dpo',
        kind: 'learn',
        title: 'Read DPO and derive the loss from the Bradley-Terry model and the KL-constrained RL objective.',
        links: [{ label: 'arXiv 2305.18290', url: 'https://arxiv.org/abs/2305.18290' }]
      },
      {
        id: 'pr-orpo',
        kind: 'learn',
        title: 'Reread ORPO and check each claim against your own W&B curves.',
        links: [{ label: 'arXiv 2403.07691', url: 'https://arxiv.org/abs/2403.07691' }]
      },
      {
        id: 'pr-grpo',
        kind: 'learn',
        title: 'Read GRPO in DeepSeekMath section 4, then DeepSeek-R1 for RL with verifiable rewards.',
        links: [
          { label: 'DeepSeekMath, arXiv 2402.03300', url: 'https://arxiv.org/abs/2402.03300' },
          { label: 'DeepSeek-R1, arXiv 2501.12948', url: 'https://arxiv.org/abs/2501.12948' }
        ]
      },
      {
        id: 'pr-grpo-fixes',
        kind: 'learn',
        title: 'Read Dr. GRPO and DAPO for the known biases in GRPO and the fixes.',
        links: [
          { label: 'Dr. GRPO, arXiv 2503.20783', url: 'https://arxiv.org/abs/2503.20783' },
          { label: 'DAPO, arXiv 2503.14476', url: 'https://arxiv.org/abs/2503.14476' }
        ]
      },
      {
        id: 'pr-hf-ppo',
        kind: 'learn',
        title: 'Read The N Implementation Details of RLHF with PPO and Illustrating RLHF.',
        links: [
          { label: 'N implementation details', url: 'https://huggingface.co/blog/the_n_implementation_details_of_rlhf_with_ppo' },
          { label: 'Illustrating RLHF', url: 'https://huggingface.co/blog/rlhf' }
        ]
      },
      {
        id: 'pr-variants',
        kind: 'learn',
        title: 'Skim KTO, SimPO, and Constitutional AI so you can place them relative to DPO.',
        links: [
          { label: 'KTO, arXiv 2402.01306', url: 'https://arxiv.org/abs/2402.01306' },
          { label: 'SimPO, arXiv 2405.14734', url: 'https://arxiv.org/abs/2405.14734' },
          { label: 'Constitutional AI, arXiv 2212.08073', url: 'https://arxiv.org/abs/2212.08073' }
        ]
      },
      {
        id: 'pr-reward-hacking',
        kind: 'learn',
        title: 'Read Lilian Weng on reward hacking in RLHF.',
        links: [{ label: 'Reward Hacking in RL', url: 'https://lilianweng.github.io/posts/2024-11-28-reward-hacking/' }]
      },
      {
        id: 'pr-trl',
        kind: 'learn',
        title: 'Read the TRL trainer docs for DPO, ORPO, GRPO, and RewardTrainer, and the reward functions interface for GRPO.',
        links: [{ label: 'TRL docs', url: 'https://huggingface.co/docs/trl' }]
      },
      {
        id: 'pr-build-losses',
        kind: 'build',
        title: 'Implement the DPO and ORPO losses from scratch and match TRL on a tiny dataset.',
        detail:
          'Compute sequence log-probs of chosen and rejected under the policy and, for DPO, under a frozen reference. Write both losses in under 30 lines. Log the same metrics TRL logs (rewards/accuracies, rewards/margins, logps) and match them for a few steps.'
      },
      {
        id: 'pr-build-rm',
        kind: 'build',
        title: 'Train a Bradley-Terry reward model and look at what it learned.',
        detail:
          'Put a scalar head on a small model, train on a public preference set (UltraFeedback subset) with -log sigmoid(r_w - r_l). Plot the margin distribution, check accuracy on held-out pairs, and test for length bias by correlating reward with response length.'
      },
      {
        id: 'pr-build-grpo',
        kind: 'build',
        title: 'Run GRPO with a verifiable reward on a 0.5B model.',
        detail:
          'Use TRL GRPOTrainer on a GSM8K subset with a reward function that parses the final answer and returns 1 or 0, plus a small format reward. Group size 8, KL beta small. Watch reward mean, completion length, and the fraction of prompts with zero variance in the group (they give no gradient). This is the run that lets you say you have done GRPO.'
      },
      {
        id: 'pr-build-polite-eval',
        kind: 'build',
        title: 'Build the PoliteLlama edge-case eval and record pass rates.',
        detail:
          'Categories: exact "please", slang (pls, plz), misspellings (plis, pleease), implied politeness (could you, would you mind), other languages (por favor, bitte), and adversarial prompts that put "please" in a quoted string. Report a table on the project page.'
      },
      {
        id: 'pr-q-losses',
        kind: 'question',
        title: 'What loss functions and objectives are used for pre-training and post-training?',
        detail:
          'Pre-training: next-token cross-entropy over all positions. Objective: model the data distribution.\n\nSFT: the same cross-entropy restricted to assistant tokens. Objective: imitate demonstrations in the target format.\n\nReward model: Bradley-Terry pairwise loss, -log sigmoid(r(x, y_w) - r(x, y_l)), on a scalar head. Objective: rank responses the way humans do.\n\nPPO (RLHF): maximize E[r(x, y)] - beta KL(pi || pi_ref) with the clipped surrogate objective, a value-function loss, and often a pre-training loss mixed in. Objective: raise reward without drifting far from the reference.\n\nDPO: -log sigmoid(beta [log pi(y_w|x)/pi_ref(y_w|x) - log pi(y_l|x)/pi_ref(y_l|x)]). Objective: the closed-form solution of the same KL-constrained problem, trained offline on pairs.\n\nORPO: L_SFT on chosen plus lambda x (-log sigmoid(log odds(y_w) - log odds(y_l))), with odds(y) = P(y|x) / (1 - P(y|x)). Objective: SFT and preference in one pass without a reference model.\n\nGRPO: the PPO clipped objective with advantage A_i = (r_i - mean(r)) / std(r) over a group of G samples per prompt, plus a KL penalty to the reference, no value network. Objective: online RL with rule-based or model rewards.\n\nOthers you may be asked about: KTO (unpaired, prospect-theoretic), SimPO (length-normalized, reference-free), distillation (KL to a teacher), and process reward models (per-step labels).'
      },
      {
        id: 'pr-q-why-rm',
        kind: 'question',
        title: 'Why do we need reward modeling, and how do reward models work?',
        detail:
          'Why: most of what we want (helpful, honest, well-formatted, safe) has no programmatic checker, and asking humans to score every sample during RL is far too slow. A reward model turns a modest number of human comparisons into a function you can query millions of times, which gives online RL a training signal. Where a verifier exists (math answers, unit tests), you skip the reward model and use the verifier directly; that is RL with verifiable rewards.\n\nHow: take a pre-trained or SFT model, replace the LM head with a scalar head, read it at the final token, and train on pairs with the Bradley-Terry loss -log sigmoid(r_w - r_l). Pairs rather than absolute scores because humans are consistent at comparing and inconsistent at rating. The reward is only defined up to a constant, so it is normalized before RL.\n\nFailure modes: over-optimization (the policy finds inputs where the reward model is wrong, Goodhart), length bias (longer looks better), and distribution shift as the policy moves. Mitigations: the KL penalty, reward model ensembles, retraining the reward model on fresh samples, and length-controlled evaluation. Generative reward models and LLM-as-judge are increasingly used because they scale with the base model and can explain scores.'
      },
      {
        id: 'pr-q-dpo-orpo-grpo',
        kind: 'question',
        title: 'What is the difference between DPO, ORPO, and GRPO?',
        detail:
          'DPO: offline, needs paired chosen and rejected data, needs a frozen reference model for the log-ratios. It optimizes an implicit reward beta x log pi/pi_ref and is stable and cheap, but it cannot explore beyond the pairs and tends to lower the likelihood of both responses over training.\n\nORPO: offline, paired data, no reference model. It combines an SFT loss on the chosen response with an odds-ratio penalty that pushes chosen above rejected. One stage instead of SFT then DPO, half the memory. The odds ratio is more aggressive than a probability ratio when probabilities are small, which the paper argues is why it works from a base model without prior SFT. Your PoliteLlama curves (accuracies to 1.0, margins to 0.4, chosen logps rising while rejected falls) are the textbook picture.\n\nGRPO: online. For each prompt, sample G completions from the current policy, score them with a reward function or reward model, and use the group mean as the baseline: A_i = (r_i - mean) / std. Then apply the PPO clipped objective with a KL term to the reference. No value network, so it is far cheaper than PPO for long generations. It needs a reward you can compute on fresh samples, which is why it pairs with verifiable rewards, and it can explore behaviors that never appear in a fixed dataset.\n\nWhen to use which: DPO or ORPO when you have preference pairs and want a cheap, stable update to style or behavior. GRPO when you have a verifier or a good reward model and want to improve capability (reasoning, code) through exploration.'
      },
      {
        id: 'pr-q-kl',
        kind: 'question',
        title: 'Why does RLHF need a KL penalty to the reference model? What happens without it?',
        detail:
          'The reward model is only accurate near the distribution it was trained on. Without a constraint, the policy moves to regions where the reward model is wrong and exploits it: repeated phrases, extreme length, gibberish that scores high. The KL term keeps the policy close to the reference so the reward stays meaningful and the model keeps its general abilities. Beta trades reward against drift. DPO bakes the same constraint into its loss; GRPO adds it as a per-token estimator.'
      },
      {
        id: 'pr-q-dpo-derivation',
        kind: 'question',
        title: 'Derive DPO from the RLHF objective.',
        detail:
          'The KL-constrained objective max E[r] - beta KL(pi || pi_ref) has the closed-form optimum pi*(y|x) = pi_ref(y|x) exp(r(x, y) / beta) / Z(x). Invert it: r(x, y) = beta log pi*(y|x)/pi_ref(y|x) + beta log Z(x). Plug into the Bradley-Terry likelihood P(y_w > y_l) = sigmoid(r_w - r_l); the log Z terms cancel because both responses share x. Maximizing that likelihood over pi gives the DPO loss. The implicit reward is the beta-scaled log-ratio, which is what TRL logs as rewards/chosen and rewards/rejected.'
      },
      {
        id: 'pr-q-grpo-no-critic',
        kind: 'question',
        title: 'Why does GRPO drop the value network, and what are its known biases?',
        detail:
          'PPO needs a critic to estimate per-token advantages, which is another model the size of the policy and is hard to train for sparse end-of-sequence rewards. GRPO replaces it with the group mean as a Monte Carlo baseline, which is unbiased and needs no extra model, at the cost of more samples per prompt. Known issues: dividing by the group std upweights easy and hard prompts where rewards barely vary, and per-sequence length normalization biases toward longer wrong answers. Dr. GRPO removes both normalizations; DAPO adds clip-higher, dynamic sampling of prompts with zero-variance groups, and token-level loss.'
      },
      {
        id: 'pr-q-good-env',
        kind: 'question',
        title: 'What makes a good RL environment for training LLMs?',
        detail:
          'A reward the policy cannot game: verifiable where possible (exact-match answers, unit tests, compilers), robust to format tricks, and checked for exploits before training. Enough signal: a binary end reward is fine if the success rate is between about 10% and 90%; if every sample in a group succeeds or fails, GRPO has no gradient, so you curate difficulty or use dynamic sampling. Diversity so the model does not overfit to one template. Fast and deterministic evaluation, since you will run millions of rollouts. Clear episode boundaries and a length budget. Partial credit where it does not open a loophole (format reward, step rewards from a process reward model). And a held-out set that measures the real goal, not the proxy, because reward goes up long after the proxy stops meaning anything. Your snake work is a small version of this: the reward was fine, but the state was not Markov, so the environment as observed by the agent was the problem.'
      },
      {
        id: 'pr-q-orpo-curves',
        kind: 'question',
        title: 'Interpret your PoliteLlama training curves. What does each metric mean under ORPO?',
        detail:
          'rewards/chosen and rewards/rejected are beta times the mean per-token log-probability of each response. rewards/accuracies is the fraction of pairs where chosen scores higher; going to 1.0 by step 1,000 means the boundary was learned fast. rewards/margins is the mean difference; climbing to 0.4 shows separation growing, not only ordering. logps/chosen rising from -4 to -1 with logps/rejected falling to -5 is the push-pull of the odds-ratio term. nll_loss is the SFT part; loss is nll plus lambda times the odds-ratio loss. The healthy L-curve and a grad norm under 5 say training was stable. A critic would ask whether the rejected responses were too easy, which is why the edge-case eval matters.'
      },
      {
        id: 'pr-q-online-offline',
        kind: 'question',
        title: 'Online versus offline preference optimization: why does online usually win?',
        detail:
          'Offline methods (DPO, ORPO) learn from a fixed set of pairs generated by some other policy. As the policy changes, its own samples drift away from that data, so the loss stops teaching about the model you have. Online methods (PPO, GRPO, online DPO) score fresh samples from the current policy, so the signal stays on-distribution and the model can discover behaviors absent from the data. The cost is a reward source you can query at training time. Iterative DPO, sampling new pairs every round, is a middle ground.'
      },
      {
        id: 'pr-q-length-bias',
        kind: 'question',
        title: 'What is length bias in preference optimization, and how do you handle it?',
        detail:
          'Humans and reward models rate longer responses higher, so preference training inflates length. Under DPO the log-ratio sums over tokens, which also favors length. Fixes: length-normalized objectives (SimPO), length penalties in the reward, balanced pairs during data curation, and length-controlled win rates in evaluation (AlpacaEval 2 LC).'
      },
      {
        id: 'pr-q-prm-orm',
        kind: 'question',
        title: 'Outcome reward models versus process reward models: what and when?',
        detail:
          'An outcome reward model scores the final answer; a process reward model scores each reasoning step. PRMs give denser credit assignment and enable step-level search at inference, but need step labels (human or Monte Carlo estimated) and are easier to hack. DeepSeek-R1 chose rule-based outcome rewards over a PRM to avoid reward hacking and label cost.'
      }
    ]
  },
  {
    id: 'rl',
    n: '07',
    title: 'RL fundamentals under GRPO',
    why: 'GRPO is PPO with a group baseline, and PPO is a policy gradient with a trust region. You trained a DQN; connect it to the policy-gradient side so RLHF questions bottom out in math you own.',
    anchors: [
      {
        label: 'Snake RL',
        note: 'Dueling Double DQN, target network, 3-step returns, Huber loss, and the v1 lessons about a non-Markov state and a non-detached target.',
        href: '/rl/snake-dqn'
      }
    ],
    items: [
      {
        id: 'r-sutton',
        kind: 'learn',
        title: 'Read Sutton and Barto chapters 3 (MDPs), 6 (TD learning), and 13 (policy gradient).',
        links: [{ label: 'Reinforcement Learning: An Introduction', url: 'http://incompleteideas.net/book/the-book-2nd.html' }]
      },
      {
        id: 'r-dqn-papers',
        kind: 'learn',
        title: 'Read the DQN Nature paper, Double DQN, Dueling DQN, and skim Rainbow for the full list of fixes.',
        links: [
          { label: 'DQN (Nature)', url: 'https://www.nature.com/articles/nature14236' },
          { label: 'Double DQN, arXiv 1509.06461', url: 'https://arxiv.org/abs/1509.06461' },
          { label: 'Dueling, arXiv 1511.06581', url: 'https://arxiv.org/abs/1511.06581' },
          { label: 'Rainbow, arXiv 1710.02298', url: 'https://arxiv.org/abs/1710.02298' }
        ]
      },
      {
        id: 'r-spinning-up',
        kind: 'learn',
        title: 'Work through Spinning Up part 3 (policy gradient derivation) and the PPO page.',
        links: [{ label: 'Spinning Up', url: 'https://spinningup.openai.com/en/latest/spinningup/rl_intro3.html' }]
      },
      {
        id: 'r-ppo-gae',
        kind: 'learn',
        title: 'Read PPO and GAE, then The 37 Implementation Details of PPO.',
        links: [
          { label: 'PPO, arXiv 1707.06347', url: 'https://arxiv.org/abs/1707.06347' },
          { label: 'GAE, arXiv 1506.02438', url: 'https://arxiv.org/abs/1506.02438' },
          { label: '37 implementation details', url: 'https://iclr-blog-track.github.io/2022/03/25/ppo-implementation-details/' }
        ]
      },
      {
        id: 'r-weng-pg',
        kind: 'learn',
        title: 'Read Lilian Weng on policy gradient algorithms as a reference map.',
        links: [{ label: 'Policy Gradient Algorithms', url: 'https://lilianweng.github.io/posts/2018-04-08-policy-gradient/' }]
      },
      {
        id: 'r-build-pg-ladder',
        kind: 'build',
        title: 'Implement REINFORCE, then A2C, then PPO-clip on CartPole, each under 200 lines.',
        detail:
          'Keep the same network and log returns per episode so you can see the variance drop from REINFORCE to A2C and the stability from PPO. Then run PPO on your snake environment with the 28-dim state and compare sample efficiency against the DQN.'
      },
      {
        id: 'r-build-derivation',
        kind: 'build',
        title: 'Derive the policy gradient theorem on paper and reduce GRPO to it.',
        detail:
          'Start from J = E[R], use the log-derivative trick, show that subtracting any state-dependent baseline leaves the gradient unbiased, and arrive at E[A log pi]. Then write the GRPO objective and mark which term is the baseline, which is the trust region, and which is the KL.'
      },
      {
        id: 'r-q-mdp',
        kind: 'question',
        title: 'Define MDP, return, value, Q, and the Bellman equation. What is on-policy versus off-policy?',
        detail:
          'An MDP is states, actions, transition probabilities, rewards, and a discount gamma. The return is the discounted sum of future rewards. V(s) is the expected return from s under the policy; Q(s, a) is the same after taking a. Bellman: Q(s, a) = E[r + gamma max_a Q(s next, a)] for the optimal Q. On-policy methods learn about the policy generating the data (PPO, REINFORCE); off-policy methods learn from data generated by another policy, such as a replay buffer (DQN). PPO is nearly on-policy with a few reuse epochs guarded by the clip.'
      },
      {
        id: 'r-q-dqn-tricks',
        kind: 'question',
        title: 'Why does DQN need a replay buffer and a target network? What did the non-detached target in your v1 do?',
        detail:
          'Replay breaks the correlation between consecutive samples and reuses data, which stabilizes supervised-style updates. The target network holds the bootstrap target fixed for a while so the regression target does not move with every step, which prevents the chasing-your-own-tail divergence. In v1 the target was computed with the online network and not detached, so gradients flowed into the target term and the network was partly trained to make its target match itself, a degenerate solution. v2 fixed it with a frozen target updated every 1,000 steps.'
      },
      {
        id: 'r-q-double-dueling',
        kind: 'question',
        title: 'What do Double DQN and the dueling head fix?',
        detail:
          'Plain DQN takes max over noisy Q estimates, which is biased upward: noise gets selected. Double DQN selects the action with the online network and evaluates it with the target network, decoupling selection from evaluation. The dueling head splits Q into V(s) plus A(s, a) with the advantage mean-centered, so the network learns state value from every action and only needs to learn relative preferences, which helps when many actions have similar value.'
      },
      {
        id: 'r-q-nstep',
        kind: 'question',
        title: 'What do n-step returns trade off?',
        detail:
          'One-step TD has low variance but high bias from bootstrapping on an inaccurate Q. Monte Carlo returns are unbiased but high variance. n-step interpolates: reward for n real steps and bootstrap after. Your 3-step choice propagates the food reward back faster while keeping variance manageable. GAE does the same interpolation continuously with lambda.'
      },
      {
        id: 'r-q-pg-theorem',
        kind: 'question',
        title: 'State the policy gradient theorem. Why subtract a baseline, and what is the advantage?',
        detail:
          'grad J = E[sum_t grad log pi(a_t | s_t) x G_t], where G_t is the return from t. Subtracting a baseline b(s_t) that does not depend on the action leaves the expectation unchanged because E[grad log pi] = 0, but reduces variance. The best simple baseline is V(s_t), and G_t - V(s_t) estimates the advantage A(s_t, a_t): how much better this action was than average. GRPO uses the group mean reward as the baseline; PPO uses a learned critic with GAE.'
      },
      {
        id: 'r-q-ppo-clip',
        kind: 'question',
        title: 'What does PPO clipping do, and why can you take several epochs on one batch?',
        detail:
          'The ratio r = pi_new(a|s) / pi_old(a|s) reweights samples from the old policy. PPO optimizes min(r A, clip(r, 1 - eps, 1 + eps) A), which removes the incentive to move the ratio outside [1 - eps, 1 + eps] in the direction that increases the objective. This is an approximate trust region, so the policy cannot jump far, and the same batch can be reused for a few epochs without the importance weights becoming meaningless. eps 0.2 is standard; DAPO raises the upper clip for LLMs to allow low-probability tokens to grow.'
      },
      {
        id: 'r-q-exploration',
        kind: 'question',
        title: 'How does exploration work in DQN, in PPO, and in LLM RL?',
        detail:
          'DQN: epsilon-greedy with a decay schedule (yours went from 1.0 to 0.005 over 150K steps). PPO: stochastic policy plus an entropy bonus that keeps the distribution from collapsing. LLM RL: sampling at temperature 1 from the policy is the exploration; the group of G samples in GRPO is G explorations of the same prompt. Entropy collapse is a real failure mode in R1-style training, which is why people monitor entropy and use clip-higher.'
      },
      {
        id: 'r-q-markov',
        kind: 'question',
        title: 'Why was the v1 snake state not Markov, and why did that cap performance?',
        detail:
          'The 11 features only described the three cells adjacent to the head. Whether a move sealed the snake into a dead end depended on the body shape, which the state did not contain. Two states with identical features could have opposite optimal actions, so no function of the state could be a correct Q function and the mean plateaued near 18. Adding rays and flood-fill free space made the relevant future observable and the mean jumped to 102. The general lesson is that the agent optimizes the environment it observes, not the one you think you built.'
      },
      {
        id: 'r-q-shaping',
        kind: 'question',
        title: 'What reward shaping is safe?',
        detail:
          'Potential-based shaping, adding F(s, s next) = gamma phi(s next) - phi(s) for any potential phi, provably keeps the optimal policy unchanged. Anything else can change what is optimal, such as a per-step reward for approaching food that makes circling near food better than eating it. In LLM RL, format rewards are shaping; keep them small relative to the task reward.'
      },
      {
        id: 'r-q-map-to-llm',
        kind: 'question',
        title: 'Map the RL vocabulary onto LLM post-training.',
        detail:
          'State: the prompt plus the tokens generated so far. Action: the next token, from a vocabulary of about 128K actions. Policy: the language model. Episode: one completion, ending at EOS or the length budget. Reward: usually one scalar at the end from a reward model or verifier, so credit assignment across tokens is the hard part. Transitions are deterministic (append the token), so the environment is trivial and all the difficulty is in the reward and the exploration.'
      }
    ]
  },
  {
    id: 'inference',
    n: '08',
    title: 'Inference systems: KV cache, batching, speculative decoding, quantization',
    why: 'You built an inference engine and served models on T4s and 1080 Tis. Systems interviews go deep on the memory math and the scheduler, so make every claim on the resume something you can derive.',
    anchors: [
      {
        label: 'banana.cpp',
        note: 'KV cache, speculative decoding, continuous batching, fused kernels, and a 10x speedup claim to back with numbers.',
        href: '/projects/banana-cpp'
      },
      {
        label: 'Micro-Llama',
        note: 'An fp32 KV cache that consumed 131 KB of a 264 KB SRAM budget. The clearest memory-math example you own.',
        href: '/projects/vicharak'
      },
      {
        label: 'Fiery and American Express',
        note: 'vLLM on T4 clusters at sub-second p95 TTFT, Ray Serve on 4 x 1080 Ti, profiling FlashAttention, KV cache, and batching.',
        href: '/work'
      }
    ],
    items: [
      {
        id: 'i-weng',
        kind: 'learn',
        title: 'Read Lilian Weng on transformer inference optimization as the map of the field.',
        links: [{ label: 'Large Transformer Model Inference Optimization', url: 'https://lilianweng.github.io/posts/2023-01-10-inference-optimization/' }]
      },
      {
        id: 'i-kipply',
        kind: 'learn',
        title: 'Read Transformer Inference Arithmetic and redo every calculation for smol-llama and Llama 3 8B.',
        links: [{ label: 'kipply: inference arithmetic', url: 'https://kipp.ly/transformer-inference-arithmetic/' }]
      },
      {
        id: 'i-vllm-orca',
        kind: 'learn',
        title: 'Read the vLLM PagedAttention paper and the Orca paper on iteration-level scheduling.',
        links: [
          { label: 'vLLM, arXiv 2309.06180', url: 'https://arxiv.org/abs/2309.06180' },
          { label: 'Orca (OSDI 22)', url: 'https://www.usenix.org/conference/osdi22/presentation/yu' }
        ]
      },
      {
        id: 'i-specdec',
        kind: 'learn',
        title: 'Read both speculative decoding papers, then Medusa and EAGLE for draft-free variants.',
        links: [
          { label: 'Leviathan, arXiv 2211.17192', url: 'https://arxiv.org/abs/2211.17192' },
          { label: 'Chen, arXiv 2302.01318', url: 'https://arxiv.org/abs/2302.01318' },
          { label: 'Medusa, arXiv 2401.10774', url: 'https://arxiv.org/abs/2401.10774' },
          { label: 'EAGLE, arXiv 2401.15077', url: 'https://arxiv.org/abs/2401.15077' }
        ]
      },
      {
        id: 'i-sglang',
        kind: 'learn',
        title: 'Read SGLang RadixAttention for prefix caching, and compare with vLLM automatic prefix caching.',
        links: [{ label: 'arXiv 2312.07104', url: 'https://arxiv.org/abs/2312.07104' }]
      },
      {
        id: 'i-quant',
        kind: 'learn',
        title: 'Read A Visual Guide to Quantization, then LLM.int8, GPTQ, and AWQ.',
        links: [
          { label: 'Visual guide to quantization', url: 'https://newsletter.maartengrootendorst.com/p/a-visual-guide-to-quantization' },
          { label: 'LLM.int8, arXiv 2208.07339', url: 'https://arxiv.org/abs/2208.07339' },
          { label: 'GPTQ, arXiv 2210.17323', url: 'https://arxiv.org/abs/2210.17323' },
          { label: 'AWQ, arXiv 2306.00978', url: 'https://arxiv.org/abs/2306.00978' }
        ]
      },
      {
        id: 'i-gguf',
        kind: 'learn',
        title: 'Read the GGUF spec and the llama.cpp quantization scheme names (Q4_K_M and friends).',
        links: [{ label: 'GGUF spec', url: 'https://github.com/ggml-org/ggml/blob/master/docs/gguf.md' }]
      },
      {
        id: 'i-vllm-docs',
        kind: 'learn',
        title: 'Read the vLLM docs on scheduling, chunked prefill, prefix caching, and multi-LoRA.',
        links: [{ label: 'vLLM docs', url: 'https://docs.vllm.ai/' }]
      },
      {
        id: 'i-build-kv',
        kind: 'build',
        title: 'Add a KV cache with a prefill and decode split to your from-scratch GPT and measure it.',
        detail:
          'Measure tokens per second with and without the cache at several sequence lengths. Compute the cache size from the formula and check it against torch.cuda.max_memory_allocated. Then show that batching 8 sequences in decode barely changes latency per step.'
      },
      {
        id: 'i-build-specdec',
        kind: 'build',
        title: 'Implement speculative decoding with SmolLM2-135M drafting for SmolLM2-1.7B.',
        detail:
          'Same tokenizer, so no remapping. Draft k tokens, verify with one target forward, accept with probability min(1, p_target / p_draft), and resample from the residual on rejection. Report acceptance rate and end-to-end speedup at batch 1 and batch 16. Then explain why the speedup shrinks at batch 16.'
      },
      {
        id: 'i-build-scheduler',
        kind: 'build',
        title: 'Simulate a continuous batching scheduler in Python.',
        detail:
          'Poisson arrivals with random prompt and output lengths, a prefill cost per token, a decode step cost that grows with batch size and total KV tokens, a KV memory budget, and preemption when it runs out. Compare throughput and p95 latency against static batching. Add chunked prefill and show the effect on decode latency for running requests.'
      },
      {
        id: 'i-build-quant',
        kind: 'build',
        title: 'Quantize a model to INT8 per-channel and INT4 group-wise by hand and measure perplexity.',
        detail:
          'Implement symmetric absmax quantization per output channel for INT8 and group size 128 for INT4 with a scale per group. Dequantize on the fly in a custom Linear. Report perplexity on WikiText-2 for bf16, INT8, INT4 g128, and INT4 per-channel to see why groups matter.'
      },
      {
        id: 'i-q-kv-cache',
        kind: 'question',
        title: 'What is the KV cache, why does it work, and how big is it?',
        detail:
          'In causal attention the keys and values for past positions never change once computed, because each position only attends backward. So during decoding you store K and V per layer and per KV head, and each new token computes only its own Q, K, V, attends over the stored cache, and appends. Per-token compute drops from O(n^2) to O(n), and prefill (all prompt tokens at once) is separated from decode (one token per step).\n\nSize per token = 2 (K and V) x layers x kv_heads x head_dim x bytes. smol-llama in bf16: 2 x 32 x 5 x 64 x 2 = 40 KB per token, 80 MB for a 2,048 context. Llama 3 8B in bf16: 2 x 32 x 8 x 128 x 2 = 128 KB per token, so an 8K context is 1 GB per sequence and a batch of 32 such sequences is 32 GB, more than the 16 GB of weights. Micro-Llama in fp32: 2 x 2 x 2 x 32 x 128 x 4 = 131 KB, which was half the RP2040 SRAM. This is why GQA, MLA, KV quantization, and paged allocation exist.'
      },
      {
        id: 'i-q-prefill-decode',
        kind: 'question',
        title: 'Prefill versus decode: what bounds each, and what are TTFT and TPOT?',
        detail:
          'Prefill processes the whole prompt in one pass: large matmuls, compute-bound, and it determines time to first token (TTFT). Decode produces one token per step per sequence: every step reads all weights and the whole KV cache to do a small amount of math, so it is memory-bandwidth bound and determines time per output token (TPOT). At batch 1 an 8B bf16 model reads 16 GB per token; on an H100 at 3.3 TB/s that is about 5 ms per token regardless of how fast the math is. Batching more sequences into a decode step reuses the same weight read, so throughput rises almost linearly until arithmetic intensity hits the compute roofline.'
      },
      {
        id: 'i-q-continuous-batching',
        kind: 'question',
        title: 'What is continuous batching, and what problem does it solve?',
        detail:
          'Static batching waits for a batch to fill, runs all sequences together, and the batch finishes only when its longest sequence finishes, so short requests hold GPU slots idle and new requests wait. Continuous (iteration-level) batching, from the Orca paper, makes the scheduling decision at every decode step: finished sequences leave immediately, waiting requests join, and prefill for a new request can be interleaved with decode for running ones. Utilization and throughput go up several times and queueing latency drops. Chunked prefill splits a long prompt across steps so one huge prefill does not stall the decode latency of everyone else. It depends on the KV cache being allocated per sequence, which is where paged attention comes in.'
      },
      {
        id: 'i-q-paged-attention',
        kind: 'question',
        title: 'What is PagedAttention, and what did it fix?',
        detail:
          'Before vLLM, servers reserved a contiguous KV buffer for the maximum possible length per request, wasting 60% to 80% of KV memory to internal fragmentation and reservation. PagedAttention borrows virtual memory: KV is stored in fixed-size blocks (for example 16 tokens), a per-sequence block table maps logical to physical blocks, and blocks are allocated on demand. Near-zero waste means larger batches, which is where the throughput gain came from. Blocks can be shared across sequences with copy-on-write, which makes parallel sampling and beam search cheap and enables prefix caching of shared system prompts.'
      },
      {
        id: 'i-q-specdec',
        kind: 'question',
        title: 'How does speculative decoding work, why is it lossless, and when does it not help?',
        detail:
          'A small draft model proposes k tokens autoregressively. The target model then scores all k plus one positions in a single forward pass, which costs about the same as one decode step because decode is memory-bound. Each draft token is accepted with probability min(1, p_target / p_draft); on the first rejection you sample from the normalized max(0, p_target - p_draft) and stop. This rejection-sampling scheme produces exactly the target distribution, so it is lossless. Expected speedup depends on acceptance rate and k; 2x to 3x at batch 1 is typical. It stops helping when the server is already compute-bound at large batch, because the verification pass is no longer free, and when the draft is poorly aligned with the target. Medusa and EAGLE replace the draft model with extra heads on the target.'
      },
      {
        id: 'i-q-quantization',
        kind: 'question',
        title: 'Weight-only versus weight-and-activation quantization, per-channel versus group-wise, and why do activations have outliers?',
        detail:
          'Weight-only INT8 or INT4 (GPTQ, AWQ, GGUF k-quants) shrinks memory and speeds up memory-bound decode, with dequantization to bf16 before the matmul. Weight-and-activation INT8 or FP8 (LLM.int8, SmoothQuant, FP8 on Hopper) also speeds up compute-bound prefill because the matmul runs in low precision. Per-channel scaling uses one scale per output channel; group-wise uses a scale per 32 to 128 weights and tolerates INT4 far better. Activations in LLMs above a few billion parameters develop a few channels with values 100x larger than the rest; a single scale would crush everything else, so LLM.int8 keeps outlier channels in fp16 and SmoothQuant migrates the difficulty into the weights. KV cache quantization to INT8 or FP8 is nearly free and doubles the batch you can hold.'
      },
      {
        id: 'i-q-sampling',
        kind: 'question',
        title: 'Explain temperature, top-k, top-p, and min-p. Why does greedy decoding loop?',
        detail:
          'Temperature divides the logits before softmax: below 1 sharpens, above 1 flattens. Top-k keeps the k most likely tokens and renormalizes. Top-p (nucleus) keeps the smallest set whose cumulative probability reaches p, which adapts to how peaked the distribution is. Min-p keeps tokens whose probability is at least a fraction of the top token, which behaves better at high temperature. Greedy decoding maximizes each step locally and tends to fall into high-probability repetitive cycles because repeating a phrase raises its probability further; repetition penalties and sampling break the cycle.'
      },
      {
        id: 'i-q-engines',
        kind: 'question',
        title: 'vLLM versus SGLang versus TensorRT-LLM versus llama.cpp: when would you choose each?',
        detail:
          'vLLM: the default for GPU serving, broad model support, paged KV, continuous batching, prefix caching, multi-LoRA, OpenAI-compatible API. SGLang: similar core with RadixAttention prefix caching and a frontend for structured programs; strong when many requests share long prefixes or need constrained decoding. TensorRT-LLM: highest throughput on NVIDIA when you can afford the compile step and want FP8 and in-flight batching tuned per GPU. llama.cpp: CPU and Apple Silicon, GGUF quantization, single-user or edge. At Fiery you chose vLLM over SGLang because tool-calling support was ahead at the time; say that and what you would re-evaluate.'
      },
      {
        id: 'i-q-roofline',
        kind: 'question',
        title: 'How do you decide if a kernel is memory-bound or compute-bound?',
        detail:
          'Arithmetic intensity is FLOPs per byte moved from HBM. The ridge point of the hardware is peak FLOP/s divided by bandwidth: for an H100 about 990e12 / 3.35e12, roughly 300 FLOPs per byte. Below that intensity you are memory-bound and only fusion or fewer bytes help; above it, only more FLOP throughput helps. Decode at batch 1 has intensity around 1 (two FLOPs per weight byte read in bf16), deeply memory-bound. Prefill at 2K tokens has intensity in the thousands and is compute-bound. This is the first thing to compute before optimizing anything.'
      },
      {
        id: 'i-q-multi-lora',
        kind: 'question',
        title: 'How do you serve many LoRA adapters on one base model?',
        detail:
          'Keep one copy of the base weights and load adapters into GPU memory on demand; each request carries an adapter id. The extra computation is x A B per adapted layer, which is batched across requests with different adapters using a gathered or segmented matmul kernel (Punica, S-LoRA). vLLM supports this natively. The trade-off is a small per-token latency cost versus merged adapters that need a separate model copy each.'
      },
      {
        id: 'i-q-design-serving',
        kind: 'question',
        title: 'Design a serving system for a 70B model at 1,000 concurrent users with TTFT under 1 s.',
        detail:
          'Weights: 140 GB in bf16 or 70 GB in FP8, so tensor parallel across 2 to 4 H100s per replica, FP8 to leave KV room. KV budget: about 160 KB per token for 70B with GQA in bf16, half in FP8; size the batch from the memory left after weights. Scheduler: continuous batching with chunked prefill and prefix caching for the shared system prompt. Latency: speculative decoding for low-batch periods, disabled when compute-bound. Routing: a front-end that routes by prefix hash for cache hits and load balances across replicas; autoscale replicas on queue depth. SLOs: TTFT p95 under 1 s and TPOT around 30 ms, with admission control that rejects rather than queues past the budget. Observability: per-request tokens, TTFT, TPOT, cache hit rate, and preemption count.'
      }
    ]
  },
  {
    id: 'kernels',
    n: '09',
    title: 'GPU kernels and performance',
    why: 'The resume targets CUDA and low-level runtime roles, and banana.cpp claims fused kernels. Make the claim concrete with a matmul, a fused softmax, and a FlashAttention forward you wrote yourself.',
    anchors: [
      {
        label: 'banana.cpp',
        note: 'CPU parallelization and fused kernels behind the 10x claim.',
        href: '/projects/banana-cpp'
      },
      {
        label: 'PoliteLlama via Unsloth',
        note: 'You used Triton kernels for training; this module is about being able to write them.',
        href: '/projects/polite-orpo'
      }
    ],
    items: [
      {
        id: 'k-brrr',
        kind: 'learn',
        title: 'Read Making Deep Learning Go Brrrr From First Principles: compute, memory, and overhead.',
        links: [{ label: 'Horace He: brrr', url: 'https://horace.io/brrr_intro.html' }]
      },
      {
        id: 'k-cuda-guide',
        kind: 'learn',
        title: 'Read the CUDA programming guide chapters on the programming model and the memory hierarchy.',
        links: [{ label: 'CUDA C++ Programming Guide', url: 'https://docs.nvidia.com/cuda/cuda-c-programming-guide/' }]
      },
      {
        id: 'k-boehm',
        kind: 'learn',
        title: 'Read How to Optimize a CUDA Matmul Kernel and reproduce each kernel.',
        links: [{ label: 'siboehm: CUDA MMM', url: 'https://siboehm.com/articles/22/CUDA-MMM' }]
      },
      {
        id: 'k-triton',
        kind: 'learn',
        title: 'Do the Triton tutorials: vector add, fused softmax, matmul, and layer norm.',
        links: [{ label: 'Triton tutorials', url: 'https://triton-lang.org/main/getting-started/tutorials/index.html' }]
      },
      {
        id: 'k-online-softmax',
        kind: 'learn',
        title: 'Read Online normalizer calculation for softmax and the From Online Softmax to FlashAttention notes.',
        links: [
          { label: 'arXiv 1805.02867', url: 'https://arxiv.org/abs/1805.02867' },
          { label: 'From online softmax to FlashAttention', url: 'https://courses.cs.washington.edu/courses/cse599m/23sp/notes/flashattn.pdf' }
        ]
      },
      {
        id: 'k-gpu-mode',
        kind: 'learn',
        title: 'Watch the GPU MODE lectures on profiling, Triton, and FlashAttention.',
        links: [{ label: 'GPU MODE', url: 'https://www.youtube.com/@GPUMODE' }]
      },
      {
        id: 'k-build-sgemm',
        kind: 'build',
        title: 'Write a tiled SGEMM in CUDA in four steps and plot GFLOP/s against cuBLAS.',
        detail:
          'Naive one thread per output, then global memory coalescing, then shared-memory tiling, then register blocking with a 2D thread tile. Record the fraction of cuBLAS at each step. Stop when you can explain every jump.'
      },
      {
        id: 'k-build-triton-fused',
        kind: 'build',
        title: 'Write fused softmax and fused RMSNorm kernels in Triton and benchmark them.',
        detail: 'Compare against PyTorch eager and torch.compile at several row sizes. Explain the result with bytes moved.'
      },
      {
        id: 'k-build-flash',
        kind: 'build',
        title: 'Implement the FlashAttention forward pass with online softmax and verify against SDPA.',
        detail:
          'In Triton on GPU, or in C++ for banana.cpp on CPU. Tile over keys, keep a running max and sum per query row, rescale the accumulator when the max changes. Match torch SDPA to 1e-3 in bf16, then measure memory at 8K sequence length.'
      },
      {
        id: 'k-q-memory-hierarchy',
        kind: 'question',
        title: 'Describe the GPU memory hierarchy and why FlashAttention is called IO-aware.',
        detail:
          'HBM is large (80 GB) and slow relative to compute (3.35 TB/s on H100). SRAM or shared memory per streaming multiprocessor is small (about 228 KB) and roughly an order of magnitude faster. Registers are faster still. Standard attention writes the n x n scores to HBM and reads them back for softmax and again for the value product. FlashAttention keeps a tile of Q, K, V, and scores in SRAM, finishes the block, and only writes the output, so HBM traffic drops from O(n^2) to O(n^2 d^2 / M) with M the SRAM size. It counts memory accesses, not FLOPs, hence IO-aware.'
      },
      {
        id: 'k-q-fusion',
        kind: 'question',
        title: 'What is kernel fusion and why does it help?',
        detail:
          'Elementwise and reduction ops such as bias add, activation, dropout, residual add, and norms are memory-bound: each reads and writes the whole tensor and does almost no math. Running them as separate kernels moves the tensor through HBM once per op. Fusing them into one kernel reads the input once and writes the output once. Fusion also removes kernel launch overhead, which dominates for small tensors. torch.compile does this automatically by generating Triton; Unsloth does it by hand for the training path.'
      },
      {
        id: 'k-q-online-softmax',
        kind: 'question',
        title: 'How do you compute softmax in a single pass with online normalization?',
        detail:
          'Keep a running max m and running sum l. For each new block of values x: m_new = max(m, max(x)), l = l x exp(m - m_new) + sum(exp(x - m_new)), m = m_new. The rescaling factor exp(m - m_new) corrects the earlier sum for the new max. FlashAttention applies the same factor to the partial output accumulator O so that O = sum(exp(s - m) V) / l comes out exactly right after the last block.'
      },
      {
        id: 'k-q-cuda-terms',
        kind: 'question',
        title: 'Define warp, block, grid, occupancy, coalesced access, and bank conflict.',
        detail:
          'A warp is 32 threads executing in lockstep. A block is a group of warps that share shared memory and can synchronize. A grid is all blocks of a launch. Occupancy is the fraction of the SM warp slots in use, limited by registers and shared memory per block. Coalesced access means consecutive threads read consecutive addresses so the warp issues one wide transaction. A bank conflict is two threads in a warp hitting different addresses in the same shared memory bank, which serializes the access; padding the tile by one column is the usual fix.'
      },
      {
        id: 'k-q-torch-compile',
        kind: 'question',
        title: 'What does torch.compile do?',
        detail:
          'TorchDynamo captures the Python program into an FX graph by tracing bytecode, with graph breaks where it cannot. AOTAutograd produces the backward graph. TorchInductor lowers the graph, fuses elementwise and reduction ops, and generates Triton kernels for GPU and C++ for CPU, with autotuning. Gains come from fusion and from removing Python overhead. smol-llama used it for training speed.'
      },
      {
        id: 'k-q-cpu-inference',
        kind: 'question',
        title: 'What did the CPU-side optimizations in banana.cpp actually do?',
        detail:
          'Decode is a sequence of matrix-vector products, bandwidth-bound on CPU as on GPU, so the wins are reading weights once per token in a cache-friendly layout, using SIMD (AVX2 or NEON) for the dot products, threading across output rows or heads, blocking for L2, and keeping activations in fp32 while weights stay fp16 or bf16 with in-register conversion. Fusing RMSNorm into the following matvec and the SwiGLU gate into the up-projection removes passes over activations. Have the before-and-after tokens per second for one model ready.'
      }
    ]
  },
  {
    id: 'retrieval-agents',
    n: '10',
    title: 'Retrieval, encoders, and agent systems',
    why: 'The day job. Interviewers who ask about encoders often want to hear that you know when not to use a decoder. Your Fiery Scribe and American Express work are the examples.',
    anchors: [
      {
        label: 'American Express',
        note: 'Hybrid dense and keyword retrieval over 200K+ docs, p95 from 30 s to 2 s.',
        href: '/work'
      },
      {
        label: 'Fiery Scribe and Fiery Chat',
        note: 'ModernBERT plus an LLM for structured output, and a RAG assistant with SFT.',
        href: '/work'
      },
      {
        label: 'NoPokeDB',
        note: 'HNSW through hnswlib with SQLite metadata and crash recovery.',
        href: '/projects/nopokedb'
      },
      {
        label: 'Wand AI',
        note: 'Agent workflow observability, credential flows, and dynamic tool definitions.',
        href: '/work'
      }
    ],
    items: [
      {
        id: 'ra-modernbert',
        kind: 'learn',
        title: 'Read ModernBERT: what changed versus BERT and why encoders still matter.',
        links: [{ label: 'arXiv 2412.13663', url: 'https://arxiv.org/abs/2412.13663' }]
      },
      {
        id: 'ra-hnsw',
        kind: 'learn',
        title: 'Read the HNSW paper and the hnswlib README on M, efConstruction, and ef.',
        links: [
          { label: 'arXiv 1603.09320', url: 'https://arxiv.org/abs/1603.09320' },
          { label: 'hnswlib', url: 'https://github.com/nmslib/hnswlib' }
        ]
      },
      {
        id: 'ra-embeddings',
        kind: 'learn',
        title: 'Read Sentence-BERT and ColBERT, and skim the MTEB leaderboard methodology.',
        links: [
          { label: 'Sentence-BERT, arXiv 1908.10084', url: 'https://arxiv.org/abs/1908.10084' },
          { label: 'ColBERT, arXiv 2004.12832', url: 'https://arxiv.org/abs/2004.12832' },
          { label: 'MTEB, arXiv 2210.07316', url: 'https://arxiv.org/abs/2210.07316' }
        ]
      },
      {
        id: 'ra-rrf',
        kind: 'learn',
        title: 'Read the reciprocal rank fusion paper and the original RAG paper.',
        links: [
          { label: 'RRF (Cormack et al.)', url: 'https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf' },
          { label: 'RAG, arXiv 2005.11401', url: 'https://arxiv.org/abs/2005.11401' }
        ]
      },
      {
        id: 'ra-agents',
        kind: 'learn',
        title: 'Read Building effective agents for the vocabulary of workflows versus agents.',
        links: [{ label: 'Building effective agents', url: 'https://www.anthropic.com/research/building-effective-agents' }]
      },
      {
        id: 'ra-build-hybrid',
        kind: 'build',
        title: 'Rebuild the American Express retriever in miniature and measure it.',
        detail:
          'BM25 plus a small dense model (bge-small) over a 10K-document corpus, RRF fusion, a cross-encoder reranker on the top 50. Measure recall at 10 and p95 latency for each stage. Add a query cache and an embedding cache and report the delta so you can tell the 30 s to 2 s story with a reproduction.'
      },
      {
        id: 'ra-build-hnsw-sweep',
        kind: 'build',
        title: 'Sweep HNSW parameters in NoPokeDB and plot recall against QPS.',
        detail: 'M in 8, 16, 32; efConstruction in 100, 200; efSearch from 16 to 512. Publish the plot in the repo README.'
      },
      {
        id: 'ra-q-encoder-use',
        kind: 'question',
        title: 'When do you use an encoder-only model, and why ModernBERT for Fiery Scribe instead of an LLM?',
        detail:
          'Encoders are the right tool when the output is a representation or a label rather than text: embeddings for retrieval, classification, token tagging, reranking. They see the whole input bidirectionally, are 10x to 100x cheaper than a decoder of similar quality on those tasks, and run at low latency on a CPU or a small GPU. For Scribe, the intent and slot structure of a print request is a classification and extraction problem; ModernBERT handles it deterministically and cheaply, and the LLM is reserved for the parts that need generation. ModernBERT modernized BERT with RoPE, GeGLU, unpadding, FlashAttention, an 8K context, and 2T tokens of training.'
      },
      {
        id: 'ra-q-bi-cross',
        kind: 'question',
        title: 'Bi-encoder versus cross-encoder: why rerank?',
        detail:
          'A bi-encoder embeds query and document independently, so documents can be indexed once and searched with a nearest-neighbor index; that is what makes retrieval over 200K documents fast. A cross-encoder reads the query and document together and is far more accurate but costs a forward pass per pair, so it can only score a short list. The standard pipeline is a bi-encoder or hybrid first stage to get the top 50 to 100, then a cross-encoder rerank. ColBERT sits between them with late interaction over token embeddings.'
      },
      {
        id: 'ra-q-hnsw',
        kind: 'question',
        title: 'How does HNSW work, what do M and ef control, and why is recall below 1?',
        detail:
          'A hierarchy of proximity graphs: sparse upper layers for long jumps, a dense bottom layer with all points. Search starts at the top, greedily descends to the nearest neighbor at each layer, then does a best-first search at the bottom keeping ef candidates. M is the number of links per node (memory and recall), efConstruction is the candidate list size during insertion (build time and graph quality), efSearch is the candidate list size during queries (latency and recall). Recall is below 1 because greedy search can get stuck in a local region; raising ef trades latency for recall. Deletions are the weak point, which is why NoPokeDB keeps metadata in SQLite.'
      },
      {
        id: 'ra-q-hybrid',
        kind: 'question',
        title: 'Why hybrid retrieval, and how do you fuse dense and keyword results?',
        detail:
          'Dense embeddings capture paraphrase and semantics but miss exact identifiers, part numbers, acronyms, and rare terms, which is where BM25 is strong. Fusion by reciprocal rank (score = sum of 1 / (k + rank), k about 60) needs no score calibration and works well; weighted score fusion needs normalized scores. Then rerank. Evaluate by recall at k on a labeled query set, not by eyeballing.'
      },
      {
        id: 'ra-q-latency-story',
        kind: 'question',
        title: 'How exactly did you cut p95 from 30 s to 2 s?',
        detail:
          'Have the causal chain ready. Typical shape: profiling showed the time was in embedding the query on CPU, in an unindexed keyword scan, and in reranking too many candidates. Fixes: cache query embeddings and popular results, move embeddings to a GPU or a smaller model, limit the rerank set, parallelize the two retrievers, add an index to the keyword store, and precompute document embeddings offline. Give the p50 as well as the p95 and say what the error budget was.'
      },
      {
        id: 'ra-q-rag-eval',
        kind: 'question',
        title: 'How do you evaluate a RAG system, and how do you choose chunking?',
        detail:
          'Separate retrieval from generation. Retrieval: recall at k and MRR on a labeled set of query to relevant-chunk pairs. Generation: faithfulness (is every claim supported by the retrieved context), answer correctness against references, and refusal rate when context is missing, usually with an LLM judge plus spot-checked human labels. Chunking: start with 256 to 512 tokens with overlap, respect document structure (headings, paragraphs), store parent context for the generator, and tune on retrieval recall, not intuition.'
      },
      {
        id: 'ra-q-tool-calling',
        kind: 'question',
        title: 'How does an LLM call a tool, and how do you make tool use robust and observable?',
        detail:
          'The model emits a structured block (JSON with a function name and arguments) that the runtime parses, executes, and returns as a tool-result message; the model then continues. Robustness: constrained decoding or schema validation, retries with the validation error fed back, idempotent tools, timeouts, and allowlists. Observability, as you built at Wand AI: a span per model call and per tool call under a trace id, with inputs, outputs, tokens, latency, and cost, streamed as events so a failed run can be replayed. Credential handling belongs in the runtime (OAuth flows with deferred auth and token refresh), never in the prompt.'
      },
      {
        id: 'ra-q-sft-vs-rag',
        kind: 'question',
        title: 'Fine-tuning versus retrieval for knowledge: when each?',
        detail:
          'Retrieval for facts that change, need citation, or are too many to memorize; the model stays general and updates are a reindex. Fine-tuning for behavior, format, domain vocabulary, and tone, and for skills the base model lacks. Fiery Chat used both: RAG for the knowledge base and SFT to stop the model inventing acronym expansions. Fine-tuning to inject facts works poorly and is hard to update.'
      }
    ]
  },
  {
    id: 'loop',
    n: '11',
    title: 'The interview loop',
    why: 'Knowledge decays without retrieval practice. This module is the cadence that keeps the rest of the page true.',
    anchors: [],
    items: [
      {
        id: 'lp-weekly-code',
        kind: 'build',
        title: 'Weekly: one timed from-scratch coding drill, rotating through the list.',
        detail:
          'Rotation: decoder-only transformer, attention with backward in NumPy, KV cache with prefill and decode, LoRA wrapper and merge, DPO and ORPO losses, byte-level BPE, PPO-clip on CartPole, online softmax. Time-box each to 45 minutes. Uncheck this item every Monday.'
      },
      {
        id: 'lp-weekly-questions',
        kind: 'build',
        title: 'Weekly: answer five random questions from this page out loud in under two minutes each.',
        detail:
          'Record yourself. If you stumble, uncheck that question so it comes back. Aim to answer with one concrete number or artifact from your own work in every answer.'
      },
      {
        id: 'lp-stories',
        kind: 'build',
        title: 'Write a 90-second story for every artifact on this page.',
        detail:
          'Structure: the problem, the decision and the alternative you rejected, the numbers, what broke, and what you would do differently. Store them in the repo next to this file.'
      },
      {
        id: 'lp-log',
        kind: 'build',
        title: 'After every interview, add the questions you were asked to the curriculum.',
        detail: 'Append them as question items in src/lib/roadmap/curriculum.ts under the right module, with an answer sketch written the same day while the gap is fresh.'
      },
      {
        id: 'lp-q-went-wrong',
        kind: 'question',
        title: 'Tell me about a training run that went wrong and what you learned.',
        detail:
          'Two ready answers. Snake v1: a non-detached bootstrap target plus a non-Markov state; the fix was a frozen target network and a 28-feature state, and the mean went from 18 to 102. PoliteLlama: the preference data encoded a lexical rule, so the model learned tokens rather than politeness and fails on misspellings and implied requests; the lesson is that preference optimization amplifies whatever pattern is in the data, and the fix is semantic coverage in the dataset plus an adversarial eval.'
      },
      {
        id: 'lp-q-deepest',
        kind: 'question',
        title: 'Walk me through your most technically deep project end to end.',
        detail:
          'smol-llama. Motivation and budget, the tokenizer decision, the architecture and why each component, the training configuration with the Chinchilla check, the throughput and MFU with the honest gap analysis, what the loss curve looked like, how you evaluated, and what you would change. Then bridge to banana.cpp: the same model running in your own C++ engine with a KV cache.'
      },
      {
        id: 'lp-q-tradeoff',
        kind: 'question',
        title: 'Describe a technical decision where you chose against the popular option.',
        detail:
          'vLLM over SGLang at Fiery for tool-calling support, with what you would re-evaluate. Or ORPO over SFT then DPO for PoliteLlama, trading a reference-free single stage against the risk of a less-studied method. Name the criteria, the evidence, and the reversibility.'
      }
    ]
  }
];

/** Every item across every module, in order. */
export const ALL_ITEMS: Item[] = MODULES.flatMap((m) => m.items);

export const TOTAL_ITEMS = ALL_ITEMS.length;

export const KIND_LABEL: Record<ItemKind, string> = {
  learn: 'Learn',
  build: 'Build',
  question: 'Questions'
};

export const KIND_ORDER: ItemKind[] = ['learn', 'build', 'question'];
