<script lang="ts">
  /**
   * The day and night cycle button. It owns the theme logic: choices, labels,
   * and the sunset or sunrise animation. Shells control the look through
   * `class` and the default slot. The slot receives `night` (true in the
   * dark theme) and `choice` (`light`, `dark`, or `system`).
   */
  import { design } from "$lib/stores/design";
  import {
    theme,
    themeChoice,
    nextThemeChoice,
    resolveTheme,
    setThemeChoice,
    type ThemeChoice,
  } from "$lib/stores/theme";
  import { playDayNightCycle } from "$lib/dayNightCycle";
  import { DESIGN_METAS } from "./metas";

  let className = "";
  export { className as class };

  $: themeCopy = DESIGN_METAS[$design].themeCopy;
  $: upcoming = nextThemeChoice($themeChoice, $theme);
  $: upcomingTheme = resolveTheme(upcoming);
  $: themeAction =
    upcoming === "system"
      ? "Use system theme"
      : upcomingTheme === "dark"
        ? themeCopy.toDark
        : themeCopy.toLight;
  $: themeTitle =
    $themeChoice === "system"
      ? `System, ${$theme === "dark" ? themeCopy.dark : themeCopy.light}`
      : $theme === "dark"
        ? themeCopy.dark
        : themeCopy.light;

  function themeLabel(next: ThemeChoice, to: "light" | "dark"): string {
    if (next === "system") return "System";
    return to === "dark" ? themeCopy.dark : themeCopy.light;
  }

  // Plays the full sunset or sunrise, and flips the theme partway through.
  // Choosing the system again skips the sky when the page is already that color.
  let cycling = false;
  async function onThemeClick(event: MouseEvent) {
    if (cycling) return;
    const next = nextThemeChoice($themeChoice, $theme);
    const to = resolveTheme(next);
    if (to === $theme) {
      setThemeChoice(next);
      return;
    }
    cycling = true;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    try {
      await playDayNightCycle({
        origin: { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 },
        to,
        label: themeLabel(next, to),
        onSwitch: () => setThemeChoice(next),
      });
    } finally {
      cycling = false;
    }
  }
</script>

<button
  type="button"
  class="theme-toggle {className}"
  data-choice={$themeChoice}
  on:click={onThemeClick}
  aria-label={themeAction}
  title={themeTitle}
>
  <slot night={$theme === "dark"} choice={$themeChoice}>
    <!-- A sun on the horizon: risen in daylight, set at night. -->
    <svg viewBox="0 0 24 24" aria-hidden="true" class:night={$theme === "dark"}>
      <defs>
        <clipPath id="horizon-clip"><rect x="0" y="0" width="24" height="16" /></clipPath>
      </defs>
      <g clip-path="url(#horizon-clip)">
        <circle class="sun" cx="12" cy="12" r="5" />
      </g>
      <path d="M3 16h18" />
      <path class="water" d="M7 19.5h10" />
    </svg>
  </slot>
</button>

<style>
  .theme-toggle {
    display: inline-grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
  }
  .theme-toggle[data-choice="system"] {
    border-color: var(--line);
  }
  .theme-toggle:hover {
    color: var(--ink);
    background: var(--hover-wash);
  }
  .theme-toggle :global(svg) {
    width: 20px;
    height: 20px;
    overflow: visible;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
  }
  .theme-toggle :global(.sun) {
    fill: currentColor;
    stroke: none;
    transform: translateY(-1px);
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .theme-toggle :global(.night .sun) {
    transform: translateY(7px);
  }
  .theme-toggle :global(.water) {
    opacity: 0.5;
  }
</style>
