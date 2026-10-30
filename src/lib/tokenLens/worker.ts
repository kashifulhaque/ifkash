import {
  pipeline,
  env,
  LogitsProcessor,
  LogitsProcessorList,
  InterruptableStoppingCriteria,
  StoppingCriteriaList,
  type TextGenerationPipeline,
} from '@huggingface/transformers';
import { sampleTopP, softmaxStats, topKProbs } from './math';
import type { LensBackend, LensOptions, LensStep, LensWorkerToMain, MainToLensWorker, TokenCandidate } from './types';

env.allowLocalModels = false;

// Same model and dtype as the site agent, so the browser cache is shared.
const MODEL_ID = 'onnx-community/Qwen3-0.6B-ONNX';
const SYSTEM = 'You are a concise assistant.';
const TOP_P = 0.9;

let generator: TextGenerationPipeline | null = null;
let usedDevice: LensBackend = 'wasm';
let loading = false;
let busy = false;
const stopper = new InterruptableStoppingCriteria();

function post(msg: LensWorkerToMain) {
  self.postMessage(msg);
}

self.onmessage = async (event: MessageEvent<MainToLensWorker>) => {
  const msg = event.data;
  switch (msg.type) {
    case 'load':
      await loadModel();
      break;
    case 'generate':
      await generate(msg.prompt, msg.options ?? {});
      break;
    case 'stop':
      stopper.interrupt();
      break;
  }
};

async function loadModel() {
  if (generator) {
    post({ type: 'ready', backend: usedDevice });
    return;
  }
  if (loading) return;
  loading = true;
  try {
    post({ type: 'progress', status: 'downloading', progress: 0, message: 'Starting download...' });

    let device: LensBackend = 'wasm';
    try {
      // @ts-ignore - navigator.gpu may not exist
      if (navigator.gpu) {
        // @ts-ignore
        const adapter = await navigator.gpu.requestAdapter();
        if (adapter) device = 'webgpu';
      }
    } catch {
      device = 'wasm';
    }

    // @ts-ignore - pipeline() produces complex union type
    generator = (await pipeline('text-generation', MODEL_ID, {
      device,
      dtype: device === 'webgpu' ? 'q4f16' : 'q4',
      progress_callback: (progress: any) => {
        if (progress.status === 'progress') {
          post({
            type: 'progress',
            status: 'downloading',
            progress: Math.round(progress.progress ?? 0),
            message: progress.file ? `Downloading ${progress.file}` : 'Downloading model...',
          });
        } else if (progress.status === 'ready') {
          post({ type: 'progress', status: 'loading', progress: 100, message: 'Initializing model...' });
        }
      },
    })) as TextGenerationPipeline;

    usedDevice = device;
    post({ type: 'ready', backend: device });
  } catch (err: any) {
    post({ type: 'error', error: err?.message ?? 'Failed to load model' });
  } finally {
    loading = false;
  }
}

/**
 * Runs inside model.generate() on every step, after the forward pass and with
 * KV cache reuse. It reads the raw logits, reports the real distribution, then
 * does the sampling itself and rewrites the logits so that the (greedy) sampler
 * inside generate() can only pick the token chosen here. That makes `chosen`
 * exact, and applies top-p, which transformers.js 3.8.1 does not implement.
 */
class LensProcessor extends LogitsProcessor {
  index = 0;
  lastChosen = -1;
  private last = performance.now();

  constructor(
    private decode: (id: number) => string,
    private topK: number,
    private temperature: number,
  ) {
    super();
  }

  _call(_input_ids: bigint[][], logits: any) {
    const data = logits.data as Float32Array;
    const vocab = logits.dims.at(-1) as number;
    const offset = data.length - vocab; // last position, batch 0

    const stats = softmaxStats(data, 1, offset, vocab);
    const top = topKProbs(data, this.topK, stats, offset, vocab);

    let chosenId: number;
    if (this.temperature <= 0) {
      chosenId = top[0]?.id ?? 0;
      if (top.length === 0) chosenId = topKProbs(data, 1, stats, offset, vocab)[0].id;
    } else {
      chosenId = sampleTopP(data, this.temperature, TOP_P, Math.random, offset, vocab);
    }

    const toCandidate = (id: number, p: number): TokenCandidate => ({ text: this.decode(id), id, p });
    const candidates = top.map((c) => toCandidate(c.id, c.p));
    const inTop = top.find((c) => c.id === chosenId);
    const chosenP = inTop ? inTop.p : Math.exp(data[offset + chosenId] - stats.max - stats.logZ);

    const now = performance.now();
    const step: LensStep = {
      index: this.index++,
      chosen: toCandidate(chosenId, chosenP),
      candidates,
      entropy: stats.entropyBits,
      ms: now - this.last,
    };
    this.last = now;
    this.lastChosen = chosenId;
    post({ type: 'step', step });

    // Force the greedy sampler to take exactly the chosen token.
    data.fill(-Infinity, offset, offset + vocab);
    data[offset + chosenId] = 0;
    return logits;
  }
}

function eosIds(model: any): number[] {
  const e = model?.generation_config?.eos_token_id;
  if (e == null) return [];
  return (Array.isArray(e) ? e : [e]).map(Number);
}

async function generate(prompt: string, options: LensOptions) {
  if (!generator) {
    post({ type: 'error', error: 'Model not loaded' });
    return;
  }
  if (busy) {
    post({ type: 'error', error: 'Already generating' });
    return;
  }
  busy = true;
  try {
    const tokenizer = generator.tokenizer;
    const model = generator.model as any;
    const topK = Math.max(1, Math.floor(options.topK ?? 8));
    const maxNewTokens = Math.max(1, Math.floor(options.maxNewTokens ?? 48));
    const temperature = options.temperature ?? 0.8;

    const decode = (id: number) => tokenizer.decode([id]);

    // The user prompt's own tokens, decoded one by one (not the chat template).
    const promptIds = tokenizer(prompt, { add_special_tokens: false }).input_ids.tolist()[0] as bigint[];
    post({ type: 'prompt', tokens: promptIds.map((id) => decode(Number(id))) });

    const promptText = tokenizer.apply_chat_template(
      [
        { role: 'system', content: SYSTEM },
        { role: 'user', content: prompt },
      ] as any,
      {
        tokenize: false,
        add_generation_prompt: true,
        // @ts-ignore - enable_thinking is a Qwen3-specific template variable
        enable_thinking: false,
      },
    ) as string;

    const inputs = tokenizer(promptText, { return_tensors: 'pt', add_special_tokens: false });

    stopper.reset();
    const processor = new LensProcessor(decode, topK, temperature);
    const processors = new LogitsProcessorList();
    processors.push(processor);
    const criteria = new StoppingCriteriaList();
    criteria.push(stopper);

    const start = performance.now();
    await model.generate({
      ...inputs,
      max_new_tokens: maxNewTokens,
      // Greedy at the library level: all sampling happens in LensProcessor.
      do_sample: false,
      num_beams: 1,
      logits_processor: processors,
      stopping_criteria: criteria,
    });
    const seconds = (performance.now() - start) / 1000;

    const eos = eosIds(model);
    const reason: 'eos' | 'length' | 'stopped' = stopper.interrupted
      ? 'stopped'
      : eos.includes(processor.lastChosen)
        ? 'eos'
        : 'length';
    post({ type: 'done', reason, tokensPerSecond: seconds > 0 ? processor.index / seconds : 0 });
  } catch (err: any) {
    post({ type: 'error', error: err?.message ?? 'Generation failed' });
  } finally {
    busy = false;
  }
}
