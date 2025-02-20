<script lang="ts">
  import { onMount } from "svelte";

  // Name translations
  const nameTranslations = [
    { lang: "English", text: "Kashiful Haque" },
    { lang: "Japanese", text: "カシフル・ハック" },
    { lang: "Hindi", text: "कशिफुल हक़" },
    { lang: "Arabic", text: "كاشف الحق" },
    { lang: "Kannada", text: "ಕಶಿಫುಲ್ ಹಕ್" },
    { lang: "Thai", text: "คาชิฟุล ฮัก" },
    { lang: "Korean", text: "카시풀 하크" },
  ];

  let currentNameIndex = 0;
  let name = nameTranslations[currentNameIndex].text;
  let longPressTimer: number | null = null;
  let isLongPressing = false;

  // Page routes and social links
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
  ];

  onMount(() => {
    // Rotate name every 2 seconds
    const nameRotationInterval = setInterval(() => {
      currentNameIndex = (currentNameIndex + 1) % nameTranslations.length;
      name = nameTranslations[currentNameIndex].text;
    }, 2500);

    return () => {
      clearInterval(nameRotationInterval);
    };
  });

  // Long press detection handlers
  function handleMouseDown() {
    isLongPressing = false;
    longPressTimer = window.setTimeout(() => {
      isLongPressing = true;
      window.open("./assets/Kashiful_Haque-dark.pdf", "_blank");
    }, 2000);
  }

  function handleMouseUp() {
    if (longPressTimer) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
    // Only navigate to regular PDF if not long-pressed
    if (!isLongPressing) {
      window.open("./assets/Kashiful_Haque.pdf", "_blank");
    }
    isLongPressing = false;
  }

  function handleMouseLeave() {
    if (longPressTimer) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
    isLongPressing = false;
  }
</script>

<svelte:head>
  <title>{name} • Portfolio</title>
</svelte:head>

<div
  class="min-h-screen bg-neutral-900 text-gray-100 space-grotesk-400 px-4 sm:px-8 py-8"
>
  <!-- Hero Section -->
  <section class="flex flex-col items-center text-center mb-8">
    <img
      src="images/sekiro.jpg"
      alt="Sekiro holding Kusabimaru"
      title="Sekiro holding Kusabimaru"
      class="w-32 h-32 rounded-full object-cover border-2 border-gray-700 mb-4"
    />
    <h1
      class="text-3xl font-bold space-grotesk-700 transition-opacity duration-500"
    >
      {name}
    </h1>
    <p class="text-md text-gray-400 mt-2">
      <a
        href="mailto:haque.kashiful7@gmail.com"
        target="_blank"
        rel="noopener noreferrer"
        class="text-blue-300 hover:underline"
      >
        haque.kashiful7@gmail.com
      </a>
      <span class="mx-2">•</span>
      <a
        href="mailto:me@ifkash.dev"
        target="_blank"
        rel="noopener noreferrer"
        class="text-blue-300 hover:underline"
      >
        me@ifkash.dev
      </a>
    </p>

    <!-- Download Resume button with long press detection -->
    <button
      on:mousedown={handleMouseDown}
      on:mouseup={handleMouseUp}
      on:mouseleave={handleMouseLeave}
      on:touchstart={handleMouseDown}
      on:touchend={handleMouseUp}
      on:touchcancel={handleMouseLeave}
      class="mt-4 inline-flex items-center px-4 py-2 bg-teal-400 hover:bg-teal-500 transition-colors rounded text-neutral-900 text-md font-semibold cursor-pointer relative overflow-hidden"
    >
      <i class="fa-solid fa-download mr-2"></i>
      Download Resume
      <div
        class="absolute bottom-0 left-0 h-1 bg-blue-500 transition-all duration-300"
        style={`width: ${isLongPressing ? "100%" : "0%"}`}
      ></div>
    </button>
    <div class="text-xs text-gray-500 mt-1">
      Hold for 2 seconds for dark mode version
    </div>
  </section>

  <!-- Social Links -->
  <section class="mb-8">
    <div class="flex justify-center gap-6">
      {#each socialLinks as { href, icon, text }}
        <a
          {href}
          target="_blank"
          rel="noopener noreferrer"
          class="flex items-center gap-2 hover:text-blue-400 transition-colors text-blue-100"
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
          class="px-3 py-1 border border-teal-700 rounded transition-colors hover:bg-teal-800 text-md"
        >
          {text}
        </a>
      {/each}
    </div>
  </section>

  <!-- Summary -->
  <section class="mb-8 max-w-2xl mx-auto">
    <p class="text-base text-teal-100 text-center leading-relaxed">
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
      class="flex flex-wrap items-center justify-center gap-2 text-blue-200 text-sm"
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
