<script lang="ts">
  import { marked } from "marked";
  import { onMount } from "svelte";
  import DOMPurify from "dompurify";

  let messages: { role: string; content: string }[] = [
    {
      role: "system",
      content:
        "You are a helpful assistant created by Kashiful Haque (aka Kashif). Kashif works as an Associate Software Engineer at Fiery (an Epson Company) and he has 2 years of experience across various fields like Backend Development, Data Science and Machine Learning (with focus on NLP and LLMs)",
    },
  ];
  let userMessage = "";
  let loading = false;
  let chatContainer: HTMLElement;

  const API_URL = "/llm";
  const MAX_MESSAGES = 30;

  onMount(() => {
    scrollToBottom();
  });

  async function sendMessage() {
    if (userMessage.trim() === "") return;

    const userContent = userMessage.trim();
    addMessage({ role: "user", content: userContent });
    userMessage = "";
    loading = true;

    try {
      const response = await fetch(API_URL, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model: "@cf/meta/llama-3.2-3b-instruct",
          messages: messages,
        }),
      });

      const data = await response.json();
      const assistantMessage = data.choices[0]?.message?.content;
      if (assistantMessage) {
        addMessage({ role: "assistant", content: assistantMessage });
      }
    } catch (error) {
      console.error("Error:", error);
    } finally {
      loading = false;
    }
  }

  function addMessage(message: { role: string; content: string }) {
    messages = [...messages, message];
    if (messages.length > MAX_MESSAGES + 1) {
      messages = [messages[0], ...messages.slice(-MAX_MESSAGES)];
    }
    scrollToBottom();
  }

  function scrollToBottom() {
    setTimeout(() => {
      if (chatContainer) {
        chatContainer.scrollTop = chatContainer.scrollHeight;
      }
    }, 0);
  }

  function parseMarkdown(content: string) {
    return DOMPurify.sanitize(marked(content));
  }

  $: displayMessages = messages.filter((message) => message.role !== "system");
</script>

<div class="chat-container">
  <div class="messages" bind:this={chatContainer}>
    {#each displayMessages as message}
      <div class="message {message.role}">
        <strong>{message.role === "user" ? "You" : "Assistant"}:</strong>

        {#if message.role === "assistant"}
          <div class="assistant">{@html parseMarkdown(message.content)}</div>
        {:else}
          <p>{message.content}</p>
        {/if}
      </div>
    {/each}
  </div>

  <div class="input-area">
    <input
      type="text"
      bind:value={userMessage}
      placeholder="Type your message ..."
      on:keypress={(e) => e.key === "Enter" && sendMessage()}
      disabled={loading}
    />

    <button
      on:click={sendMessage}
      disabled={loading || userMessage.trim() === ""}
    >
      Send
    </button>
  </div>

  {#if loading}
    <div class="loading">Thinking ...</div>
  {/if}
</div>

<style>
  .chat-container {
    max-width: 800px;
    margin: 0 auto;
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 1rem;
    box-sizing: border-box;
  }
  .messages {
    flex-grow: 1;
    overflow-y: auto;
    padding-bottom: 1rem;
    margin-bottom: 1rem;
    border: 1px solid #eaeaea;
    border-radius: 8px;
    padding: 1rem;
  }
  .message {
    margin-bottom: 0.5rem;
    padding: 0.5rem;
    border-radius: 8px;
  }
  .user {
    background-color: #f0f0f0;
    text-align: right;
    color: #333;
  }
  .assistant {
    background-color: #e6f3ff;
    text-align: left;
    color: #0070f3;
  }
  .input-area {
    display: flex;
    margin-top: auto;
  }
  input {
    flex-grow: 1;
    padding: 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    margin-right: 0.5rem;
    color: black;
  }
  button {
    padding: 0.5rem 1rem;
    background-color: #0070f3;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  button:disabled {
    background-color: #ccc;
    color: gray;
    cursor: not-allowed;
  }
  .loading {
    text-align: center;
    margin-top: 1rem;
  }

  @media (max-width: 600px) {
    .chat-container {
      padding: 0.5rem;
    }
    .messages {
      padding: 0.5rem;
    }
    .input-area {
      flex-direction: column;
    }
    input {
      margin-right: 0;
      margin-bottom: 0.5rem;
    }
  }
</style>
