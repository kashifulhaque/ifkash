<script lang="ts">
  import "../app.css";
  // Every design's overrides load up front. They are scoped by
  // `data-design`, and they must come after app.css to win ties.
  import "$lib/designs/almanac/design.css";
  import "$lib/designs/japandi/design.css";
  import "$lib/designs/ember/design.css";
  import { onMount, tick } from "svelte";
  import { page } from "$app/stores";
  import { afterNavigate } from "$app/navigation";
  import { animate, utils } from "animejs";
  import { prefersReducedMotion } from "$lib/motion";
  import AiAgent from "$lib/components/AiAgent.svelte";
  import { registry } from "$lib/designs/registry";
  import { activeDesign, hydrated } from "$lib/designs/active";
  import { isFullWidthPath } from "$lib/designs/shared";

  $: isFullWidth = isFullWidthPath($page.url.pathname);
  $: Shell = registry[$activeDesign].Shell;

  // A short fade and lift for the page content on client-side navigation.
  // Every shell marks its content wrapper with id="main".
  afterNavigate(({ from, to }) => {
    const el = document.getElementById("main");
    if (!el || !from || !to || from.url.pathname === to.url.pathname) return;
    if (prefersReducedMotion()) return;
    const anim = animate(el, {
      opacity: [0, 1],
      translateY: [10, 0],
      duration: 450,
      ease: "outExpo",
      onComplete: (a) => utils.cleanInlineStyles(a),
    });
    // Never leave the page hidden if a throttled renderer skips the frames.
    setTimeout(() => {
      if (anim.completed) return;
      anim.cancel();
      el.style.removeProperty("opacity");
      el.style.removeProperty("transform");
    }, 1200);
  });

  // The server render and hydration use the default shell. After mount, the
  // saved design's shell takes over, and then the page can be shown again.
  onMount(() => {
    hydrated.set(true);
    tick().then(() => document.documentElement.removeAttribute("data-shell-pending"));
  });
</script>

<svelte:component this={Shell} fullWidth={isFullWidth}>
  <slot />
</svelte:component>

{#if !isFullWidth}
  <AiAgent />
{/if}
