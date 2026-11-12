// The dev overlay panel: a DOM box drawn over the canvas with the core tick time, frame
// time, draw calls, triangles, the actor list and the state hash, plus the overlay
// toggles and dev commands. Styled with the site tokens from src/app.css.

import type { DrawActor } from '../engine/snapshot';
import type { DevCommands } from './commands';
import type { DebugDraw } from './debugDraw';
import { OVERLAY_NAMES, type OverlayName } from './params';

export interface PanelStats {
  /** Average core tick time over the last window, in ms. */
  tickMs: number;
  /** Slowest core tick in the window, in ms. */
  tickMaxMs: number;
  frameMs: number;
  calls: number;
  triangles: number;
  tick: number;
  hash: string;
  actors: DrawActor[];
}

const CSS = `
.lf-devtools { position: absolute; top: calc(var(--s-4) + 2.75rem); right: var(--s-4); width: 17rem; max-height: calc(100dvh - 8rem); overflow: auto;
  padding: var(--s-2) var(--s-3); background: var(--surface); border: 1px solid var(--line); color: var(--ink);
  font-family: var(--font-mono); font-size: var(--text-xs); }
.lf-devtools h2 { margin: var(--s-2) 0 var(--s-1); font-size: var(--text-xs); font-weight: 600; color: var(--ink-3); text-transform: uppercase; letter-spacing: 0.04em; }
.lf-devtools h2:first-child { margin-top: 0; }
.lf-devtools dl { display: grid; grid-template-columns: auto 1fr; gap: 2px var(--s-3); margin: 0; }
.lf-devtools dt { color: var(--ink-3); }
.lf-devtools dd { margin: 0; text-align: right; overflow-wrap: anywhere; }
.lf-devtools label { display: flex; gap: var(--s-2); align-items: center; }
.lf-devtools label span:last-child { margin-left: auto; color: var(--ink-3); }
.lf-devtools table { width: 100%; border-collapse: collapse; }
.lf-devtools td, .lf-devtools th { padding: 0 var(--s-1) 0 0; text-align: left; font-weight: 400; }
.lf-devtools td:nth-child(n+3) { text-align: right; }
.lf-devtools select, .lf-devtools input[type=number], .lf-devtools button { font: inherit; color: var(--ink); background: var(--bg); border: 1px solid var(--line-strong); padding: 1px var(--s-1); }
.lf-devtools button { cursor: pointer; }
.lf-devtools .row { display: flex; gap: var(--s-2); margin: var(--s-1) 0; align-items: center; }
.lf-devtools .row > :first-child { flex: 1; min-width: 0; }
.lf-devtools .warn { color: var(--falu); margin: var(--s-1) 0 0; }
.lf-devtools .note { color: var(--ink-3); }
`;

function el<K extends keyof HTMLElementTagNameMap>(tag: K, attrs: Partial<HTMLElementTagNameMap[K]> = {}, ...kids: Array<Node | string>): HTMLElementTagNameMap[K] {
  const node = Object.assign(document.createElement(tag), attrs);
  node.append(...kids);
  return node;
}

export interface PanelDeps {
  parent: HTMLElement;
  scenario: string;
  spawnIds: string[];
  commands: DevCommands;
  draw: DebugDraw;
  warnings: string[];
  onOverlayChange(): void;
}

export class DevPanel {
  private root: HTMLElement;
  private style: HTMLStyleElement;
  private readout = new Map<string, HTMLElement>();
  private counts = new Map<OverlayName, HTMLElement>();
  private actorBody = el('tbody');
  private actorCount = el('h2');
  private freeze!: HTMLInputElement;

  constructor(private deps: PanelDeps) {
    this.style = el('style', { textContent: CSS });
    document.head.append(this.style);
    const { commands, draw } = deps;

    const stats = el('dl');
    for (const key of ['tick', 'core tick', 'frame', 'draw calls', 'triangles', 'actors', 'hash']) {
      const dd = el('dd', { textContent: '-' });
      this.readout.set(key, dd);
      stats.append(el('dt', { textContent: key }), dd);
    }

    const overlays = el('div');
    for (const name of OVERLAY_NAMES) {
      const box = el('input', { type: 'checkbox', checked: draw.isEnabled(name) });
      const count = el('span', { textContent: '0' });
      this.counts.set(name, count);
      box.addEventListener('change', () => {
        draw.setEnabled(name, box.checked);
        deps.onOverlayChange();
      });
      overlays.append(el('label', {}, box, el('span', { textContent: name }), count));
    }

    this.freeze = el('input', { type: 'checkbox', checked: commands.isFrozen });
    this.freeze.addEventListener('change', () => commands.setFrozen(this.freeze.checked));

    const cmds = el('div');
    cmds.append(el('label', {}, this.freeze, el('span', { textContent: 'freeze simulation' })));
    if (commands.available) {
      const select = el('select');
      for (const id of deps.spawnIds) select.append(el('option', { value: id, textContent: id }));
      const go = el('button', { type: 'button', textContent: 'Teleport' });
      go.addEventListener('click', () => commands.teleport(select.value));
      const god = el('input', { type: 'checkbox', checked: commands.god() });
      god.addEventListener('change', () => commands.setGod(god.checked));
      const level = el('input', { type: 'number', min: '1', max: '999', value: String(commands.level() ?? 1) });
      level.addEventListener('change', () => {
        const set = commands.setLevel(Number(level.value));
        if (set !== null) level.value = String(set);
      });
      const setLevel = el('span', { className: 'note', textContent: 'level (stored; no effect yet)' });
      cmds.append(
        el('div', { className: 'row' }, select, go),
        el('label', {}, god, el('span', { textContent: 'god mode' })),
        el('div', { className: 'row' }, level, setLevel)
      );
    } else {
      cmds.append(el('p', { className: 'warn', textContent: 'WASM built without the dev feature: teleport, god mode and level are off. Run LF_WASM_FEATURES=dev bash game/scripts/build-wasm.sh.' }));
    }

    const actors = el('table', {}, el('thead', {}, el('tr', {}, el('th', { textContent: 'id' }), el('th', { textContent: 'pos' }), el('th', { textContent: 'hp' }))), this.actorBody);

    this.root = el(
      'aside',
      { className: 'lf-devtools' },
      el('h2', { textContent: `dev · ${deps.scenario}` }),
      stats,
      el('h2', { textContent: 'overlays' }),
      overlays,
      el('h2', { textContent: 'commands' }),
      cmds,
      this.actorCount,
      actors
    );
    this.root.setAttribute('aria-label', 'Developer tools');
    for (const w of deps.warnings) this.root.append(el('p', { className: 'warn', textContent: w }));
    deps.parent.append(this.root);
    // Keep key presses in the panel (typing a level) from reaching the game's input.
    this.root.addEventListener('keydown', (e) => e.stopPropagation());
    this.root.addEventListener('keyup', (e) => e.stopPropagation());
  }

  update(s: PanelStats): void {
    const set = (k: string, v: string): void => {
      const node = this.readout.get(k);
      if (node) node.textContent = v;
    };
    set('tick', String(s.tick));
    set('core tick', `${s.tickMs.toFixed(2)} ms (max ${s.tickMaxMs.toFixed(2)})`);
    set('frame', `${s.frameMs.toFixed(1)} ms`);
    set('draw calls', String(s.calls));
    set('triangles', s.triangles.toLocaleString('en'));
    set('actors', String(s.actors.length));
    set('hash', s.hash);
    for (const [name, node] of this.counts) node.textContent = String(this.deps.draw.counts[name]);
    this.freeze.checked = this.deps.commands.isFrozen;
    this.actorCount.textContent = `actors (${s.actors.length})`;
    this.actorBody.replaceChildren(
      ...s.actors.map((a, i) =>
        el(
          'tr',
          {},
          el('td', { textContent: i === 0 ? 'player' : `#${a.id & 0xfffff}` }),
          el('td', { textContent: `${a.x.toFixed(1)}, ${a.y.toFixed(1)}, ${a.z.toFixed(1)}` }),
          el('td', { textContent: `${Math.round(a.hpFrac * 100)}%` })
        )
      )
    );
  }

  dispose(): void {
    this.root.remove();
    this.style.remove();
  }
}
