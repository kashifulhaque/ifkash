<script lang="ts">
  import axios from "axios";
  import { onMount } from "svelte";
  import { page } from "$app/stores";

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
  let leetcodePayload = {
    username: "ifkash",
    count: 20,
  };

  let loadingStats = true;
  let loadingSubmissions = true;

  $: {
    const params = $page.params;
    leetcodePayload.username = params.username || "ifkash";
  }

  onMount(async () => {
    try {
      // Fetch LeetCode stats
      const statsResponse = await axios.get("/api/lc/profile", {
        params: { username: leetcodePayload.username, count: 20 },
      });
      leetcodeStats = statsResponse.data;
      loadingStats = false;

      // Fetch recent submissions
      const submissionsResponse = await axios.post(
        "/api/lc/submissions",
        leetcodePayload
      );
      recentSubmissions =
        submissionsResponse.data.data.recentAcSubmissionList || [];
      loadingSubmissions = false;
    } catch (err) {
      console.error(err);
      loadingStats = false;
      loadingSubmissions = false;
    }
  });
</script>

<div class="min-h-screen text-gray-100 space-grotesk-400 px-4 sm:px-8 py-8 bg-neutral-900">
  <section>
    <h1 class="text-2xl font-bold mb-6 flex items-center gap-2">
      <i class="fa-solid fa-list-check"></i>
      Leetcode stats
      <span class="text-sm text-gray-400">•</span>
      <a href="/" class="text-base text-blue-300 hover:underline">go back</a>
    </h1>

    {#if loadingStats}
      <p class="text-lg text-center">Loading stats...</p>
    {:else}
      <div class="flex gap-4 mb-6">
        <span
          class="inline-flex items-center gap-1 bg-green-800 px-2 py-1 rounded text-sm font-medium"
        >
          <i class="fa-solid fa-egg"></i>
          {leetcodeStats.easy.complete}/{leetcodeStats.easy.total} Easy
        </span>
        <span
          class="inline-flex items-center gap-1 bg-yellow-800 px-2 py-1 rounded text-sm font-medium"
        >
          <i class="fa-solid fa-square"></i>
          {leetcodeStats.medium.complete}/{leetcodeStats.medium.total} Medium
        </span>
        <span
          class="inline-flex items-center gap-1 bg-red-800 px-2 py-1 rounded text-sm font-medium"
        >
          <i class="fa-solid fa-person-running"></i>
          {leetcodeStats.hard.complete}/{leetcodeStats.hard.total} Hard
        </span>
      </div>
    {/if}

    {#if loadingSubmissions}
      <p class="text-lg text-center">Loading submissions...</p>
    {:else}
      <table class="w-full table-auto border-collapse">
        <thead>
          <tr>
            <th
              class="border-b border-gray-700 pb-2 text-left text-sm font-semibold"
              >Date</th
            >
            <th
              class="border-b border-gray-700 pb-2 text-left text-sm font-semibold"
              >Title</th
            >
          </tr>
        </thead>
        <tbody>
          {#if recentSubmissions.length > 0}
            {#each recentSubmissions as { id, title, titleSlug, timestamp }}
              <tr class="hover:bg-gray-800">
                <td class="py-2 text-sm text-gray-300">
                  {new Date(parseInt(timestamp) * 1000).toLocaleDateString(
                    "en-US",
                    {
                      year: "numeric",
                      month: "short",
                      day: "numeric",
                    }
                  )}
                </td>
                <td class="py-2 text-sm">
                  <a
                    href={`https://leetcode.com/problems/${titleSlug}/`}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-blue-200 hover:underline"
                  >
                    {title}
                  </a>
                </td>
              </tr>
            {/each}
          {:else}
            <tr>
              <td colspan="2" class="py-2 text-center text-sm">
                No recent submissions found.
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    {/if}
  </section>
</div>
