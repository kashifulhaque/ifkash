<script lang="ts">
  let messages: { role: string; content: string }[] = [
    { role: "system", content: "You are a helpful assistant" },
  ];
  let userMessage = "";
  let loading = false;

  const API_URL = "/llm";

  async function sendMessage() {
    if (userMessage.trim() === "") return;

    const userContent = userMessage.trim();
    // Add the user's message to the conversation
    messages = [...messages, { role: "user", content: userContent }];
    userMessage = "";
    loading = true;

    try {
      // Send the user message to the API proxy
      const response = await fetch(API_URL, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          model: "@cf/meta/llama-3.2-3b-instruct",
          messages: messages, // Send all messages including the system message
        }),
      });

      const data = await response.json();
      // Add the LLM's response to the conversation
      const assistantMessage = data.choices[0]?.message?.content;
      if (assistantMessage) {
        messages = [
          ...messages,
          { role: "assistant", content: assistantMessage },
        ];
      }
    } catch (error) {
      console.error("Error:", error);
    } finally {
      loading = false;
    }
  }
</script>

<div class="chat-container">
  <div class="messages">
    {#each messages as message, index}
      <div class="message {message.role}">
        <strong>{message.role === "user" ? "You" : "Assistant"}:</strong>
        <p>{message.content}</p>
      </div>
    {/each}
  </div>

  <input
    type="text"
    bind:value={userMessage}
    placeholder="Type your message..."
    on:keypress={(e) => e.key === "Enter" && sendMessage()}
    disabled={loading}
  />

  <button
    on:click={sendMessage}
    disabled={loading || userMessage.trim() === ""}
  >
    Send
  </button>

  {#if loading}
    <div class="loading">Thinking...</div>
  {/if}
</div>

<style>
  .chat-container {
    max-width: 600px;
    margin: 2rem auto;
    border: 1px solid #eaeaea;
    padding: 1rem;
    border-radius: 8px;
  }
  .messages {
    max-height: 400px;
    overflow-y: auto;
    padding-bottom: 1rem;
  }
  .message {
    margin-bottom: 0.5rem;
  }
  .user {
    text-align: right;
    color: #333;
  }
  .assistant {
    text-align: left;
    color: #0070f3;
  }
  .loading {
    text-align: center;
    margin-top: 1rem;
  }
</style>
