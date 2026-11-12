// The dialogue's view of the simulation (WP-068): plain objects parsed from the floats and JSON
// the core exports (`dialogue_state`, `npc_json`, `station_json` in game/crates/wasm/src/lib.rs),
// and the events that open and close a conversation. Pure and read-only, with no Three.js or
// WASM imports. The simulation owns the conversation and the quests; nothing here writes them.

import { EVENT_WORDS } from '../../engine/events';

/** `queue_command` kinds of the dialogue commands (game/crates/wasm/src/menu.rs). */
export const DIALOGUE_COMMAND = {
  Ask: 40,
  EndTalk: 41,
  Appearance: 42
} as const;

/** The topic number of the quest line, and of "back to the menu" (`lf_content::npc`). */
export const QUEST_TOPIC = 254;
export const BACK_TOPIC = 255;

/** `*` as a slug names every item of the section. */
export const ALL_SLUGS = '*';

export interface FactRef {
  section: string;
  slug: string;
}

export interface Topic {
  id: string;
  label: string;
  line: string;
  facts: FactRef[];
  service?: 'Respec' | 'Upgrade' | 'Shop' | null;
}

export interface StateLines {
  before: string;
  during: string;
  after: string;
}

export interface QuestDef {
  id: string;
  name: string;
  label: string;
  replies: StateLines;
  needs: Array<{ Flag: string } | { Item: string }>;
  at_least: number;
  done_line: string;
  reward: { flags: string[]; codex: string[]; items: string[] };
}

export interface NpcDef {
  id: string;
  name: string;
  role: string;
  greeting: StateLines;
  farewell: string;
  topics: Topic[];
  quest?: QuestDef | null;
}

export type StationKind = 'Archive' | 'Ledger' | 'Mirror' | 'Keepsakes' | 'Hearth';

export interface StationDef {
  id: string;
  kind: StationKind;
  name: string;
  line: string;
  facts: FactRef[];
  unmapped_only: boolean;
  presets: string[];
  dyes: Array<{ name: string; unlock_flag?: string | null }>;
}

/** Floats before the per-NPC quest states in `dialogue_state`. Must equal `STATE_FIXED_WORDS`. */
export const DIALOGUE_FIXED_WORDS = 12;

/** Which quest reply is showing, from the core's `Reply`. */
export const REPLY = { None: 0, Offer: 1, Reminder: 2, Done: 3, After: 4 } as const;

export interface DialogueState {
  open: boolean;
  /** What is open: an NPC or a station. */
  kind: 'npc' | 'station';
  index: number;
  /** 0 the menu, a topic's index plus one, or `QUEST_TOPIC`. */
  topic: number;
  reply: number;
  /** The open NPC's quest: 0 before, 1 during, 2 after. */
  questState: number;
  questHave: number;
  questRequired: number;
  questTotal: number;
  mirrorPreset: number;
  mirrorDye: number;
  /** Bit i is set when the Mirror's dye i is owned. */
  dyeMask: number;
  /** Every NPC's quest state, in the scenario's order. */
  npcQuests: number[];
}

export const emptyDialogueState = (): DialogueState => ({
  open: false,
  kind: 'npc',
  index: 0,
  topic: 0,
  reply: 0,
  questState: 0,
  questHave: 0,
  questRequired: 0,
  questTotal: 0,
  mirrorPreset: 0,
  mirrorDye: 0,
  dyeMask: 0,
  npcQuests: []
});

export function parseDialogueState(words: ArrayLike<number>): DialogueState {
  if (words.length < DIALOGUE_FIXED_WORDS) return emptyDialogueState();
  return {
    open: words[0] > 0.5,
    kind: words[1] > 0.5 ? 'station' : 'npc',
    index: Math.round(words[2]),
    topic: Math.round(words[3]),
    reply: Math.round(words[4]),
    questState: Math.round(words[5]),
    questHave: Math.round(words[6]),
    questRequired: Math.round(words[7]),
    questTotal: Math.round(words[8]),
    mirrorPreset: Math.round(words[9]),
    mirrorDye: Math.round(words[10]),
    dyeMask: Math.round(words[11]),
    npcQuests: Array.from({ length: words.length - DIALOGUE_FIXED_WORDS }, (_, i) =>
      Math.round(words[DIALOGUE_FIXED_WORDS + i])
    )
  };
}

/** Parses an NPC or station definition; null for the empty string the core returns for no such entry. */
export function parseDef<T>(json: string): T | null {
  if (!json) return null;
  try {
    return JSON.parse(json) as T;
  } catch {
    return null;
  }
}

/** The line an NPC says first, by the state of its quest. */
export function greetingFor(npc: NpcDef, questState: number): string {
  const g = npc.greeting;
  return questState === 0 ? g.before : questState === 1 ? g.during : g.after;
}

/** The quest reply showing, as text; empty when the player has not asked about the quest. */
export function questReply(npc: NpcDef, reply: number): string {
  const q = npc.quest;
  if (!q) return '';
  switch (reply) {
    case REPLY.Offer:
      return q.replies.before;
    case REPLY.Reminder:
      return q.replies.during;
    case REPLY.Done:
      return q.done_line;
    case REPLY.After:
      return q.replies.after;
    default:
      return '';
  }
}

// --- Events -----------------------------------------------------------------------------

export const EVENT_DIALOGUE = 31;
export const EVENT_QUEST = 32;
export const EVENT_APPEARANCE = 33;

export type DialogueEvent =
  | { kind: 'dialogue'; what: 'opened' | 'closed' | 'topic'; target: 'npc' | 'station'; index: number; topic: number }
  | { kind: 'quest'; npc: number; state: number }
  | { kind: 'appearance'; preset: number; dye: number };

const WHAT = ['opened', 'closed', 'topic'] as const;

/** Decodes the dialogue events (kinds 31 to 33) in a drained word list, in order. */
export function dialogueEventsFromWords(words: ArrayLike<number>): DialogueEvent[] {
  const out: DialogueEvent[] = [];
  let i = 0;
  while (i < words.length) {
    const kind = words[i];
    const len = EVENT_WORDS[kind];
    if (len === undefined || i + len > words.length) break;
    if (kind === EVENT_DIALOGUE) {
      out.push({
        kind: 'dialogue',
        what: WHAT[words[i + 1]] ?? 'topic',
        target: words[i + 2] === 1 ? 'station' : 'npc',
        index: words[i + 3],
        topic: words[i + 4]
      });
    } else if (kind === EVENT_QUEST) {
      out.push({ kind: 'quest', npc: words[i + 1], state: words[i + 2] });
    } else if (kind === EVENT_APPEARANCE) {
      out.push({ kind: 'appearance', preset: words[i + 1], dye: words[i + 2] });
    }
    i += len;
  }
  return out;
}
