<script lang="ts">
  import axios from "axios";
  import { onMount } from "svelte";

  let hnData = {};

  onMount(async () => {
    try {
      const { data } = await axios.get("/hn");
      hnData = data;
    } catch (err) {
      console.error(err);
    } finally {
      console.log(hnData);
    }
  });
</script>

<h1>Tech news will appear here</h1>
<pre>{JSON.stringify(hnData, null, 2)}</pre>
