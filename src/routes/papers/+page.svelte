<script lang="ts">
  import { onMount } from "svelte";

  /// Schema for API response
  interface Tag {
    name: string;
    url: string;
  }

  interface Paper {
    paper_url: string;
    code_url: string;
    image_url: string;
    title: string;
    github_repo: string;
    framework: string;
    publish_date: string;
    description: string;
    tags: Tag[] | null;
  }

  let papers: Paper[] = [];
  let loading: boolean = true;
  let error: any = null;

  /// This gets triggered when the component is mounted
  onMount(async () => {
    try {
      const response = await fetch("/api/pwc");
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();

      if (!data || !Array.isArray(data)) {
        throw new Error("Invalid data received from API");
      }

      papers = data.filter((paper) => paper && paper.title && paper.paper_url);
    } catch (err) {
      console.error("Error fetching papers:", err);
      error = err.message;
    } finally {
      loading = false;
    }
  });
</script>

<svelte:head>
  <title>Papers with Code</title>
</svelte:head>

<main id="container--papers">
  <h1 class="margin-y">
    <i class="fa-solid fa-file-contract"></i> Papers with Code •
    <a href="/">home</a>
  </h1>

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
                class="paper-title"
              >
                {paper.title}
              </a>
            </h2>
            <div class="paper-description">
              <small>{paper.description}</small>
            </div>

            <!-- <div class="paper-meta">
              <small class="publish-date margin-right"
                >{paper.publish_date}</small
              >
            </div> -->

            <div class="paper-links">
              <!-- <a
                href={paper.paper_url}
                target="_blank"
                rel="noopener noreferrer"
                class="view-paper-btn"
              >
                view paper
              </a> -->
              {#if paper.github_repo}
                <a
                  href={paper.github_repo}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <i class="fa-brands fa-github"></i> code
                </a>
              {/if}
            </div>

            {#if paper.tags && paper.tags.length > 0}
              <div class="paper-tags">
                {#each paper.tags as tag}
                  {#if tag.name.length != 0}
                    <a
                      href={tag.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="tag"
                    >
                      {tag.name}
                    </a>
                  {/if}
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</main>

<style>
  @import url("https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap");

  small {
    opacity: 0.64;
    font-size: small;
    font-style: italic;
  }

  h2 {
    margin-top: 0.5em;
    margin-bottom: 0.75em;
  }

  .view-paper-btn {
    text-decoration: none;
    color: var(--mainLinkColor);
    font-weight: 500;
    margin-right: 1em;
  }

  .paper-title {
    color: var(--mainTextColor-light);
    font-family: "Inter", sans-serif;
    font-optical-sizing: auto;
    font-weight: 400;
    font-style: normal;
  }

  .paper-description {
    margin-bottom: 0.5em;
  }

  .papers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2em;
  }

  .card--project {
    background-color: var(--mainBgColor);
    overflow: hidden;
    transition: transform 0.3s ease;
  }

  .paper-image {
    width: 100%;
    height: 200px;
    object-fit: cover;
    border-radius: 4px;
  }

  .paper-content {
    padding-top: 0em;
    padding-left: 0.25em;
    padding-right: 0.25em;
    padding-bottom: 0.25em;
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
    flex-direction: column;
    flex-wrap: wrap;
  }

  .tag {
    text-transform: lowercase;
    color: var(--mainTextColor);
    padding-top: 0.2em;
    padding-bottom: 0.2em;
    font-size: 0.7em;
  }

  .paper-links {
    display: flex;
    margin-bottom: 0.75em;
    justify-content: flex-start;
    color: var(--mainLinkColor);
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

  .error-message {
    color: #f63737;
    font-weight: bold;
  }
</style>
