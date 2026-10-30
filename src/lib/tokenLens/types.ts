// Message protocol between the Token Lens page and its Web Worker.
// The worker runs the same local model as the site agent and reports the real
// next-token distribution at every decoding step.

export type LensBackend = 'webgpu' | 'wasm';

export interface TokenCandidate {
  /** Decoded text of the token, including any leading space (for example " the"). */
  text: string;
  /** Vocabulary id of the token. */
  id: number;
  /** Softmax probability at temperature 1, in [0, 1]. */
  p: number;
}

export interface LensStep {
  /** Zero-based index of the generated token. */
  index: number;
  /** The token the model actually emitted at this step. */
  chosen: TokenCandidate;
  /** Top-k candidates sorted by descending probability. May or may not include `chosen`. */
  candidates: TokenCandidate[];
  /** Shannon entropy of the full distribution, in bits. */
  entropy: number;
  /** Milliseconds spent on this step. */
  ms: number;
}

export interface LensOptions {
  /** Number of candidates to report per step. Default 8. */
  topK?: number;
  /** Maximum number of tokens to generate. Default 48. */
  maxNewTokens?: number;
  /** 0 means greedy decoding; otherwise sample at this temperature. Default 0.8. */
  temperature?: number;
}

export type MainToLensWorker =
  | { type: 'load' }
  | { type: 'generate'; prompt: string; options?: LensOptions }
  | { type: 'stop' };

export type LensWorkerToMain =
  | { type: 'progress'; status: 'downloading' | 'loading'; progress: number; message?: string }
  | { type: 'ready'; backend: LensBackend }
  /** Sent once after tokenizing the prompt, before the first step. */
  | { type: 'prompt'; tokens: string[] }
  | { type: 'step'; step: LensStep }
  | { type: 'done'; reason: 'eos' | 'length' | 'stopped'; tokensPerSecond: number }
  | { type: 'error'; error: string };
