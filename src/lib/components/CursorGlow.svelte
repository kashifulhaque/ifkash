<script lang="ts">
  import { onMount } from "svelte";

  let glow: HTMLDivElement;

  onMount(() => {
    const finePointer = window.matchMedia("(hover: hover) and (pointer: fine)");
    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
    let frame: number | null = null;
    let fadeTimer: ReturnType<typeof setTimeout> | null = null;
    let x = 0;
    let y = 0;

    function hide() {
      glow.classList.remove("is-visible");
    }

    function render() {
      glow.style.transform = `translate3d(${x - 16}px, ${y - 16}px, 0)`;
      glow.classList.add("is-visible");
      frame = null;

      if (fadeTimer !== null) clearTimeout(fadeTimer);
      fadeTimer = setTimeout(hide, 500);
    }

    function handlePointerMove(event: PointerEvent) {
      if (!finePointer.matches || reducedMotion.matches || event.pointerType === "touch") {
        return;
      }

      x = event.clientX;
      y = event.clientY;
      if (frame === null) frame = requestAnimationFrame(render);
    }

    function handlePreferenceChange() {
      if (!finePointer.matches || reducedMotion.matches) hide();
    }

    window.addEventListener("pointermove", handlePointerMove, { passive: true });
    window.addEventListener("blur", hide);
    document.documentElement.addEventListener("mouseleave", hide);
    finePointer.addEventListener("change", handlePreferenceChange);
    reducedMotion.addEventListener("change", handlePreferenceChange);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("blur", hide);
      document.documentElement.removeEventListener("mouseleave", hide);
      finePointer.removeEventListener("change", handlePreferenceChange);
      reducedMotion.removeEventListener("change", handlePreferenceChange);
      if (frame !== null) cancelAnimationFrame(frame);
      if (fadeTimer !== null) clearTimeout(fadeTimer);
    };
  });
</script>

<div class="cursor-glow" bind:this={glow} aria-hidden="true"></div>

<style>
  .cursor-glow {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 2000;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    pointer-events: none;
    opacity: 0;
    background: radial-gradient(
      circle,
      color-mix(in srgb, var(--accent) 16%, transparent) 0%,
      color-mix(in srgb, var(--accent) 7%, transparent) 48%,
      transparent 72%
    );
    filter: blur(1px);
    transition: opacity 180ms ease;
    will-change: transform, opacity;
  }

  .cursor-glow:global(.is-visible) {
    opacity: 1;
  }

  @media (hover: none), (pointer: coarse), (prefers-reduced-motion: reduce) {
    .cursor-glow {
      display: none;
    }
  }
</style>
