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
    {
      href: "https://deepwiki.com/kashifulhaque/ifkash",
      icon: "fa-solid fa-book",
      text: "DeepWiki",
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
  <meta property="og:title" content="{name} • Portfolio" />
</svelte:head>

<div
  class="space-grotesk-400 min-h-screen bg-neutral-900 px-4 py-8 text-gray-100 sm:px-8"
>
  <!-- Hero Section -->
  <section class="mb-8 flex flex-col items-center text-center">
    <img
      src="images/sekiro.jpg"
      alt="Sekiro holding Kusabimaru"
      title="Sekiro holding Kusabimaru"
      class="mb-4 h-32 w-32 rounded-full border-2 border-gray-700 object-cover"
    />
    <h1
      class="space-grotesk-700 text-3xl font-bold transition-opacity duration-500"
    >
      {name}
    </h1>
    <p class="text-md mt-2 text-gray-400">
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
      class="text-md relative mt-4 inline-flex cursor-pointer items-center overflow-hidden rounded bg-teal-400 px-4 py-2 font-semibold text-neutral-900 transition-colors hover:bg-teal-500"
    >
      <i class="fa-solid fa-download mr-2"></i>
      Download Resume
      <div
        class="absolute bottom-0 left-0 h-1 bg-blue-500 transition-all duration-300"
        style={`width: ${isLongPressing ? "100%" : "0%"}`}
      ></div>
    </button>
    <div class="mt-1 text-xs text-gray-500">
      Hold for 2 seconds for dark mode version
    </div>
    <a
      href="./assets/Kashiful_Haque-dark.pdf"
      target="_blank"
      rel="noopener noreferrer"
      class="text-md mt-4 inline-flex cursor-pointer items-center rounded bg-teal-400 px-4 py-2 font-semibold text-neutral-900 transition-colors hover:bg-teal-500"
      role="button"
    >
      <i class="fa-solid fa-download mr-2"></i>
      Download Resume (Dark Mode)
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
          class="flex items-center gap-2 text-blue-100 transition-colors hover:text-blue-400"
          aria-label={text}
        >
          <i class={`${icon} text-xl`} aria-hidden="true"></i>
          <!-- Screen readers will use aria-label; visual text shown when space permits -->
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
          class="text-md rounded border border-teal-700 px-3 py-1 transition-colors hover:bg-teal-800"
        >
          {text}
        </a>
      {/each}
    </div>
  </section>

  <!-- Summary -->
  <section class="mx-auto mb-8 max-w-2xl">
    <p class="text-center text-base leading-relaxed text-teal-100">
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
      class="flex flex-wrap items-center justify-center gap-2 text-sm text-blue-200"
    >
      <a href="/dotfiles" class="hover:underline">dotfiles</a>
      <span>•</span>
      <a href="/news" class="hover:underline">tech news</a>
      <span>•</span>
      <a href="/tech-blogs" class="hover:underline">tech blogs</a>
      <span>•</span>
      <a href="/papers" class="hover:underline">ML papers</a>
      <span>•</span>
      <a href="/chat" class="hover:underline">chat</a>
      <span>•</span>
      <a href="/leetcode" class="hover:underline">leetcode stats</a>
    </div>
  </footer>
</div>
