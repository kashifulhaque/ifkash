<script>
  import "../app.css";
  import { dev } from "$app/environment";
  import { injectAnalytics } from "@vercel/analytics/sveltekit";

  injectAnalytics({
    mode: dev ? "development" : "production"
  });
</script>

<slot />
