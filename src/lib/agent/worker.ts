import {
  pipeline,
  TextStreamer,
  env,
  type TextGenerationPipeline,
} from '@huggingface/transformers';
import { SYSTEM_PROMPT } from './context';
import type { ChatMessage, MainToWorkerMessage } from './types';

// Configure transformers.js for browser usage
env.allowLocalModels = false;

const MODEL_ID = 'onnx-community/Qwen3-0.6B-ONNX';

let generator: TextGenerationPipeline | null = null;
let usedDevice: 'webgpu' | 'wasm' = 'wasm';
let currentState: 'idle' | 'downloading' | 'loading' | 'ready' | 'generating' = 'idle';

self.onmessage = async (event: MessageEvent<MainToWorkerMessage>) => {
  const msg = event.data;

  switch (msg.type) {
    case 'load':
      await loadModel();
      break;
    case 'generate':
      await generate(msg.messages);
      break;
    case 'unload':
      await unloadModel();
      break;
    case 'status':
      self.postMessage({ type: 'status', state: currentState });
      break;
  }
};

async function loadModel() {
  if (generator) {
    self.postMessage({ type: 'ready', backend: usedDevice });
    return;
  }

  try {
    currentState = 'downloading';
    self.postMessage({ type: 'progress', status: 'downloading', progress: 0, message: 'Starting download...' });

    // Detect WebGPU support
    let device: 'webgpu' | 'wasm' = 'wasm';
    try {
      // @ts-ignore - navigator.gpu may not exist
      if (navigator.gpu) {
        // @ts-ignore
        const adapter = await navigator.gpu.requestAdapter();
        if (adapter) {
          device = 'webgpu';
        }
      }
    } catch {
      device = 'wasm';
    }

    currentState = 'downloading';

    // @ts-ignore - pipeline() produces complex union type
    generator = (await pipeline('text-generation', MODEL_ID, {
      device,
      dtype: device === 'webgpu' ? 'q4f16' : 'q4',
      progress_callback: (progress: any) => {
        if (progress.status === 'progress') {
          const pct = progress.progress ?? 0;
          self.postMessage({
            type: 'progress',
            status: 'downloading',
            progress: Math.round(pct),
            message: progress.file ? `Downloading ${progress.file}` : 'Downloading model...',
          });
        } else if (progress.status === 'ready') {
          currentState = 'loading';
          self.postMessage({
            type: 'progress',
            status: 'loading',
            progress: 100,
            message: 'Initializing model...',
          });
        }
      },
    })) as TextGenerationPipeline;

    usedDevice = device;
    currentState = 'ready';
    self.postMessage({ type: 'ready', backend: device });
  } catch (err: any) {
    currentState = 'idle';
    self.postMessage({ type: 'error', error: err?.message ?? 'Failed to load model' });
  }
}

async function generate(messages: ChatMessage[]) {
  if (!generator) {
    self.postMessage({ type: 'error', error: 'Model not loaded' });
    return;
  }

  try {
    currentState = 'generating';

    const tokenizer = generator.tokenizer;
    const model = generator.model;

    // Build conversation with system prompt
    const conversation: ChatMessage[] = [
      { role: 'system', content: SYSTEM_PROMPT },
      ...messages,
    ];

    // Apply chat template to get the full prompt text, then tokenize
    // Disable thinking mode for Qwen3 to get direct answers
    const promptText = tokenizer.apply_chat_template(conversation as any, {
      tokenize: false,
      add_generation_prompt: true,
      // @ts-ignore - enable_thinking is a Qwen3-specific template variable
      enable_thinking: false,
    }) as string;

    const inputs = tokenizer(promptText, {
      return_tensors: 'pt',
      add_special_tokens: false,
    });

    const promptLength = inputs.input_ids.dims[1];

    // Set up streamer that skips the prompt tokens
    const streamer = new TextStreamer(tokenizer, {
      skip_prompt: true,
      skip_special_tokens: true,
      callback_function: (text: string) => {
        self.postMessage({ type: 'token', data: text });
      },
    });

    // Generate directly on the model
    const outputIds = await model.generate({
      ...inputs,
      max_new_tokens: 512,
      do_sample: true,
      temperature: 0.7,
      top_p: 0.9,
      streamer,
    });

    // Decode only the new tokens (skip the prompt)
    const newTokens = (outputIds as any).slice(null, [promptLength, null]);
    let content = tokenizer.batch_decode(newTokens, {
      skip_special_tokens: true,
    })[0] ?? '';

    // Strip any residual <think>...</think> blocks from Qwen3 output
    content = content.replace(/<think>[\s\S]*?<\/think>/g, '').trim();

    currentState = 'ready';
    self.postMessage({ type: 'done', content });
  } catch (err: any) {
    currentState = 'ready';
    self.postMessage({ type: 'error', error: err?.message ?? 'Generation failed' });
  }
}

async function unloadModel() {
  // Signal unloaded, then close the worker entirely.
  // This avoids OrtRun IO Binding errors from dispose().
  generator = null;
  currentState = 'idle';
  self.postMessage({ type: 'unloaded' });
  self.close();
}
