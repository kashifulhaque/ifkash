<script lang="ts">
  // Hollowhearth's dialogue (WP-068): an NPC's greeting and **Ask about...** menu, and the five
  // stations (the Archive shelves, the Ledger Stand, the Mirror, the keepsake table, the Hearth).
  // DOM over the canvas, drawn with the site's tokens so it follows the design and the theme.
  //
  // The simulation owns the conversation and the quests. This component reads the state the core
  // exports (`host.read()`) and changes it only through `host`, which queues core `Command`s. Facts
  // are never copied into game data: a topic names portfolio entries by section and slug, and the
  // plain facts panel shows them exactly as `content.ts` writes them. **No fact is locked behind a
  // quest**; a quest only changes the greeting and pays a reward.
  //
  // Input: the keyboard (arrows, Enter or Space, Escape), a gamepad (D-pad or left stick, A, B), and
  // screen readers (a modal dialog, labelled lists and buttons, a polite live region).
  import { createEventDispatcher, onDestroy, tick } from 'svelte';
  import { name as ownerName, tagline, type LootItem } from '$lib/content';
  import { codex } from '../../codex/store';
  import { focusFirst, moveForKey, navigate, startGamepadNav } from '../menus/nav';
  import FactsPanel from '../portfolio/FactsPanel.svelte';
  import { resumeHref } from '../portfolio/resume';
  import type { DialogueHost } from './dialogueHost';
  import {
    emptyDialogueState,
    greetingFor,
    QUEST_TOPIC,
    questReply,
    type DialogueEvent,
    type DialogueState,
    type NpcDef,
    type StationDef
  } from './dialogueState';
  import { keepsakes, resolveFacts } from './facts';

  export let host: DialogueHost;
  export let engine: { setPaused(paused: boolean): void };
  /** True while a conversation is open. Bind to it to hold your own keys. */
  export let open = false;

  const dispatch = createEventDispatcher<{ unlock: string[] }>();

  let state: DialogueState = emptyDialogueState();
  let npc: NpcDef | null = null;
  let station: StationDef | null = null;
  let root: HTMLElement | undefined;
  let live = '';
  let stopPad: (() => void) | null = null;

  $: npc = open && state.kind === 'npc' ? host.npc(state.index) : null;
  $: station = open && state.kind === 'station' ? host.station(state.index) : null;
  $: topic = npc && state.topic > 0 && state.topic < QUEST_TOPIC ? (npc.topics[state.topic - 1] ?? null) : null;
  $: onQuest = state.topic === QUEST_TOPIC;
  $: speech = npc
    ? onQuest
      ? questReply(npc, state.reply)
      : topic
        ? topic.line
        : greetingFor(npc, state.questState)
    : (station?.line ?? '');
  $: facts = ((): LootItem[] => {
    if (topic) return resolveFacts(topic.facts);
    if (!station) return [];
    return station.unmapped_only ? keepsakes(station.facts) : resolveFacts(station.facts);
  })();
  $: title = npc?.name ?? station?.name ?? '';
  $: role = npc?.role ?? 'Hollowhearth';
  $: quest = npc?.quest ?? null;
  $: service = topic?.service ?? null;
  $: questNote =
    state.questState === 0
      ? 'A request'
      : state.questState === 1
        ? `${state.questHave} of ${state.questRequired} brought`
        : 'Done';

  const SERVICE_NOTE: Record<string, string> = {
    Respec: 'Rest at a Lantern to rewrite your attributes. It costs a Rewrite Quill.',
    Upgrade: 'Rest at a Lantern to upgrade or temper a weapon.',
    Shop: 'The Peddler’s counter opens in a later update. The stock is listed above.'
  };

  function announce(text: string): void {
    live = '';
    setTimeout(() => (live = text), 40);
  }

  function refresh(): void {
    state = host.read();
  }

  /** The engine calls this for each dialogue event from the core. */
  export function handleEvent(e: DialogueEvent): void {
    if (e.kind === 'dialogue') {
      if (e.what === 'opened') void show();
      else if (e.what === 'closed') close();
      else {
        refresh();
        void tick().then(() => announce(speech));
      }
    } else if (e.kind === 'quest') {
      refresh();
      if (e.state === 2) {
        const def = host.npc(e.npc)?.quest;
        if (def?.reward.codex.length) dispatch('unlock', def.reward.codex);
      }
    } else if (e.kind === 'appearance') {
      refresh();
    }
  }

  async function show(): Promise<void> {
    refresh();
    if (!open) {
      open = true;
      engine.setPaused(true);
      stopPad = startGamepadNav((m) => {
        if (!root) return;
        if (m === 'back' || m === 'menu') end();
        else navigate(root, m);
      });
    }
    await tick();
    announce(`${title}. ${speech}`);
    if (root) focusFirst(root);
  }

  function close(): void {
    if (!open) return;
    open = false;
    stopPad?.();
    stopPad = null;
    state = emptyDialogueState();
    engine.setPaused(false);
  }

  /** Ends the conversation: the core closes it and the event closes the panel. */
  function end(): void {
    host.end();
    // A paused world answers at once, but close here too if the core did not report it.
    if (open && !host.read().open) close();
  }

  function back(): void {
    if (npc && state.topic !== 0) host.back();
    else end();
  }

  function onKey(e: KeyboardEvent): void {
    if (!open || !root) return;
    if (e.key === 'Tab') {
      const items = [...root.querySelectorAll<HTMLElement>('button, a[href]')].filter(
        (el) => !el.hasAttribute('disabled') && el.offsetParent !== null
      );
      if (items.length === 0) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
      return;
    }
    const m = moveForKey(e);
    if (m === null) return;
    e.preventDefault();
    e.stopPropagation();
    if (m === 'back') back();
    else navigate(root, m);
  }

  onDestroy(() => {
    stopPad?.();
    if (open) engine.setPaused(false);
  });

  function readResume(): void {
    codex.mark('reader');
  }
</script>

<svelte:window on:keydown|capture={onKey} />

<div class="lf-dialogue-live" aria-live="polite" role="status">{live}</div>

{#if open && (npc || station)}
  <div class="lf-dialogue" bind:this={root}>
    <div class="panel" role="dialog" aria-modal="true" aria-labelledby="lf-dialogue-title">
      <header class="head">
        <div>
          <p class="eyebrow">{role}</p>
          <h2 id="lf-dialogue-title">{title}</h2>
        </div>
        <button type="button" class="leave" data-nav="main" on:click={end}>
          {npc ? 'Say goodbye' : 'Step away'} <kbd>Esc</kbd>
        </button>
      </header>

      <div class="body">
        <div class="talk">
          <p class="speech" class:quest={onQuest}>{speech}</p>
          {#if service}
            <p class="note">{SERVICE_NOTE[service]}</p>
          {/if}

          {#if npc}
            <h3 id="lf-ask">Ask about</h3>
            <ul class="menu" aria-labelledby="lf-ask">
              {#each npc.topics as t, i (t.id)}
                <li>
                  <button
                    type="button"
                    data-nav="main"
                    aria-current={state.topic === i + 1 ? 'true' : undefined}
                    on:click={() => host.ask(i)}>{t.label}</button
                  >
                </li>
              {/each}
              {#if quest}
                <li>
                  <button
                    type="button"
                    class="quest-button"
                    data-nav="main"
                    aria-current={onQuest ? 'true' : undefined}
                    on:click={() => host.askQuest()}
                  >
                    <span>{quest.label}</span>
                    <small class:done={state.questState === 2}>{questNote}</small>
                  </button>
                </li>
              {/if}
            </ul>
          {/if}

          {#if station?.kind === 'Mirror'}
            <h3 id="lf-presets">Appearance</h3>
            <ul class="menu choices" aria-labelledby="lf-presets">
              {#each station.presets as p, i}
                <li>
                  <button
                    type="button"
                    data-nav="main"
                    aria-pressed={state.mirrorPreset === i}
                    on:click={() => host.appearance(i, state.mirrorDye)}>{p}</button
                  >
                </li>
              {/each}
            </ul>
            <h3 id="lf-dyes">Dye</h3>
            <ul class="menu choices" aria-labelledby="lf-dyes">
              {#each station.dyes as d, i}
                {@const owned = (state.dyeMask & (1 << i)) !== 0}
                <li>
                  <button
                    type="button"
                    data-nav="main"
                    aria-pressed={state.mirrorDye === i}
                    aria-disabled={owned ? undefined : 'true'}
                    on:click={() => owned && host.appearance(state.mirrorPreset, i)}
                  >
                    <span>{d.name}</span>
                    {#if !owned}<small>A quest reward</small>{/if}
                  </button>
                </li>
              {/each}
            </ul>
          {/if}

          {#if station?.kind === 'Ledger'}
            <p class="actions">
              <a class="btn primary" data-nav="main" href={resumeHref()} target="_blank" rel="noopener noreferrer" on:click={readResume}
                >Read the résumé</a
              >
            </p>
          {/if}
        </div>

        <div class="facts" aria-label="Facts">
          {#if station?.kind === 'Hearth'}
            <article class="inscription">
              <h3>The inscription</h3>
              <p class="name">{ownerName}</p>
              <p class="tagline">{tagline}</p>
            </article>
          {:else if station?.kind === 'Keepsakes' && facts.length === 0}
            <p class="note">Nothing here yet. Every project already has a relic of its own.</p>
          {:else if facts.length}
            <ul class="fact-list">
              {#each facts as item (item.slug ?? item.title)}
                <li><FactsPanel {item} level={3} /></li>
              {/each}
            </ul>
          {:else if npc && !topic && !onQuest}
            <p class="note">Choose something to ask about. Everything is open; nothing waits on a quest.</p>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .lf-dialogue-live {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .lf-dialogue {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 20;
    display: flex;
    justify-content: center;
    padding: var(--s-4);
    font-family: var(--font-sans);
    font-size: var(--text-base);
    color: var(--ink);
    pointer-events: none;
  }

  .panel {
    pointer-events: auto;
    /* The text size setting (WP-082) zooms the panel; the limits shrink to match. */
    zoom: var(--lf-text, 1);
    width: min(62em, calc(100% / var(--lf-text, 1)));
    max-height: calc(min(26em, 70dvh) / var(--lf-text, 1));
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-md);
    box-shadow: 0 18px 48px var(--shadow-color);
    overflow: hidden;
  }

  .head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-3) var(--s-5);
    border-bottom: 1px solid var(--line);
  }

  .eyebrow {
    margin: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  h2 {
    margin: 2px 0 0;
    font-family: var(--font-display);
    font-size: var(--text-lg, 1.375rem);
    font-weight: 500;
    line-height: 1.2;
  }

  h3 {
    margin: var(--s-4) 0 var(--s-2);
    color: var(--ink-3);
    font-size: var(--text-xs);
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .body {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: var(--s-5);
    padding: var(--s-4) var(--s-5);
    overflow: hidden;
    min-height: 0;
  }

  .talk,
  .facts {
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
  }

  .speech {
    margin: 0;
    max-width: var(--measure);
    font-family: var(--font-display);
    font-size: var(--text-md);
    line-height: 1.5;
    color: var(--ink);
  }

  .speech.quest {
    padding-left: var(--s-3);
    border-left: 3px solid var(--moss);
  }

  .note {
    margin: var(--s-3) 0 0;
    max-width: var(--measure);
    color: var(--ink-2);
    font-size: var(--text-sm);
  }

  .menu {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--s-1);
  }

  .menu.choices {
    grid-template-columns: repeat(auto-fit, minmax(7em, 1fr));
  }

  .fact-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--s-3);
  }

  .inscription {
    padding: var(--s-4);
    background: var(--surface-2);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
  }

  .inscription h3 {
    margin-top: 0;
  }

  .inscription .name {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--text-lg, 1.375rem);
  }

  .inscription .tagline {
    margin: var(--s-2) 0 0;
    max-width: var(--measure);
    color: var(--ink-2);
    line-height: 1.5;
  }

  .actions {
    margin: var(--s-4) 0 0;
  }

  .btn,
  button {
    font: inherit;
    color: var(--ink);
    background: var(--surface-2);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    padding: 0.5em 0.75em;
    cursor: pointer;
    min-height: 2.5em;
    text-align: left;
    text-decoration: none;
    display: inline-flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
  }

  .menu button {
    width: 100%;
  }

  .btn:hover,
  button:hover {
    border-color: var(--line-strong);
    background: var(--hover-wash);
  }

  .btn:focus-visible,
  button:focus-visible {
    outline: 2px solid var(--fjord);
    outline-offset: 2px;
    border-color: var(--fjord);
    background: var(--fjord-wash);
  }

  button[aria-current='true'],
  button[aria-pressed='true'] {
    border-color: var(--fjord);
    background: var(--fjord-wash);
  }

  button[aria-disabled='true'] {
    color: var(--ink-3);
    cursor: not-allowed;
    border-style: dashed;
  }

  .primary {
    background: var(--fjord);
    border-color: var(--fjord-strong);
    color: var(--ink-inverse);
  }

  .primary:hover {
    background: var(--fjord-strong);
  }

  small {
    color: var(--ink-3);
    font-size: var(--text-xs);
  }

  small.done {
    color: var(--moss);
  }

  kbd {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: 0 4px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-xs);
    background: var(--surface);
  }

  .leave {
    flex: none;
  }

  @media (max-width: 640px) {
    .body {
      grid-template-columns: 1fr;
      overflow-y: auto;
    }

    .talk,
    .facts {
      overflow: visible;
    }

    .leave kbd {
      display: none;
    }
  }

  @media (prefers-reduced-motion: no-preference) {
    .panel {
      animation: lf-dialogue-in var(--dur) var(--ease-out);
    }
  }

  @keyframes lf-dialogue-in {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
