<script lang="ts">
  let title: string = "Kashiful Haque";
  const pageRoutes = [
    { href: "/work", text: "Work Experience" },
    { href: "/education", text: "Education" },
    { href: "/projects", text: "Projects" },
    { href: "/tech", text: "Tech stack" },
  ];
  const socialLinks = [
    {
      href: "https://blog.ifkash.dev",
      icon: "fa-brands fa-hashnode",
      text: "Blog",
    },
    {
      href: "https://www.linkedin.com/in/kashifulhaque",
      icon: "fa-brands fa-linkedin",
      text: "Linkedin",
    },
    {
      href: "https://github.com/kashifulhaque",
      icon: "fa-brands fa-github",
      text: "Github",
    },
    {
      href: "https://leetcode.com/u/ifkash/",
      icon: "fa-solid fa-code",
      text: "Leetcode",
    },
    {
      href: "https://www.instagram.com/enderboi25/",
      icon: "fa-brands fa-instagram",
      text: "Instagram",
    },
  ];
</script>

<svelte:head>
  <title>{title} • Portfolio</title>
</svelte:head>

<!-- Main container with dark theme, responsive paddings, and Space Grotesk applied -->
<div
  class="min-h-screen bg-gray-900 text-gray-100 space-grotesk-400 px-4 sm:px-8 py-8"
>
  <!-- Hero Section -->
  <section class="flex flex-col items-center text-center mb-8">
    <img
      src="images/sekiro.jpg"
      alt="Sekiro holding Kusabimaru"
      title="Sekiro holding Kusabimaru"
      class="w-32 h-32 rounded-full object-cover border-2 border-gray-700 mb-4"
    />
    <h1 class="text-3xl font-bold space-grotesk-700">{title}</h1>
    <p class="text-sm text-gray-400 mt-2">
      <a
        href="mailto:haque.kashiful7@gmail.com"
        target="_blank"
        rel="noopener noreferrer"
        class="text-blue-500 hover:underline"
      >
        haque.kashiful7@gmail.com
      </a>
      <span class="mx-2">•</span>
      <a
        href="mailto:me@ifkash.dev"
        target="_blank"
        rel="noopener noreferrer"
        class="text-blue-500 hover:underline"
      >
        me@ifkash.dev
      </a>
    </p>
    <a
      href="./assets/Kashiful_Haque.pdf"
      target="_blank"
      class="mt-4 inline-flex items-center px-4 py-2 bg-blue-600 hover:bg-blue-700 transition-colors rounded text-white text-xs"
    >
      <i class="fa-solid fa-download mr-2"></i>
      Download Resume
    </a>
  </section>

  <!-- Social Links -->
  <section class="mb-8">
    <div class="flex justify-center gap-6">
      {#each socialLinks as { href, icon, text }}
        <a
          {href}
          target="_blank"
          rel="noopener noreferrer"
          class="flex items-center gap-2 hover:text-blue-500 transition-colors text-gray-200"
        >
          <i class={`${icon} text-xl`}></i>
          <!-- Hide descriptive text on extra-small screens -->
          <span class="hidden sm:inline">{text}</span>
        </a>
      {/each}
    </div>
  </section>

  <!-- Page Routes -->
  <section class="mb-8">
    <div class="flex justify-center gap-4">
      {#each pageRoutes as { href, text }}
        <a
          {href}
          class="px-3 py-1 border border-gray-700 rounded transition-colors hover:bg-gray-800 text-sm"
        >
          {text}
        </a>
      {/each}
    </div>
  </section>

  <!-- Summary -->
  <section class="mb-8 max-w-2xl mx-auto">
    <h2
      class="text-2xl font-bold flex items-center justify-center mb-4 space-grotesk-600"
    >
      <i class="fa-solid fa-bolt text-yellow-500 mr-2"></i>
      SUMMARY
    </h2>
    <p class="text-base text-gray-300 text-center leading-relaxed">
      I work at the intersection of Software Engineering, Data Science & Machine
      Learning focusing on Natural Language Processing (NLP), Generative AI
      (GenAI) and Large Language Models (LLMs). I also have a keen interest in
      cloud, linux and other tools surrounding tech.
      <br /><br />
      Born and brought up in Kolkata, I lived most of my life there. Since 2022,
      I have travelled to cities such as Hyderabad, Chennai, and Bengaluru (current)
      for work/studies.
    </p>
  </section>

  <!-- Footer Links -->
  <footer class="text-center">
    <div
      class="flex flex-wrap items-center justify-center gap-2 text-blue-500 text-xs"
    >
      <a href="/news" class="hover:underline">hot tech news</a>
      <span>•</span>
      <a href="/papers" class="hover:underline">hot ML papers</a>
      <span>•</span>
      <a href="/chat" class="hover:underline">chat with me</a>
      <span>•</span>
      <a href="/leetcode" class="hover:underline">my leetcode stats</a>
    </div>
  </footer>
</div>
