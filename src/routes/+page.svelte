<script lang="ts">
  import axios from "axios";
  import { onMount } from "svelte";

  let title: string = "Kashiful Haque";
  let recentSubmissions: Array<{
    id: string;
    title: string;
    titleSlug: string;
    timestamp: string;
  }> = [];
  let leetcodeStats = {
    easy: { complete: 0, total: 0 },
    medium: { complete: 0, total: 0 },
    hard: { complete: 0, total: 0 },
    total: { complete: 0, total: 0 },
  };

  /// Fetches profile and submissions data
  onMount(async () => {
    try {
      // Fetch leetcode stats
      const statsResponse = await axios.get("/api/lc/profile");
      leetcodeStats = statsResponse.data;

      // Fetch recent submissions
      const submissionsResponse = await axios.post("/api/lc/submissions", {
        username: "ifkash",
        count: 5,
      });
      recentSubmissions =
        submissionsResponse.data.data.recentAcSubmissionList || [];
    } catch (err) {
      console.error(err);
    }
  });

  const pageRoutes = [
    {
      href: "/work",
      text: "Work Experience",
    },
    {
      href: "/education",
      text: "Education",
    },
    {
      href: "/projects",
      text: "Projects",
    },
    {
      href: "/tech",
      text: "Tech stack",
    },
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

<div id="container--main">
  <section id="wrapper--hero" class="section--page">
    <img
      id="profile-pic"
      src="images/gojo.webp"
      alt="Gojo Satoru in Shibuya"
      title="Gojo Satoru in Shibuya • Source: @Deltanpopo on X (formerly Twitter)"
    />

    <div>
      <h1 id="user-name">{title}</h1>
      <p>
        <a
          href="mailto:haque.kashiful7@gmail.com"
          target="_blank"
          rel="noopener noreferrer"
          class="email"
        >
          haque.kashiful7@gmail.com</a
        >
        •
        <a
          href="mailto:me@ifkash.dev"
          target="_blank"
          rel="noopener noreferrer"
          class="email">me@ifkash.dev</a
        >
      </p>
      <p>
        <a class="resume" href="./assets/Kashiful_Haque.pdf" target="_blank"
          ><i class="fa-solid fa-download"></i> Download Resume
        </a>
      </p>
    </div>
  </section>

  <section class="section--page">
    <div id="socials--list">
      {#each socialLinks as { href, icon, text }}
        <a {href} target="_blank" rel="noopener noreferrer" class="profiles">
          <i class={icon}></i>
          {text}
        </a>
      {/each}
    </div>
  </section>

  <section class="section--page">
    <h2><i class="fa-solid fa-bolt"></i> SUMMARY</h2>
    <p>
      I work at the intersection of Software Engineering, Data Science & Machine
      Learning focussing on Natural Language Processing (NLP), Generative AI
      (GenAI) and Large Language Models (LLMs). I also have a keen interest in
      cloud, linux and other tools surrounding tech.
      <br />
      <br />
      Born and brought up in Kolkata, I lived most of my life there. Since 2022,
      I have travelled to cities such as Hyderabad, Chennai and Bengaluru (current)
      for work/studies.
    </p>
  </section>

  <section class="section--page">
    <div id="socials--list">
      {#each pageRoutes as { href, text }}
        <a {href} class="profiles">
          {text}
        </a>
      {/each}
    </div>
  </section>

  <a href="./assets/Kashiful_Haque-dark.pdf" target="_blank"
    ><hr class="center-ball" style="visibility: hidden;" /></a
  >

  <section class="section--page">
    <h2>Recent LeetCode Submissions</h2>
    <table class="submissions-table">
      <thead>
        <tr>
          <th>Title</th>
          <th>Timestamp</th>
          <th>Link</th>
        </tr>
      </thead>
      <tbody>
        {#if recentSubmissions.length > 0}
          {#each recentSubmissions as { id, title, titleSlug, timestamp }}
            <tr>
              <td>{title}</td>
              <td>{new Date(parseInt(timestamp) * 1000).toLocaleString()}</td>
              <td>
                <a
                  href={`https://leetcode.com/problems/${titleSlug}/`}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  View Problem
                </a>
              </td>
            </tr>
          {/each}
        {:else}
          <tr>
            <td colspan="3">No recent submissions found.</td>
          </tr>
        {/if}
      </tbody>
    </table>
  </section>

  <section class="section--page section--page-text-center footer">
    <div>
      <a href="/news">hot tech news</a>
      •
      <a href="/chat">chat with me</a>
      •
      <a href="/papers">hot ML papers</a>
    </div>
    <div>
      <p>
        <span class="lc-easy"
          >{leetcodeStats.easy.complete}/{leetcodeStats.easy.total} easy</span
        >
        •
        <span class="lc-medium"
          >{leetcodeStats.medium.complete}/{leetcodeStats.medium.total} medium</span
        >
        •
        <span class="lc-hard"
          >{leetcodeStats.hard.complete}/{leetcodeStats.hard.total} hard</span
        > on leetcode
      </p>
    </div>
    <p>
      <a target="_blank" rel="noopener noreferrer" href="https://kit.svelte.dev"
        >Built with SvelteKit</a
      >
      •
      <a
        target="_blank"
        rel="noopener noreferrer"
        href="https://github.com/kashifulhaque/ifkash">Fork me on GitHub</a
      >
    </p>
  </section>
</div>

<svelte:head>
  <title>{title} • Portfolio</title>
</svelte:head>
