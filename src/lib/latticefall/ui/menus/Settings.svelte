<script lang="ts">
  // Settings: controls, audio, graphics, difficulty, and accessibility. Controls drive the
  // input controller's bindings, audio drives the audio settings store, and the HUD options
  // drive the HUD's settings store; graphics, difficulty, and the other accessibility options
  // live in `menuSettings.ts`. Options the game doesn't read yet say so.
  import type { InputController } from '../../input';
  import { BUSES, audioSettings, setBusVolume, setMuted, type Bus } from '../../audio/settings';
  import { SCALE_MAX, SCALE_MIN, type HudSettingsStore, type HudVisibility } from '../hud/hudSettings';
  import {
    APPLIED,
    RANGES,
    WAYFARER_PRESET,
    type ColorBlind,
    type MenuSettings,
    type MenuSettingsStore,
    type Quality
  } from './menuSettings';
  import ControlsSettings from './ControlsSettings.svelte';

  export let engine: { input: InputController };
  export let hudSettings: HudSettingsStore;
  export let menuSettings: MenuSettingsStore;
  export let announce: (text: string) => void;

  const TABS = ['Controls', 'Audio', 'Graphics', 'Difficulty', 'Accessibility'] as const;
  let tab: (typeof TABS)[number] = 'Controls';
  let root: HTMLElement;
  let controls: { back?: () => boolean } | undefined;

  const BUS_LABELS: Record<Bus, string> = {
    master: 'Master',
    music: 'Music',
    effects: 'Effects',
    ambience: 'Ambience',
    ui: 'Interface'
  };
  const QUALITIES: Quality[] = ['low', 'medium', 'high'];
  const COLOR_MODES: { id: ColorBlind; label: string }[] = [
    { id: 'off', label: 'Off' },
    { id: 'protanopia', label: 'Protanopia' },
    { id: 'deuteranopia', label: 'Deuteranopia' },
    { id: 'tritanopia', label: 'Tritanopia' }
  ];
  const VISIBILITY: { id: HudVisibility; label: string }[] = [
    { id: 'always', label: 'Always shown' },
    { id: 'auto', label: 'Hide out of combat' },
    { id: 'hidden', label: 'Hidden' }
  ];

  $: s = $menuSettings as MenuSettings;
  $: audio = $audioSettings;
  $: wayfarer = (Object.keys(WAYFARER_PRESET) as (keyof MenuSettings['difficulty'])[]).every(
    (k) => s.difficulty[k] === WAYFARER_PRESET[k]
  );

  function set(fn: (m: MenuSettings) => void): void {
    menuSettings.update((m) => {
      fn(m);
      return m;
    });
  }

  function setHud(patch: Partial<{ scale: number; visibility: HudVisibility; showTimer: boolean }>): void {
    hudSettings.update((h) => ({ ...h, ...patch }));
    hudSettings.persist();
  }

  function onVisibility(e: Event): void {
    setHud({ visibility: (e.currentTarget as HTMLSelectElement).value as HudVisibility });
  }

  function onColorBlind(e: Event): void {
    const value = (e.currentTarget as HTMLSelectElement).value as ColorBlind;
    set((m) => (m.accessibility.colorBlind = value));
  }

  const pct = (v: number) => `${Math.round(v * 100)}%`;

  /** A control's own step first (the controls menu's capture, a focused slider), then the menu's. */
  export function back(): boolean {
    if (controls?.back?.()) return true;
    const active = document.activeElement;
    if (active instanceof HTMLElement && active.dataset.nav === 'body') {
      root?.querySelector<HTMLElement>('[data-nav="tabs"][aria-selected="true"]')?.focus();
      return true;
    }
    return false;
  }
</script>

<div class="cols settings" bind:this={root}>
  <ul class="list tabs" role="tablist" aria-label="Settings" aria-orientation="vertical">
    {#each TABS as t (t)}
      <li>
        <button
          type="button"
          role="tab"
          data-nav="tabs"
          aria-selected={tab === t}
          on:focus={() => (tab = t)}
          on:click={() => (tab = t)}
        >
          {t}
        </button>
      </li>
    {/each}
  </ul>

  <div role="tabpanel" aria-label={tab} class="body">
    {#if tab === 'Controls'}
      <ControlsSettings bind:this={controls} input={engine.input} {announce} />
    {:else if tab === 'Audio'}
      <div class="group">
        <h3>Audio</h3>
        <ul class="list opts">
          <li>
            <label>
              <input type="checkbox" data-nav="body" checked={audio.muted} on:change={(e) => setMuted(e.currentTarget.checked)} />
              Mute all sound
            </label>
          </li>
          {#each BUSES as b (b)}
            <li class="slider">
              <label for={`lf-bus-${b}`}>{BUS_LABELS[b]} <output>{pct(audio.volumes[b])}</output></label>
              <input
                id={`lf-bus-${b}`}
                type="range"
                data-nav="body"
                min="0"
                max="100"
                step="5"
                value={Math.round(audio.volumes[b] * 100)}
                aria-valuetext={pct(audio.volumes[b])}
                on:input={(e) => setBusVolume(b, Number(e.currentTarget.value) / 100)}
              />
            </li>
          {/each}
        </ul>
        <p class="note">Muting keeps your volumes, and both are remembered between visits.</p>
      </div>
    {:else if tab === 'Graphics'}
      <div class="group">
        <h3>Graphics</h3>
        <fieldset>
          <legend>Quality</legend>
          {#each QUALITIES as q (q)}
            <label class="radio">
              <input
                type="radio"
                name="lf-quality"
                data-nav="body"
                checked={s.graphics.quality === q}
                on:change={() => set((m) => (m.graphics.quality = q))}
              />
              {q[0].toUpperCase() + q.slice(1)}
            </label>
          {/each}
        </fieldset>
        {#if !APPLIED.quality}
          <p class="note">Saved, but the renderer doesn't read it yet. The presets arrive with the quality work.</p>
        {/if}
      </div>
    {:else if tab === 'Difficulty'}
      <div class="group">
        <h3>Difficulty</h3>
        <ul class="list opts">
          <li class="slider">
            <label for="lf-dmg">Damage taken <output>{s.difficulty.damageTaken}%</output></label>
            <input id="lf-dmg" type="range" data-nav="body" {...RANGES.damageTaken} value={s.difficulty.damageTaken} aria-valuetext={`${s.difficulty.damageTaken} percent`} on:input={(e) => set((m) => (m.difficulty.damageTaken = Number(e.currentTarget.value)))} />
          </li>
          <li class="slider">
            <label for="lf-ehp">Enemy HP <output>{s.difficulty.enemyHp}%</output></label>
            <input id="lf-ehp" type="range" data-nav="body" {...RANGES.enemyHp} value={s.difficulty.enemyHp} aria-valuetext={`${s.difficulty.enemyHp} percent`} on:input={(e) => set((m) => (m.difficulty.enemyHp = Number(e.currentTarget.value)))} />
          </li>
          <li class="slider">
            <label for="lf-speed">Game speed <output>{s.difficulty.gameSpeed}%</output></label>
            <input id="lf-speed" type="range" data-nav="body" {...RANGES.gameSpeed} value={s.difficulty.gameSpeed} aria-valuetext={`${s.difficulty.gameSpeed} percent`} on:input={(e) => set((m) => (m.difficulty.gameSpeed = Number(e.currentTarget.value)))} />
          </li>
          <li>
            <label><input type="checkbox" data-nav="body" checked={s.difficulty.infiniteStamina} on:change={(e) => set((m) => (m.difficulty.infiniteStamina = e.currentTarget.checked))} /> Infinite stamina</label>
          </li>
          <li>
            <label><input type="checkbox" data-nav="body" checked={s.difficulty.keepGlimmer} on:change={(e) => set((m) => (m.difficulty.keepGlimmer = e.currentTarget.checked))} /> Keep Glimmer on death</label>
          </li>
          <li>
            <label><input type="checkbox" data-nav="body" checked={s.difficulty.bossSkip} on:change={(e) => set((m) => (m.difficulty.bossSkip = e.currentTarget.checked))} /> Boss skip after 3 deaths</label>
          </li>
        </ul>
        <p class="row">
          <button
            type="button"
            data-nav="body"
            aria-pressed={wayfarer}
            on:click={() => {
              set((m) => (m.difficulty = { ...WAYFARER_PRESET }));
              announce('Wayfarer mode settings applied');
            }}
          >Wayfarer mode preset</button>
          <button
            type="button"
            data-nav="body"
            on:click={() => {
              menuSettings.reset();
              announce('Settings reset to the defaults');
            }}
          >Reset all menu settings</button>
        </p>
        {#if !APPLIED.difficulty}
          <p class="note">
            Nothing is locked behind difficulty. Game speed works now (it only changes how fast time runs in the
            browser). The other options are saved, but the simulation doesn't read them yet; they become part of
            the world configuration when the core does.
          </p>
        {/if}
      </div>
    {:else}
      <div class="group">
        <h3>Accessibility</h3>
        <ul class="list opts">
          <li class="slider">
            <label for="lf-text">Text size in menus <output>{s.accessibility.textSize}%</output></label>
            <input id="lf-text" type="range" data-nav="body" {...RANGES.textSize} value={s.accessibility.textSize} aria-valuetext={`${s.accessibility.textSize} percent`} on:input={(e) => set((m) => (m.accessibility.textSize = Number(e.currentTarget.value)))} />
          </li>
          <li class="slider">
            <label for="lf-hudscale">HUD scale <output>{$hudSettings.scale}%</output></label>
            <input id="lf-hudscale" type="range" data-nav="body" min={SCALE_MIN} max={SCALE_MAX} step="5" value={$hudSettings.scale} aria-valuetext={`${$hudSettings.scale} percent`} on:input={(e) => setHud({ scale: Number(e.currentTarget.value) })} />
          </li>
          <li>
            <label for="lf-hudvis">HUD</label>
            <select id="lf-hudvis" data-nav="body" value={$hudSettings.visibility} on:change={onVisibility}>
              {#each VISIBILITY as v (v.id)}
                <option value={v.id} selected={$hudSettings.visibility === v.id}>{v.label}</option>
              {/each}
            </select>
          </li>
          <li class="slider">
            <label for="lf-fov">Field of view <output>{s.accessibility.fov}&deg;</output></label>
            <input id="lf-fov" type="range" data-nav="body" {...RANGES.fov} value={s.accessibility.fov} aria-valuetext={`${s.accessibility.fov} degrees`} on:input={(e) => set((m) => (m.accessibility.fov = Number(e.currentTarget.value)))} />
          </li>
          <li class="slider">
            <label for="lf-camdist">Camera distance <output>{s.accessibility.cameraDistance} m</output></label>
            <input id="lf-camdist" type="range" data-nav="body" {...RANGES.cameraDistance} value={s.accessibility.cameraDistance} aria-valuetext={`${s.accessibility.cameraDistance} meters`} on:input={(e) => set((m) => (m.accessibility.cameraDistance = Number(e.currentTarget.value)))} />
          </li>
          <li>
            <label><input type="checkbox" data-nav="body" checked={s.accessibility.reducedMotion} on:change={(e) => set((m) => (m.accessibility.reducedMotion = e.currentTarget.checked))} /> Reduced motion (no camera shake or hit flashes, steady warnings, still menus)</label>
          </li>
          <li>
            <label><input type="checkbox" data-nav="body" checked={$hudSettings.showTimer} on:change={(e) => setHud({ showTimer: e.currentTarget.checked })} /> Show the in-game timer on the HUD</label>
          </li>
          <li>
            <label for="lf-cb">Color-blind mode</label>
            <select id="lf-cb" data-nav="body" on:change={onColorBlind}>
              {#each COLOR_MODES as c (c.id)}
                <option value={c.id} selected={s.accessibility.colorBlind === c.id}>{c.label}</option>
              {/each}
            </select>
          </li>
          <li>
            <label><input type="checkbox" data-nav="body" checked={s.accessibility.subtitles} on:change={(e) => set((m) => (m.accessibility.subtitles = e.currentTarget.checked))} /> Subtitles for speech and important sounds</label>
            {#if s.accessibility.subtitles}
              <p class="sample" aria-hidden="true"><span>&#9834;</span>[The bell tolls]</p>
            {/if}
          </li>
        </ul>
        <p class="note">
          Everything here takes effect now and is remembered between visits. Reduced motion starts on when your
          system asks for it. Color-blind modes recolor the HUD and the warnings; every status and warning also
          has its own shape or pattern, in every mode. Subtitles caption the bell, ice, chains, and boss sounds
          (conversations are always text).
        </p>
      </div>
    {/if}
  </div>
</div>

<style>
  .cols.settings {
    grid-template-columns: minmax(9em, 12em) minmax(0, 1fr);
  }

  [role='tab'][aria-selected='true'] {
    border-color: var(--fjord);
    box-shadow: inset 3px 0 0 var(--fjord);
  }

  .opts li {
    padding: 0.25em 0;
  }

  .opts label {
    display: inline-flex;
    gap: 0.5em;
    align-items: center;
  }

  .slider {
    display: grid;
    gap: 0.25em;
  }

  .slider label {
    display: flex;
    justify-content: space-between;
  }

  output {
    font-family: var(--font-mono);
    color: var(--ink-2);
  }

  input[type='range'] {
    width: 100%;
    accent-color: var(--fjord);
  }

  fieldset {
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    padding: 0.5em 0.75em;
    display: flex;
    gap: 1.25em;
  }

  .radio {
    display: inline-flex;
    gap: 0.4em;
    align-items: center;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5em;
  }

  select {
    margin-left: 0.5em;
  }

  /* How a subtitle looks, at the current text size. */
  .sample {
    display: block;
    width: max-content;
    margin: 0.4em 0 0;
    padding: 0.15em 0.75em;
    border: 1px solid var(--line-strong);
    border-radius: 3px;
    background: var(--surface);
    color: var(--ink);
  }

  .sample span {
    margin-right: 0.5em;
    color: var(--ink-3);
  }
</style>
