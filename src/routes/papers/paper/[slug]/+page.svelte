<script>
  import { page } from "$app/stores";

  // Access the slug from the URL
  $: slug = "/paper/" + $page.params.slug;

  // You can fetch paper details based on the slug here
</script>

<h1>Paper Slug: {slug}</h1>

<!-- Use the slug to fetch or display more details about the paper -->
