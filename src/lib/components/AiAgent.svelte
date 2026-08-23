<script lang="ts">
  import { onMount, tick } from "svelte";
  import { goto } from "$app/navigation";
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import { NAVIGATION_MAP } from "$lib/agent/context";
  import type {
    ChatMessage,
    AgentState,
    WorkerToMainMessage,
    NavigationSuggestion,
  } from "$lib/agent/types";

  let isOpen = false;
  let agentState: AgentState = { status: "idle" };
  let messages: ChatMessage[] = [];
  let inputText = "";
  let currentStreamingText = "";
  let messagesContainer: HTMLDivElement;
  let inputEl: HTMLTextAreaElement;
  let worker: Worker | null = null;

  let navSuggestions: Map<number, NavigationSuggestion[]> = new Map();

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
    worker = new Worker(new URL("$lib/agent/worker.ts", import.meta.url), {
      type: "module",
    });
    worker.onmessage = handleWorkerMessage;
  }

  function handleWorkerMessage(event: MessageEvent<WorkerToMainMessage>) {
    const msg = event.data;

    switch (msg.type) {
      case "progress":
        agentState = {
          status: msg.status === "downloading" ? "downloading" : "loading",
          progress: msg.progress,
        };
        break;

      case "ready":
        agentState = { status: "ready", backend: msg.backend };
        break;

      case "token":
        currentStreamingText += msg.data;
        scrollToBottom();
        break;

      case "done": {
        let finalText = currentStreamingText || msg.content;
        finalText = finalText.replace(/<think>[\s\S]*?<\/think>/g, "").trim();
        messages = [
          ...messages,
          { role: "assistant", content: finalText, speed: msg.speed },
        ];
        const idx = messages.length - 1;
        const suggestions = parseNavigationSuggestions(finalText);
        if (suggestions.length > 0) {
          navSuggestions = new Map(navSuggestions);
          navSuggestions.set(idx, suggestions);
        }
        currentStreamingText = "";
        agentState = { ...agentState, status: "ready" };
        scrollToBottom();
        break;
      }

      case "unloaded":
        agentState = { status: "idle" };
        messages = [];
        navSuggestions = new Map();
        currentStreamingText = "";
        break;

      case "status":
        agentState = { ...agentState, status: msg.state };
        break;

      case "error":
        agentState = { status: "error", error: msg.error };
        currentStreamingText = "";
        break;
    }
  }

  function loadModel() {
    initWorker();
    worker?.postMessage({ type: "load" });
  }

  function unloadModel() {
    worker?.postMessage({ type: "unload" });
    worker = null;
  }

  function sendMessage() {
    const text = inputText.trim();
    if (!text || agentState.status !== "ready") return;

    messages = [...messages, { role: "user", content: text }];
    inputText = "";
    currentStreamingText = "";
    agentState = { ...agentState, status: "generating" };
    scrollToBottom();

    const conversationHistory = messages.filter((m) => m.role !== "system");
    worker?.postMessage({ type: "generate", messages: conversationHistory });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
    if (e.key === "Escape") {
      closePanel();
    }
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && isOpen) {
      closePanel();
    }
  }

  function parseNavigationSuggestions(text: string): NavigationSuggestion[] {
    const suggestions: NavigationSuggestion[] = [];
    let match;

    const regex = new RegExp(NAV_REGEX.source, "g");
    while ((match = regex.exec(text)) !== null) {
      const path = match[1].trim();
      const label = getLabelForPath(path);
      suggestions.push({ label, path });
    }

    if (suggestions.length === 0) {
      const lowerText = text.toLowerCase();
      for (const [keyword, path] of Object.entries(NAVIGATION_MAP)) {
        if (
          lowerText.includes(keyword.toLowerCase()) &&
          !suggestions.some((s) => s.path === path)
        ) {
          if (keyword.length >= 3) {
            suggestions.push({ label: getLabelForPath(path), path });
          }
        }
      }
      suggestions.splice(3);
    }

    return suggestions;
  }

  function getLabelForPath(path: string): string {
    const labels: Record<string, string> = {
      "/": "Home",
      "/work": "Work",
      "/projects": "Projects",
      "/projects/polite-orpo": "PoliteLlama",
      "/projects/banana-cpp": "banana.cpp",
      "/projects/smol-llama": "smol-llama",
      "/projects/vicharak": "Vicharak Micro-Llama",
      "/projects/smoltorch": "smoltorch",
      "/projects/nopokedb": "NoPokeDB",
      "/projects/boo": "Boo",
      "/projects/ferray": "ferray",
      "/rl": "Reinforcement learning",
      "/rl/snake-dqn": "Snake DQN",
      "/tools": "Tools",
      "/tools/pdf-annotator": "PDF Annotator",
      "/tools/splitter": "Expense Splitter",
      "/education": "Education",
      "/blog": "Blog",
      "/fitness": "Fitness",
      "/fitness/workout": "Workout",
      "/game": "Game",
      "/api/cv?format=view": "View resume",
    };
    return labels[path] || path;
  }

  function stripNavigationMarkers(text: string): string {
    return text.replace(NAV_REGEX, "").trim();
  }

  function renderMarkdown(text: string): string {
    let cleaned = stripNavigationMarkers(text);
    cleaned = cleaned.replace(/<think>[\s\S]*?<\/think>/g, "").trim();
    cleaned = cleaned.replace(/<think>[\s\S]*$/g, "").trim();
    const html = marked.parse(cleaned, { async: false }) as string;
    return DOMPurify.sanitize(html);
  }

  function navigateTo(path: string) {
    if (path.startsWith("/api/")) {
      window.open(path, "_blank");
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
      case "idle":
        return "Load the local model first";
      case "downloading":
        return "Downloading the model";
      case "loading":
        return "Loading the model";
      case "generating":
        return "Writing a response";
      case "error":
        return "The model isn't available";
      case "ready":
        return "Ask about work or projects";
      default:
        return "Ask about work or projects";
    }
  }

  $: canSend = agentState.status === "ready" && inputText.trim().length > 0;
  $: isModelLoaded = ["ready", "generating"].includes(agentState.status);
  $: isLoading = ["downloading", "loading"].includes(agentState.status);

  onMount(() => {
    window.addEventListener("keydown", handleGlobalKeydown);

    return () => {
      window.removeEventListener("keydown", handleGlobalKeydown);
      if (worker) {
        worker.terminate();
        worker = null;
      }
    };
  });
</script>

<button
  class="agent-fab"
  class:active={isOpen}
  on:click={togglePanel}
  aria-label={isOpen ? "Close site guide" : "Open site guide"}
  aria-expanded={isOpen}
  aria-controls="site-guide"
>
  {#if isOpen}
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <line x1="18" y1="6" x2="6" y2="18"></line>
      <line x1="6" y1="6" x2="18" y2="18"></line>
    </svg>
  {:else}
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 8V4H8"></path>
      <rect width="16" height="12" x="4" y="8" rx="2"></rect>
      <path d="M2 14h2M20 14h2M15 13v2M9 13v2"></path>
    </svg>
  {/if}
  <span>{isOpen ? "Close" : "Ask"}</span>
</button>

{#if isOpen}
  <div id="site-guide" class="agent-panel" role="dialog" aria-label="Site guide">
    <div class="agent-header">
      <div class="agent-header-left">
        <span class="agent-title">Site guide</span>
        {#if agentState.backend}
          <span class="agent-badge">{agentState.backend === "webgpu" ? "WebGPU" : "WASM"}</span>
        {/if}
      </div>
      <div class="agent-header-right">
        {#if isModelLoaded}
          <button
            type="button"
            class="agent-btn-sm"
            on:click={unloadModel}
            title="Unload model"
            aria-label="Unload model"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M18.36 6.64a9 9 0 1 1-12.73 0"></path>
              <line x1="12" y1="2" x2="12" y2="12"></line>
            </svg>
          </button>
        {/if}
        <button
          type="button"
          class="agent-btn-sm"
          on:click={closePanel}
          title="Close"
          aria-label="Close site guide"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>

    <div class="agent-body" bind:this={messagesContainer}>
      {#if agentState.status === "idle"}
        <div class="agent-welcome">
          <p class="welcome-kicker">Runs on your device</p>
          <h2>Ask about Kashif</h2>
          <p class="welcome-text">
            Load a 300 MB local model to ask about my work, projects, and
            experience. Your messages stay in this browser.
          </p>
          <button class="agent-btn-load" on:click={loadModel}
            >Load local model</button
          >
        </div>
      {:else if isLoading}
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
            <LoadingState
              label={agentState.status === "downloading"
                ? `Downloading model ${agentState.progress ?? 0}%`
                : "Initializing model"}
            />
            {#if agentState.status === "downloading"}
              <div class="progress-bar-track">
                <div class="progress-bar-fill" style="width: {agentState.progress ?? 0}%"></div>
              </div>
            {/if}
          </div>
        </div>
      {:else if agentState.status === "error"}
        <div class="agent-welcome">
          <p class="error-text">{agentState.error ?? "Something went wrong."}</p>
          <button class="agent-btn-load" on:click={loadModel}>Retry</button>
        </div>
      {:else}
        {#if messages.length === 0 && !currentStreamingText}
          <div class="agent-empty">
            <p class="welcome-kicker">Local model ready</p>
            <h2>What would you like to know?</h2>
            <p>Ask about Kashif's work, projects, education, or skills.</p>
          </div>
        {/if}

        {#each messages as message, i}
          <div class="msg" class:msg-user={message.role === "user"} class:msg-assistant={message.role === "assistant"}>
            {#if message.role === "user"}
              <div class="msg-bubble msg-bubble-user">{message.content}</div>
            {:else}
              <div class="msg-bubble msg-bubble-assistant">
                {@html renderMarkdown(message.content)}
              </div>
              {#if message.speed}
                <div class="msg-speed">{message.speed}</div>
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
        aria-label="Send message"
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
  .agent-fab {
    position: fixed;
    right: 24px;
    bottom: 24px;
    z-index: 70;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    height: 40px;
    padding: 0 12px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    background: var(--agent-bg);
    color: var(--ink-2);
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    box-shadow: 0 8px 28px var(--agent-shadow);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    transition:
      color 160ms var(--ease-inout),
      border-color 160ms var(--ease-inout),
      background-color 160ms var(--ease-inout);
  }

  .agent-fab:hover,
  .agent-fab.active {
    border-color: var(--foreground);
    background: var(--background);
    color: var(--foreground);
  }

  .agent-fab svg,
  .agent-btn-sm svg {
    width: 15px;
    height: 15px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .agent-panel {
    position: fixed;
    right: 24px;
    bottom: 76px;
    z-index: 70;
    display: flex;
    flex-direction: column;
    width: 400px;
    height: min(540px, calc(100vh - 112px));
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--agent-bg);
    color: var(--foreground);
    box-shadow: 0 20px 60px var(--agent-shadow);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    animation: panel-enter 160ms var(--ease-out);
  }

  @keyframes panel-enter {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .agent-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex: 0 0 auto;
    min-height: 56px;
    padding: 12px 14px 12px 18px;
    border-bottom: 1px solid var(--border);
  }

  .agent-header-left,
  .agent-header-right {
    display: flex;
    align-items: center;
  }

  .agent-header-left {
    gap: 8px;
  }

  .agent-header-right {
    gap: 2px;
  }

  .agent-title {
    color: var(--foreground);
    font-size: 14px;
    font-weight: 600;
  }

  .agent-badge {
    color: var(--ink-3);
    font-size: 11px;
    font-weight: 500;
  }

  .agent-btn-sm {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-3);
    transition:
      color 160ms var(--ease-inout),
      background-color 160ms var(--ease-inout);
  }

  .agent-btn-sm:hover {
    background: var(--hover);
    color: var(--foreground);
  }

  .agent-body {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
    padding: 18px;
    overflow-y: auto;
  }

  .agent-welcome,
  .agent-empty {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    min-height: 100%;
    text-align: left;
  }

  .agent-welcome {
    gap: 0;
    padding: 24px 6px;
  }

  .agent-empty {
    padding: 24px 6px;
  }

  .welcome-kicker {
    margin: 0 0 8px;
    color: var(--ink-3);
    font-size: 12px;
    font-weight: 500;
  }

  .agent-welcome h2,
  .agent-empty h2 {
    margin: 0;
    color: var(--foreground);
    font-size: 24px;
    font-weight: 600;
    line-height: 1.15;
    letter-spacing: -0.025em;
  }

  .welcome-text,
  .agent-empty > p:last-child {
    max-width: 34ch;
    margin: 14px 0 0;
    color: var(--ink-2);
    font-size: 14px;
    line-height: 1.55;
  }

  .welcome-icon {
    margin-bottom: 16px;
    color: var(--ink-3);
  }

  .welcome-icon.loading {
    animation: icon-pulse 1.5s ease-in-out infinite;
  }

  @keyframes icon-pulse {
    0%,
    100% {
      opacity: 0.45;
    }
    50% {
      opacity: 1;
    }
  }

  .agent-btn-load {
    min-height: 40px;
    margin-top: 24px;
    padding: 9px 15px;
    border: 1px solid var(--foreground);
    border-radius: var(--radius-sm);
    background: var(--foreground);
    color: var(--background);
    font-size: 13px;
    font-weight: 500;
    transition:
      background-color 160ms var(--ease-inout),
      border-color 160ms var(--ease-inout);
  }

  .agent-btn-load:hover {
    border-color: var(--accent-2);
    background: var(--accent-2);
  }

  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    max-width: 280px;
  }

  .progress-bar-track {
    width: 100%;
    height: 2px;
    overflow: hidden;
    background: var(--pressed);
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--foreground);
    transition: width 300ms ease-out;
  }

  .error-text {
    margin: 0;
    color: var(--destructive);
    font-size: 13px;
    line-height: 1.5;
  }

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
    max-width: 88%;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    line-height: 1.55;
    overflow-wrap: anywhere;
  }

  .msg-bubble-user {
    background: var(--pressed);
    color: var(--foreground);
  }

  .msg-bubble-assistant {
    max-width: 100%;
    padding: 2px 0;
    background: transparent;
    color: var(--ink-2);
  }

  .msg-bubble-assistant :global(p) {
    margin: 0 0 8px;
    color: inherit;
    font-size: inherit;
    line-height: inherit;
  }

  .msg-bubble-assistant :global(p:last-child) {
    margin-bottom: 0;
  }

  .msg-bubble-assistant :global(strong) {
    color: var(--foreground);
    font-weight: 600;
  }

  .msg-bubble-assistant :global(code) {
    padding: 1px 4px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--popover);
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .msg-bubble-assistant :global(ul),
  .msg-bubble-assistant :global(ol) {
    margin: 6px 0;
    padding-left: 18px;
  }

  .msg-bubble-assistant :global(li) {
    margin-bottom: 2px;
  }

  .msg-speed {
    margin: 4px 0 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .cursor-blink {
    color: var(--ink-3);
    animation: blink 800ms step-end infinite;
  }

  @keyframes blink {
    50% {
      opacity: 0;
    }
  }

  .nav-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }

  .nav-pill {
    min-height: 30px;
    padding: 5px 9px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    font-size: 12px;
    font-weight: 500;
    transition:
      color 160ms var(--ease-inout),
      border-color 160ms var(--ease-inout),
      background-color 160ms var(--ease-inout);
  }

  .nav-pill:hover {
    border-color: var(--border-strong);
    background: var(--hover);
    color: var(--foreground);
  }

  .agent-input-area {
    display: flex;
    align-items: flex-end;
    flex: 0 0 auto;
    gap: 8px;
    padding: 14px;
    border-top: 1px solid var(--border);
  }

  .agent-input {
    flex: 1;
    min-height: 40px;
    max-height: 88px;
    padding: 9px 11px;
    resize: none;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: var(--popover);
    color: var(--foreground);
    font-family: var(--font-sans);
    font-size: 13px;
    line-height: 20px;
    transition: border-color 160ms var(--ease-inout);
  }

  .agent-input::placeholder {
    color: var(--ink-3);
  }

  .agent-input:focus {
    border-color: var(--border-strong);
    outline: none;
  }

  .agent-input:disabled {
    cursor: not-allowed;
  }

  .agent-send-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    width: 40px;
    height: 40px;
    padding: 0;
    border: 1px solid var(--foreground);
    border-radius: var(--radius-sm);
    background: var(--foreground);
    color: var(--background);
    transition:
      opacity 160ms var(--ease-inout),
      background-color 160ms var(--ease-inout);
  }

  .agent-send-btn:hover:not(:disabled) {
    background: var(--accent-2);
  }

  .agent-send-btn:disabled {
    cursor: not-allowed;
    opacity: 0.28;
  }

  @media (max-width: 560px) {
    .agent-fab {
      right: 16px;
      bottom: 16px;
    }

    .agent-panel {
      right: 16px;
      bottom: 68px;
      left: 16px;
      width: auto;
      height: min(560px, calc(100dvh - 96px));
    }
  }
</style>
