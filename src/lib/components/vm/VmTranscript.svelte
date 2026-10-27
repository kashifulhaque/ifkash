<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { ArrowLeft, MessageSquare } from 'lucide-svelte';
  import Conversation from './Conversation.svelte';
  import { vmApi, VmSignedOut } from '$lib/vmApi';
  import { fromMessages, type ConvItem } from '$lib/vmConversation';

  export let id: string;
  export let title = '';
  export let cwd = '';

  const dispatch = createEventDispatcher<{ back: void; continue: { sessionId: string; cwd: string } }>();

  let items: ConvItem[] = [];
  let loading = true;
  let error = '';
  let sessionCwd = cwd;

  onMount(async () => {
    try {
      const t = await vmApi.getSession(id);
      items = fromMessages(t.messages);
      sessionCwd = t.cwd || cwd;
    } catch (e) {
      if (!(e instanceof VmSignedOut)) error = (e as Error).message;
    } finally {
      loading = false;
    }
  });
</script>

<div class="tr">
  <div class="head">
    <button type="button" class="btn btn-sm" on:click={() => dispatch('back')}><ArrowLeft size={14} aria-hidden="true" /> Sessions</button>
    <button type="button" class="btn btn-sm btn-primary" disabled={loading || !!error} on:click={() => dispatch('continue', { sessionId: id, cwd: sessionCwd })}>
      <MessageSquare size={14} aria-hidden="true" /> Continue in chat
    </button>
  </div>
  <h2 class="title">{title || 'Untitled session'}</h2>
  <p class="meta">{sessionCwd}</p>
  {#if loading}
    <p class="muted">Loading transcript…</p>
  {:else if error}
    <p class="err" role="alert">{error}</p>
  {:else if items.length === 0}
    <p class="muted">This transcript has no messages.</p>
  {:else}
    <Conversation {items} />
  {/if}
</div>

<style>
  .tr { display: flex; flex-direction: column; gap: var(--s-3); min-width: 0; }
  .head { display: flex; justify-content: space-between; gap: var(--s-2); position: sticky; top: 64px; z-index: 4; padding: var(--s-2) 0; background: var(--bg); }
  .title { margin: 0; font-family: var(--font-display); font-size: var(--text-md); font-weight: 500; line-height: 1.3; overflow-wrap: anywhere; }
  .meta { margin: 0; color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); overflow-wrap: anywhere; }
  .muted { color: var(--ink-3); font-size: var(--text-sm); margin: 0; }
  .err { color: var(--falu); font-size: var(--text-sm); margin: 0; }
</style>
