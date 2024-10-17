<script>
  import { page } from "$app/stores";

  // Access the user_id parameter
  $: slug = $page.params.slug;
</script>

<h1>Slug: {slug}</h1>
