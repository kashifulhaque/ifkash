<script lang="ts">
  import axios from "axios";
  import { onMount } from "svelte";

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
    count: 10
  };

  /// Fetches profile and submissions data
  onMount(async () => {
    try {
      /// Fetch leetcode stats
      const statsResponse = await axios.get("/api/lc/profile");
      leetcodeStats = statsResponse.data;

      /// Fetch recent submissions
      const submissionsResponse = await axios.post("/api/lc/submissions", leetcodePayload);
      recentSubmissions = submissionsResponse.data.data.recentAcSubmissionList || [];
    } catch (err) {
      console.error(err);
    }
  });
</script>

<div id="container--main">
  <section class="section--page">
    <h1><i class="fa-solid fa-list-check"></i> Leetcode stats • <a href="/">home</a></h1>

    <!-- LeetCode Stats Bar -->
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

    <!-- LeetCode Submissions Table -->
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
  </section>
</div>
