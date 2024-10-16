<script>
  import { onMount } from "svelte";

  let papers = [];
  let loading = true;
  let error = null;

  onMount(async () => {
    try {
      const response = await fetch("/api/pwc");
      if (!response.ok) {
        throw new Error("Failed to fetch papers");
      }
      papers = await response.json();
    } catch (err) {
      error = err.message;
    } finally {
      loading = false;
    }
  });
</script>

<svelte:head>
  <title>Papers with Code</title>
</svelte:head>

<main id="container--main">
  <h1 class="section--page-text-center">Papers with Code</h1>

  {#if loading}
    <p class="section--page-text-center">Loading papers...</p>
  {:else if error}
    <p class="section--page-text-center error-message">{error}</p>
  {:else}
    <div class="papers-grid">
      {#each papers as paper}
        <div class="card--project">
          <img src={paper.image_url} alt={paper.title} class="paper-image" />
          <div class="paper-content">
            <h2>
              <a
                href={paper.paper_url}
                target="_blank"
                rel="noopener noreferrer"
              >
                {paper.title}
              </a>
            </h2>
            <p>{paper.description}</p>
            <div class="paper-meta">
              <span class="publish-date margin-right">{paper.publish_date}</span
              >
              {#if paper.github_repo}
                <a
                  href={paper.github_repo}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="margin-left"
                >
                  GitHub Repo
                </a>
              {/if}
            </div>
            <div class="paper-tags">
              {#each paper.tags as tag}
                <a
                  href={tag.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="tag"
                >
                  {tag.name}
                </a>
              {/each}
            </div>
            <div class="paper-links">
              <a
                href={paper.paper_url}
                target="_blank"
                rel="noopener noreferrer"
                class="btn btn-primary"
              >
                View Paper
              </a>
              <a
                href={paper.code_url}
                target="_blank"
                rel="noopener noreferrer"
                class="btn btn-secondary"
              >
                View Code
              </a>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</main>

<style>
  .papers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2em;
  }

  .card--project {
    background-color: var(--mainBgColor);
    border: 1px solid var(--mainBorderColor);
    border-radius: 8px;
    overflow: hidden;
    transition: transform 0.3s ease;
  }

  .card--project:hover {
    transform: translateY(-5px);
  }

  .paper-image {
    width: 100%;
    height: 200px;
    object-fit: cover;
  }

  .paper-content {
    padding: 1em;
  }

  .paper-meta {
    display: flex;
    align-items: center;
    margin-bottom: 0.5em;
  }

  .publish-date {
    font-size: 0.9em;
    color: var(--secondaryTextColor);
  }

  .paper-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5em;
    margin-bottom: 1em;
  }

  .tag {
    background-color: var(--mainBorderColor);
    color: var(--mainTextColor);
    padding: 0.2em 0.5em;
    border-radius: 4px;
    font-size: 0.9em;
  }

  .paper-links {
    display: flex;
    justify-content: space-between;
  }

  .btn {
    padding: 0.5em 1em;
    border-radius: 4px;
    text-align: center;
    transition: background-color 0.3s ease;
  }

  .btn-primary {
    background-color: var(--mainLinkColor);
    color: var(--mainBgColor);
  }

  .btn-primary:hover {
    background-color: var(--mainTextColor);
  }

  .btn-secondary {
    background-color: var(--mainBorderColor);
    color: var(--mainTextColor);
  }

  .btn-secondary:hover {
    background-color: var(--secondaryTextColor);
  }

  .error-message {
    color: #f63737;
  }

  @media (max-width: 600px) {
    .papers-grid {
      grid-template-columns: 1fr;
    }

    .paper-links {
      flex-direction: column;
      gap: 1em;
    }
  }
</style>
