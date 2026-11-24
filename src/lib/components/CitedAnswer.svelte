<script lang="ts">
  /* An Exa answer with its [n] markers linked to a numbered source list.
   * Used by the command deck (ask my portfolio) and the workout research notes. */
  import { citeParts, type Source } from '$lib/exaApi';

  export let answer: string;
  export let sources: Source[] = [];
  /** Open sources in a new tab (external research) or in place (own pages). */
  export let external = false;

  $: paragraphs = answer.split(/\n\s*\n/).map((p) => citeParts(p.trim(), sources));

  const host = (url: string) => {
    try {
      const u = new URL(url);
      return (u.hostname.replace(/^www\./, '') + u.pathname).replace(/\/$/, '');
    } catch {
      return url;
    }
  };
</script>

{#each paragraphs as parts}<p class="answer">
  {#each parts as part}
    {#if 'text' in part}{part.text}{:else}<a
        class="cite"
        href={part.url}
        target={external ? '_blank' : undefined}
        rel={external ? 'noopener' : undefined}>{part.n}</a
      >{/if}
  {/each}
</p>{/each}
{#if sources.length}
  <ol class="sources">
    {#each sources as s}
      <li>
        <a href={s.url} target={external ? '_blank' : undefined} rel={external ? 'noopener' : undefined}>
          <span class="s-title">{s.title || host(s.url)}</span>
          <span class="s-host">{host(s.url)}</span>
        </a>
      </li>
    {/each}
  </ol>
{/if}

<style>
  .answer {
    margin: 0;
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    line-height: 1.6;
  }

  .answer + .answer {
    margin-top: var(--s-3);
  }

  .cite {
    display: inline-grid;
    place-items: center;
    min-width: 16px;
    height: 16px;
    margin: 0 1px;
    padding: 0 3px;
    border: 1px solid var(--line);
    border-radius: var(--radius-xs);
    color: var(--fjord);
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 1;
    text-decoration: none;
    vertical-align: 2px;
  }

  .cite:hover {
    border-color: var(--fjord);
  }

  .sources {
    display: grid;
    gap: var(--s-1);
    margin: var(--s-3) 0 0;
    padding-left: var(--s-5);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .sources a {
    display: flex;
    flex-wrap: wrap;
    gap: 0 var(--s-2);
    color: var(--ink-2);
    text-decoration: none;
  }

  .sources a:hover .s-title {
    color: var(--fjord);
  }

  .s-title {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }

  .s-host {
    overflow: hidden;
    max-width: 100%;
    color: var(--ink-3);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
