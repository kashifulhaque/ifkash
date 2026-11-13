<script lang="ts">
  import EntryPage from '../../EntryPage.svelte';
  import type { PageData } from './$types';

  export let data: PageData;
</script>

<EntryPage entry={data.entry} />
