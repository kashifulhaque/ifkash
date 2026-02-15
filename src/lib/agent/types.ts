export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface AgentState {
  status: 'idle' | 'downloading' | 'loading' | 'ready' | 'generating' | 'error';
  progress?: number;
  error?: string;
  backend?: 'webgpu' | 'wasm';
}

export type NavigationSuggestion = {
  label: string;
  path: string;
};

// Messages sent from main thread to worker
export type MainToWorkerMessage =
  | { type: 'load' }
  | { type: 'generate'; messages: ChatMessage[] }
  | { type: 'unload' }
  | { type: 'status' };

// Messages sent from worker to main thread
export type WorkerToMainMessage =
  | { type: 'progress'; status: 'downloading' | 'loading'; progress: number; message?: string }
  | { type: 'ready'; backend: 'webgpu' | 'wasm' }
  | { type: 'token'; data: string }
  | { type: 'done'; content: string }
  | { type: 'unloaded' }
  | { type: 'status'; state: AgentState['status'] }
  | { type: 'error'; error: string };
