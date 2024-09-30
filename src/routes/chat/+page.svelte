<script lang="ts">
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';

  let messages: { role: string; content: string }[] = [
    { role: 'system', content: 'You are a helpful assistant' }
  ];
  let userMessage = '';
  let loading = false;

  const API_URL = '/llm';

  async function sendMessage() {
    if (userMessage.trim() === '') return;

    const userContent = userMessage.trim();
    messages = [...messages, { role: 'user', content: userContent }];
    userMessage = '';
    loading = true;

    try {
      const response = await fetch(API_URL, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          model: '@cf/meta/llama-3.2-3b-instruct',
          messages: messages
        })
      });

      const data = await response.json();
      const assistantMessage = data.choices[0]?.message?.content;
      if (assistantMessage) {
        messages = [...messages, { role: 'assistant', content: assistantMessage }];
      }
    } catch (error) {
      console.error('Error:', error);
    } finally {
      loading = false;
    }
  }

  // Function to parse and sanitize markdown
  function parseMarkdown(content: string) {
    return DOMPurify.sanitize(marked(content));
  }
</script>

<div class="chat-container">
  <div class="messages">
    {#each messages as message, index}
      <div class="message {message.role}">
        <strong>{message.role === 'user' ? 'You' : 'Assistant'}:</strong>
        {#if message.role === 'assistant'}
          <!-- Render sanitized markdown content as HTML -->
          <div class="assistant" {@html}={parseMarkdown(message.content)}></div>
        {:else}
          <p>{message.content}</p>
        {/if}
      </div>
    {/each}
  </div>

  <input
    type="text"
    bind:value={userMessage}
    placeholder="Type your message..."
    on:keypress={(e) => e.key === 'Enter' && sendMessage()}
    disabled={loading}
  />

  <button on:click={sendMessage} disabled={loading || userMessage.trim() === ''}>
    Send
  </button>

  {#if loading}
    <div class="loading">Thinking...</div>
  {/if}
</div>
