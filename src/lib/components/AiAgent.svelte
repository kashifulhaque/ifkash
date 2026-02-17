<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import { NAVIGATION_MAP } from '$lib/agent/context';
  import type { ChatMessage, AgentState, WorkerToMainMessage, NavigationSuggestion } from '$lib/agent/types';

  let isOpen = false;
  let agentState: AgentState = { status: 'idle' };
  let messages: ChatMessage[] = [];
  let inputText = '';
  let currentStreamingText = '';
  let messagesContainer: HTMLDivElement;
  let inputEl: HTMLTextAreaElement;
  let worker: Worker | null = null;

  // Parsed navigation suggestions per message index
  let navSuggestions: Map<number, NavigationSuggestion[]> = new Map();

  // Navigation regex: [Navigate: /path]
  const NAV_REGEX = /\[Navigate:\s*(\/[^\]]*)\]/g;

  function togglePanel() {
    isOpen = !isOpen;
    if (isOpen) {
      tick().then(() => inputEl?.focus());
    }
  }

  function closePanel() {
    isOpen = false;
  }

  function initWorker() {
    if (worker) return;
    worker = new Worker(new URL('$lib/agent/worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = handleWorkerMessage;
  }

  function handleWorkerMessage(event: MessageEvent<WorkerToMainMessage>) {
    const msg = event.data;

    switch (msg.type) {
      case 'progress':
        agentState = {
          status: msg.status === 'downloading' ? 'downloading' : 'loading',
          progress: msg.progress,
        };
        break;

      case 'ready':
        agentState = { status: 'ready', backend: msg.backend };
        break;

      case 'token':
        currentStreamingText += msg.data;
        scrollToBottom();
        break;

      case 'done': {
        let finalText = currentStreamingText || msg.content;
        // Strip any <think>...</think> blocks from Qwen3 output
        finalText = finalText.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
        finalText = finalText.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
        messages = [...messages, { role: 'assistant', content: finalText, speed: msg.speed }];
        // Parse navigation suggestions from the completed message
        // Parse navigation suggestions from the completed message
        const idx = messages.length - 1;
        const suggestions = parseNavigationSuggestions(finalText);
        if (suggestions.length > 0) {
          navSuggestions = new Map(navSuggestions);
          navSuggestions.set(idx, suggestions);
        }
        currentStreamingText = '';
        agentState = { ...agentState, status: 'ready' };
        scrollToBottom();
        break;
      }

      case 'unloaded':
        agentState = { status: 'idle' };
        messages = [];
        navSuggestions = new Map();
        currentStreamingText = '';
        break;

      case 'status':
        agentState = { ...agentState, status: msg.state };
        break;

      case 'error':
        agentState = { status: 'error', error: msg.error };
        currentStreamingText = '';
        break;
    }
  }

  function loadModel() {
    initWorker();
    worker?.postMessage({ type: 'load' });
  }

  function unloadModel() {
    worker?.postMessage({ type: 'unload' });
    // Worker will self.close() after posting 'unloaded'.
    // Clean up the reference so a fresh worker is created on next load.
    worker = null;
  }

  function sendMessage() {
    const text = inputText.trim();
    if (!text || agentState.status !== 'ready') return;

    messages = [...messages, { role: 'user', content: text }];
    inputText = '';
    currentStreamingText = '';
    agentState = { ...agentState, status: 'generating' };
    scrollToBottom();

    // Send only user/assistant messages (not system) to the worker
    const conversationHistory = messages.filter(m => m.role !== 'system');
    worker?.postMessage({ type: 'generate', messages: conversationHistory });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
    if (e.key === 'Escape') {
      closePanel();
    }
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      closePanel();
    }
  }

  function parseNavigationSuggestions(text: string): NavigationSuggestion[] {
    const suggestions: NavigationSuggestion[] = [];
    let match;

    // Match explicit [Navigate: /path] markers
    const regex = new RegExp(NAV_REGEX.source, 'g');
    while ((match = regex.exec(text)) !== null) {
      const path = match[1].trim();
      const label = getLabelForPath(path);
      suggestions.push({ label, path });
    }

    // Fallback: match common references against NAVIGATION_MAP
    if (suggestions.length === 0) {
      const lowerText = text.toLowerCase();
      for (const [keyword, path] of Object.entries(NAVIGATION_MAP)) {
        if (lowerText.includes(keyword.toLowerCase()) && !suggestions.some(s => s.path === path)) {
          // Only add if the keyword appears as a likely reference
          if (keyword.length >= 3) {
            suggestions.push({ label: getLabelForPath(path), path });
          }
        }
      }
      // Limit fallback suggestions to 3
      suggestions.splice(3);
    }

    return suggestions;
  }

  function getLabelForPath(path: string): string {
    const labels: Record<string, string> = {
      '/': 'Home',
      '/work': 'Work Experience',
      '/projects': 'Projects',
      '/projects/banana-cpp': 'banana.cpp',
      '/projects/smol-llama': 'smol-llama',
      '/projects/smoltorch': 'smoltorch',
      '/projects/nopokedb': 'NoPokeDB',
      '/projects/boo': 'Boo',
      '/projects/ferray': 'ferray',
      '/education': 'Education',
      '/blog': 'Blog',
      '/api/resume?format=view': 'View Resume',
      '/leetcode': 'LeetCode',
      '/tensara': 'Tensara',
      '/news': 'Hacker News',
    };
    return labels[path] || path;
  }

  function stripNavigationMarkers(text: string): string {
    return text.replace(NAV_REGEX, '').trim();
  }

  function renderMarkdown(text: string): string {
    let cleaned = stripNavigationMarkers(text);
    // Strip <think>...</think> blocks (Qwen3 thinking mode)
    cleaned = cleaned.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
    // Also strip incomplete opening <think> tags during streaming
    cleaned = cleaned.replace(/<think>[\s\S]*$/g, '').trim();
    const html = marked.parse(cleaned, { async: false }) as string;
    return DOMPurify.sanitize(html);
  }

  function navigateTo(path: string) {
    if (path.startsWith('/api/')) {
      window.open(path, '_blank');
    } else {
      goto(path);
    }
    closePanel();
  }

  async function scrollToBottom() {
    await tick();
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  }

  function getPlaceholder(): string {
    switch (agentState.status) {
      case 'idle': return 'Load the model first...';
      case 'downloading': return 'Downloading model...';
      case 'loading': return 'Loading model...';
      case 'generating': return 'Thinking...';
      case 'error': return 'An error occurred';
      case 'ready': return 'Ask me anything...';
      default: return 'Ask me anything...';
    }
  }

  $: canSend = agentState.status === 'ready' && inputText.trim().length > 0;
  $: isModelLoaded = ['ready', 'generating'].includes(agentState.status);
  $: isLoading = ['downloading', 'loading'].includes(agentState.status);

  onMount(() => {
    window.addEventListener('keydown', handleGlobalKeydown);

    return () => {
      window.removeEventListener('keydown', handleGlobalKeydown);
      if (worker) {
        worker.terminate();
        worker = null;
      }
    };
  });
</script>

<!-- Floating button -->
<button class="agent-fab" class:active={isOpen} class:pulse={agentState.status === 'ready' && !isOpen} on:click={togglePanel} aria-label="AI Assistant">
  {#if isOpen}
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <line x1="18" y1="6" x2="6" y2="18"></line>
      <line x1="6" y1="6" x2="18" y2="18"></line>
    </svg>
  {:else}
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 8V4H8"></path>
      <rect width="16" height="12" x="4" y="8" rx="2"></rect>
      <path d="M2 14h2"></path>
      <path d="M20 14h2"></path>
      <path d="M15 13v2"></path>
      <path d="M9 13v2"></path>
    </svg>
  {/if}
</button>

<!-- Chat panel -->
{#if isOpen}
  <div class="agent-panel" role="dialog" aria-label="AI Assistant Chat">
    <!-- Header -->
    <div class="agent-header">
      <div class="agent-header-left">
        <span class="agent-title">AI Assistant</span>
        {#if agentState.backend}
          <span class="agent-badge">{agentState.backend === 'webgpu' ? 'WebGPU' : 'WASM'}</span>
        {/if}
      </div>
      <div class="agent-header-right">
        {#if isModelLoaded}
          <button class="agent-btn-sm" on:click={unloadModel} title="Unload model">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M18.36 6.64a9 9 0 1 1-12.73 0"></path>
              <line x1="12" y1="2" x2="12" y2="12"></line>
            </svg>
          </button>
        {/if}
        <button class="agent-btn-sm" on:click={closePanel} title="Close">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>

    <!-- Body -->
    <div class="agent-body" bind:this={messagesContainer}>
      {#if agentState.status === 'idle'}
        <!-- Initial prompt -->
        <div class="agent-welcome">
          <div class="welcome-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 8V4H8"></path>
              <rect width="16" height="12" x="4" y="8" rx="2"></rect>
              <path d="M2 14h2"></path>
              <path d="M20 14h2"></path>
              <path d="M15 13v2"></path>
              <path d="M9 13v2"></path>
            </svg>
          </div>
          <p class="welcome-text">
            This loads a ~300 MB language model that runs entirely in your browser. No data is sent to any server.
          </p>
          <button class="agent-btn-load" on:click={loadModel}>
            Load Model
          </button>
        </div>
      {:else if isLoading}
        <!-- Loading state -->
        <div class="agent-welcome">
          <div class="welcome-icon loading">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 8V4H8"></path>
              <rect width="16" height="12" x="4" y="8" rx="2"></rect>
              <path d="M2 14h2"></path>
              <path d="M20 14h2"></path>
              <path d="M15 13v2"></path>
              <path d="M9 13v2"></path>
            </svg>
          </div>
          <div class="progress-section">
            <div class="progress-bar-track">
              <div class="progress-bar-fill" style="width: {agentState.progress ?? 0}%"></div>
            </div>
            <span class="progress-label">
              {agentState.status === 'downloading' ? `Downloading... ${agentState.progress ?? 0}%` : 'Initializing model...'}
            </span>
          </div>
        </div>
      {:else if agentState.status === 'error'}
        <!-- Error state -->
        <div class="agent-welcome">
          <p class="error-text">{agentState.error ?? 'Something went wrong.'}</p>
          <button class="agent-btn-load" on:click={loadModel}>
            Retry
          </button>
        </div>
      {:else}
        <!-- Chat messages -->
        {#if messages.length === 0 && !currentStreamingText}
          <div class="agent-empty">
            <p>Ask me about Kashif's work, projects, education, or skills.</p>
          </div>
        {/if}

        {#each messages as message, i}
          <div class="msg" class:msg-user={message.role === 'user'} class:msg-assistant={message.role === 'assistant'}>
            {#if message.role === 'user'}
              <div class="msg-bubble msg-bubble-user">{message.content}</div>
            {:else}
              <div class="msg-bubble msg-bubble-assistant">
                {@html renderMarkdown(message.content)}
                {@html renderMarkdown(message.content)}
              </div>
              {#if message.speed}
                <div class="msg-speed">
                  {message.speed}
                </div>
              {/if}
              {#if navSuggestions.has(i)}
                <div class="nav-pills">
                  {#each navSuggestions.get(i) ?? [] as suggestion}
                    <button class="nav-pill" on:click={() => navigateTo(suggestion.path)}>
                      {suggestion.label} →
                    </button>
                  {/each}
                </div>
              {/if}
            {/if}
          </div>
        {/each}

        {#if currentStreamingText}
          <div class="msg msg-assistant">
            <div class="msg-bubble msg-bubble-assistant">
              {@html renderMarkdown(currentStreamingText)}
              <span class="cursor-blink">▍</span>
            </div>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Input -->
    <div class="agent-input-area">
      <textarea
        bind:this={inputEl}
        bind:value={inputText}
        on:keydown={handleKeydown}
        placeholder={getPlaceholder()}
        disabled={!isModelLoaded}
        rows="1"
        class="agent-input"
      ></textarea>
      <button
        class="agent-send-btn"
        on:click={sendMessage}
        disabled={!canSend}
        title="Send message"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="22" y1="2" x2="11" y2="13"></line>
          <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
        </svg>
      </button>
    </div>
  </div>
{/if}

<style>
  /* ═══════════════════════════════════════════════════════════════════════
     FLOATING ACTION BUTTON
     ═══════════════════════════════════════════════════════════════════════ */
  .agent-fab {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    z-index: 50;
    width: 48px;
    height: 48px;
    border-radius: 50%;
    border: 1px solid var(--glass-border);
    background: rgba(10, 10, 10, 0.9);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    color: var(--gray-400);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--duration-base) var(--ease-out);
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
  }

  .agent-fab:hover {
    color: var(--white);
    border-color: var(--glass-border-hover);
    transform: scale(1.05);
  }

  .agent-fab.active {
    color: var(--white);
    border-color: var(--gray-600);
  }

  .agent-fab.pulse {
    animation: fab-pulse 3s ease-in-out infinite;
  }

  @keyframes fab-pulse {
    0%, 100% { box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5); }
    50% { box-shadow: 0 4px 24px rgba(255, 255, 255, 0.08); }
  }

  /* ═══════════════════════════════════════════════════════════════════════
     CHAT PANEL
     ═══════════════════════════════════════════════════════════════════════ */
  .agent-panel {
    position: fixed;
    bottom: 4.5rem;
    right: 1rem;
    z-index: 50;
    width: 380px;
    height: 500px;
    max-height: calc(100vh - 6rem);
    display: flex;
    flex-direction: column;
    background: rgba(10, 10, 10, 0.95);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-lg);
    box-shadow: 0 8px 48px rgba(0, 0, 0, 0.6);
    animation: panel-enter var(--duration-base) var(--ease-out);
    overflow: hidden;
  }

  @keyframes panel-enter {
    from {
      opacity: 0;
      transform: translateY(12px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @media (max-width: 480px) {
    .agent-panel {
      width: calc(100vw - 2rem);
      right: 1rem;
      left: 1rem;
      height: calc(100vh - 6rem);
      max-height: calc(100vh - 6rem);
    }
  }

  /* ═══════════════════════════════════════════════════════════════════════
     HEADER
     ═══════════════════════════════════════════════════════════════════════ */
  .agent-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--glass-border);
    flex-shrink: 0;
  }

  .agent-header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .agent-title {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--white);
  }

  .agent-badge {
    font-size: 0.625rem;
    font-weight: 600;
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    background: var(--gray-800);
    color: var(--gray-400);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .agent-header-right {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .agent-btn-sm {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--gray-500);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: all var(--duration-fast) var(--ease-out);
  }

  .agent-btn-sm:hover {
    color: var(--white);
    background: var(--glass-bg-hover);
  }

  /* ═══════════════════════════════════════════════════════════════════════
     BODY / MESSAGES
     ═══════════════════════════════════════════════════════════════════════ */
  .agent-body {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .agent-welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 1rem;
    padding: 2rem 1rem;
    height: 100%;
  }

  .welcome-icon {
    color: var(--gray-600);
  }

  .welcome-icon.loading {
    animation: icon-pulse 1.5s ease-in-out infinite;
  }

  @keyframes icon-pulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
  }

  .welcome-text {
    font-size: 0.8125rem;
    color: var(--gray-500);
    line-height: 1.5;
    max-width: 280px;
  }

  .agent-btn-load {
    padding: 0.5rem 1.25rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--white);
    background: var(--gray-800);
    border: 1px solid var(--gray-700);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--duration-fast) var(--ease-out);
  }

  .agent-btn-load:hover {
    background: var(--gray-700);
    border-color: var(--gray-600);
  }

  .progress-section {
    width: 100%;
    max-width: 240px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .progress-bar-track {
    width: 100%;
    height: 4px;
    background: var(--gray-800);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--gray-400);
    border-radius: 2px;
    transition: width 0.3s ease-out;
  }

  .progress-label {
    font-size: 0.75rem;
    color: var(--gray-500);
  }

  .error-text {
    font-size: 0.8125rem;
    color: #ef4444;
    line-height: 1.5;
  }

  .agent-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
  }

  .agent-empty p {
    font-size: 0.8125rem;
    color: var(--gray-600);
    max-width: 240px;
  }

  /* Messages */
  .msg {
    display: flex;
    flex-direction: column;
  }

  .msg-user {
    align-items: flex-end;
  }

  .msg-assistant {
    align-items: flex-start;
  }

  .msg-bubble {
    max-width: 85%;
    padding: 0.625rem 0.875rem;
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    line-height: 1.55;
    word-break: break-word;
  }

  .msg-bubble-user {
    background: var(--gray-800);
    color: var(--gray-200);
    border-bottom-right-radius: var(--radius-sm);
  }

  .msg-bubble-assistant {
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    color: var(--gray-300);
    border-bottom-left-radius: var(--radius-sm);
  }

  /* Markdown styling inside assistant bubbles */
  .msg-bubble-assistant :global(p) {
    margin: 0 0 0.5rem;
  }

  .msg-bubble-assistant :global(p:last-child) {
    margin-bottom: 0;
  }

  .msg-bubble-assistant :global(strong) {
    color: var(--white);
    font-weight: 600;
  }

  .msg-bubble-assistant :global(code) {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    padding: 0.125rem 0.25rem;
    background: var(--gray-800);
    border-radius: 3px;
  }

  .msg-bubble-assistant :global(ul),
  .msg-bubble-assistant :global(ol) {
    margin: 0.25rem 0;
    padding-left: 1.25rem;
  }

  .msg-bubble-assistant :global(li) {
    margin-bottom: 0.125rem;
  }

  .msg-speed {
    font-size: 0.7rem;
    color: var(--gray-500);
    margin-top: 0.25rem;
    margin-left: 0.25rem;
    font-family: var(--font-mono);
    opacity: 0.7;
  }

  .cursor-blink {
    animation: blink 0.8s step-end infinite;
    color: var(--gray-500);
  }

  @keyframes blink {
    50% { opacity: 0; }
  }

  /* Navigation pills */
  .nav-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
    margin-top: 0.375rem;
  }

  .nav-pill {
    padding: 0.25rem 0.625rem;
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--gray-400);
    background: var(--gray-900);
    border: 1px solid var(--gray-800);
    border-radius: 999px;
    cursor: pointer;
    transition: all var(--duration-fast) var(--ease-out);
  }

  .nav-pill:hover {
    color: var(--white);
    border-color: var(--gray-600);
    background: var(--gray-800);
  }

  /* ═══════════════════════════════════════════════════════════════════════
     INPUT AREA
     ═══════════════════════════════════════════════════════════════════════ */
  .agent-input-area {
    display: flex;
    align-items: flex-end;
    gap: 0.5rem;
    padding: 0.75rem;
    border-top: 1px solid var(--glass-border);
    flex-shrink: 0;
  }

  .agent-input {
    flex: 1;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.75rem;
    color: var(--white);
    font-family: var(--font-sans);
    font-size: 0.8125rem;
    line-height: 1.4;
    resize: none;
    min-height: 36px;
    max-height: 80px;
    transition: border-color var(--duration-fast) var(--ease-out);
  }

  .agent-input::placeholder {
    color: var(--gray-600);
  }

  .agent-input:focus {
    outline: none;
    border-color: var(--gray-500);
  }

  .agent-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .agent-send-btn {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--glass-border);
    background: var(--gray-800);
    color: var(--gray-400);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--duration-fast) var(--ease-out);
    flex-shrink: 0;
  }

  .agent-send-btn:hover:not(:disabled) {
    color: var(--white);
    background: var(--gray-700);
    border-color: var(--gray-600);
  }

  .agent-send-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
