<script lang="ts">
  // Global styles for the page root's accessibility attributes (WP-082): `data-cb` (the
  // color-blind mode) remaps the site tokens the HUD and menus use for status colors to a
  // palette that stays apart for that kind of vision, and `data-calm` (reduced motion) stops
  // the DOM UI's own transitions and animations. Shape and pattern cues don't depend on these;
  // they are always on.
</script>

<style>
  /* Reduced motion: nothing in the game UI animates. */
  :global(.lf-root[data-calm] *),
  :global(.lf-root[data-calm] *::before),
  :global(.lf-root[data-calm] *::after) {
    animation: none !important;
    transition: none !important;
    scroll-behavior: auto !important;
  }

  /* Protanopia and deuteranopia: red and green merge, so health is orange, focus is blue,
     stamina is yellow, and the rest sit apart along the blue-orange axis. */
  :global(.lf-root[data-cb='protanopia']),
  :global(.lf-root[data-cb='deuteranopia']) {
    --falu: #c24a00;
    --moss: #8f7d00;
    --fjord: #0b5fa5;
    --fjord-strong: #08487d;
    --clay: #7a4a8a;
    --danger: #b0003c;
  }

  :global([data-theme='dark'] .lf-root[data-cb='protanopia']),
  :global([data-theme='dark'] .lf-root[data-cb='deuteranopia']) {
    --falu: #ff9a4d;
    --moss: #f0d34a;
    --fjord: #6cb8ff;
    --fjord-strong: #a8d4ff;
    --clay: #d9a3e8;
    --danger: #ff8aa8;
  }

  /* Tritanopia: blue and yellow merge, so health is crimson, focus is teal, stamina is orange. */
  :global(.lf-root[data-cb='tritanopia']) {
    --falu: #c0283d;
    --moss: #b36b00;
    --fjord: #1f7a76;
    --fjord-strong: #155a57;
    --clay: #c2185b;
    --danger: #8f1d2a;
  }

  :global([data-theme='dark'] .lf-root[data-cb='tritanopia']) {
    --falu: #ff8a9a;
    --moss: #ffb347;
    --fjord: #5fd0c8;
    --fjord-strong: #9de6e0;
    --clay: #ff7fb0;
    --danger: #ff8a9a;
  }
</style>
