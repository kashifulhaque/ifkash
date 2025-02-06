<script lang="ts">
  import { marked } from "marked";
  import { onMount } from "svelte";
  import DOMPurify from "dompurify";

  const systemPrompt = `
You are Kashiful Haque's knowledgeable assistant with your own identity. You have detailed knowledge about Kashiful Haque (aka Kashif), a Software Engineer with over two years of experience specializing in Backend Development, Data Science, AI, and Machine Learning. Kashiful has worked on various advanced technologies such as Natural Language Processing (NLP), Generative AI (GenAI), and Large Language Models (LLMs).

Kashiful's professional experience includes:
  - He has over 2 years of professional work experience.
  - Currently working as an Associate Software Engineer at Fiery (an Epson company), where he led the development of 'Beacon,' an NLP-driven print processing system using LLMs. He has also worked on building FieryGPT, a RAG-based solution using LLaMa 3.1 and trained LoRA adapters for enhanced question-answering capabilities.
  - As a Data Scientist Intern at Electronics for Imaging, Kashiful developed cost-effective image processing techniques for product mockups, leveraging tools like ImageMagick, Node.js, and Python.
  - As a Fullstack Developer Intern at Corteva Agriscience, he contributed to the migration and refactoring of a Flask monolith to scalable APIs, optimizing AutoML workflows and job triggering on Kubernetes.

Kashiful's socials:
  - LinkedIn: https://www.linkedin.com/in/kashifulhaque
  - Blog: https://blog.ifkash.dev
  - GitHub: https://github.com/kashifulhaque
  - Resume: https://ifkash.dev/assets/Kashiful_Haque.pdf
  - Email: haque.kashiful7@gmail.com
  - Leetcode: https://leetcode.com/u/ifkash

Kashiful also holds a Bachelor's degree in Data Science from IIT Madras and has worked on open-source projects available on GitHub.

**Guardrails**:
  - While you are well-versed in Kashiful's professional background and expertise, you should only mention his details when users specifically ask about him or his work.
  - In other conversations, focus on providing helpful and informative responses based on the user's inquiries, without automatically referring to Kashiful's background.
  - Maintain a friendly and professional tone, and respect user privacy, only sharing publicly available information about Kashiful.
      `;
  let messages: { role: string; content: string }[] = [
    {
      role: "system",
      content: systemPrompt,
    },
  ];
  let userMessage = "";
  let loading = false;
  let chatContainer: HTMLElement;

  const API_URL = "/api/llm";
  const MAX_MESSAGES = 30;
  const STORAGE_KEY = "chatMessages";

  onMount(() => {
    loadMessages();
    scrollToBottom();
  });

  function loadMessages() {
    const storedMessages = localStorage.getItem(STORAGE_KEY);
    if (storedMessages) {
      messages = JSON.parse(storedMessages);
      if (messages[0].role !== "system") {
        messages = [{ role: "system", content: systemPrompt }, ...messages];
      }
    }
  }

  function saveMessages() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(messages));
  }

  function clearChat() {
    messages = [{ role: "system", content: systemPrompt }];
    localStorage.removeItem(STORAGE_KEY);
  }

  async function sendMessage() {
    if (userMessage.trim() === "") return;
    const userContent = userMessage.trim();
    addMessage({ role: "user", content: userContent });
    userMessage = "";
    loading = true;

    try {
      const response = await fetch(API_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          model: "@cf/meta/llama-3.2-3b-instruct",
          messages: messages.slice(-MAX_MESSAGES),
        }),
      });
      const data = await response.json();
      const assistantMessage = data.choices[0]?.message?.content;
      if (assistantMessage) {
        addMessage({ role: "assistant", content: assistantMessage });
        saveMessages();
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

<div class="max-w-3xl mx-auto h-screen flex flex-col p-2 sm:p-4 bg-neutral-900 text-gray-100 space-grotesk-400">
  <!-- Messages Container -->
  <div
    class="flex-grow overflow-y-auto p-2 sm:p-4 flex flex-col"
    bind:this={chatContainer}
  >
    {#each displayMessages as message}
      <div
        class="mb-2 p-2 rounded-lg {message.role === 'user'
          ? 'text-right text-gray-100'
          : 'text-left text-blue-300'}"
      >
        <strong>{message.role === "user" ? "You" : "Assistant"}:</strong>
        {#if message.role === "assistant"}
          <div>{@html parseMarkdown(message.content)}</div>
        {:else}
          <p>{message.content}</p>
        {/if}
      </div>
    {/each}

    {#if loading}
      <div class="transform scale-[0.85] self-center mt-4">
        <img
          alt="Waiting for bot to answer"
          src="images/thinking.gif"
          class="h-14"
        />
      </div>
    {/if}
  </div>

  <!-- Input Area -->
  <div class="flex flex-col sm:flex-row mt-auto">
    <input
      type="text"
      bind:value={userMessage}
      placeholder="Type your message ..."
      on:keypress={(e) => e.key === "Enter" && sendMessage()}
      disabled={loading}
      class="flex-grow p-2 border border-gray-600 rounded mr-0 sm:mr-2 mb-2 sm:mb-0 bg-gray-800 text-gray-100"
    />

    <!-- Send message button -->
    <button
      on:click={sendMessage}
      disabled={loading || userMessage.trim() === ""}
      class="px-4 py-2 bg-teal-700 hover:bg-teal-800 transition-colors text-white rounded disabled:bg-gray-700 disabled:cursor-not-allowed cursor-pointer mb-2 sm:mb-0"
    >
      Send
    </button>

    <!-- Clear convo button -->
    <button
      on:click={clearChat}
      disabled={loading || displayMessages.length === 0}
      class="px-4 py-2 bg-red-500 text-white rounded disabled:bg-gray-600 disabled:cursor-not-allowed cursor-pointer ml-0 sm:ml-2 mb-2 sm:mb-0"
    >
      <i class="fa-solid fa-trash"></i>
    </button>
  </div>
</div>
