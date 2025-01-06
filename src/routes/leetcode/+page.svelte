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

  /// Fetches profile and submissions data
  onMount(async () => {
    try {
      /// Fetch leetcode stats
      const statsResponse = await axios.get("/api/lc/profile", {
        params: { username: leetcodePayload.username, count: 20 }
      });
      leetcodeStats = statsResponse.data;
      loadingStats = false;

      /// Fetch recent submissions
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

<div id="container--main">
  <section class="section--page">
    <h1>
      <i class="fa-solid fa-list-check"></i> Leetcode stats •
      <a href="/">home</a>
    </h1>

    <!-- LeetCode Stats Bar -->
    {#if loadingStats}
      <p>Loading stats...</p>
    {:else}
      <div class="lc-stats-bar">
        <span class="lc-badge easy">
          <i class="fa-solid fa-egg"></i>
          {leetcodeStats.easy.complete}/{leetcodeStats.easy.total} Easy
        </span>
        <span class="lc-badge medium">
          <i class="fa-solid fa-square"></i>
          {leetcodeStats.medium.complete}/{leetcodeStats.medium.total} Medium
        </span>
        <span class="lc-badge hard">
          <i class="fa-solid fa-person-running"></i>
          {leetcodeStats.hard.complete}/{leetcodeStats.hard.total} Hard
        </span>
      </div>
    {/if}

    <!-- LeetCode Submissions Table -->
    {#if loadingSubmissions}
      <p>Loading submissions...</p>
    {:else}
      <table class="submissions-table">
        <thead>
          <tr>
            <th>Date</th>
            <th>Title</th>
          </tr>
        </thead>
        <tbody>
          {#if recentSubmissions.length > 0}
            {#each recentSubmissions as { id, title, titleSlug, timestamp }}
              <tr>
                <td>
                  {new Date(parseInt(timestamp) * 1000).toLocaleDateString(
                    "en-US",
                    {
                      year: "numeric",
                      month: "short",
                      day: "numeric",
                    }
                  )}
                </td>
                <td class="leetcode-submission-title">
                  <a
                    href={`https://leetcode.com/problems/${titleSlug}/`}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {title}
                  </a>
                </td>
              </tr>
            {/each}
          {:else}
            <tr>
              <td colspan="2">No recent submissions found.</td>
            </tr>
          {/if}
        </tbody>
      </table>
    {/if}
  </section>
</div>
