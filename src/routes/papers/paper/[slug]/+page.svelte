<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";

  interface Author {
    name: string;
    url: string;
  }

  interface Paper {
    title: string;
    authors: Author[];
    dataset_url: string;
    arxiv_page_url: string;
    arxiv_pdf_url: string;
    description: string;
  }

  let error = {
    isError: false,
    message: "",
  };
  let loading: boolean = true;
  let paper: Paper;

  $: slug = $page.params.slug;
  if (slug.length == 0) {
    (error.isError = true), (error.message = "slug not found");
  }

  onMount(async () => {
    const paperUrl = `https://paperswithcode.com/paper/${slug}`;
    try {
      const response = await fetch(`/api/pwc/paper?paper_url=${paperUrl}`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();

      if (!data) {
        throw new Error("Invalid data received from API");
      }

      paper = data;
    } catch (err) {
      error.isError = true;
      error.message = err as string;
      console.error("Error fetching papers", error);
    } finally {
      loading = false;
    }
  });
</script>

<h1>{paper}</h1>
