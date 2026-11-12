// The dialogue's side of the engine (WP-068): what it reads from the core and the commands it
// sends. No Three.js or WASM imports; `LfWorld` satisfies `DialogueWorld` structurally. The
// dialogue changes the world only through `command`, which queues a core `Command` (see
// `queue_command` in game/crates/wasm/src/lib.rs). Conversation state and quest flags live in
// the simulation, so a saved game, a replay, and an RL agent all see the same dialogue.

import {
  BACK_TOPIC,
  DIALOGUE_COMMAND,
  QUEST_TOPIC,
  parseDef,
  parseDialogueState,
  type DialogueEvent,
  type DialogueState,
  type NpcDef,
  type StationDef
} from './dialogueState';

/** The part of `LfWorld` the dialogue reads. */
export interface DialogueWorld {
  dialogue_state(): Float32Array;
  npc_json(index: number): string;
  station_json(index: number): string;
}

/** How the engine reports to the dialogue UI. */
export interface DialogueSink {
  /** A conversation event from the core (opened, closed, topic, quest, appearance). */
  event(e: DialogueEvent): void;
}

export interface DialogueHost {
  read(): DialogueState;
  /** The NPC at `index` of the scenario's NPCs; definitions are cached, since they never change. */
  npc(index: number): NpcDef | null;
  station(index: number): StationDef | null;
  /** Asks the open NPC about topic `index` of its menu. */
  ask(index: number): void;
  /** Takes up or turns in the open NPC's quest. */
  askQuest(): void;
  /** Back to the open NPC's menu. */
  back(): void;
  /** Ends the conversation. */
  end(): void;
  /** Sets the Mirror's preset and dye. */
  appearance(preset: number, dye: number): void;
}

export function createDialogueHost(
  world: DialogueWorld,
  sink: { command(kind: number, a: number, b: number): void }
): DialogueHost {
  const npcs = new Map<number, NpcDef | null>();
  const stations = new Map<number, StationDef | null>();
  const send = (kind: number, a = 0, b = 0) => sink.command(kind, a, b);
  return {
    read: () => parseDialogueState(world.dialogue_state()),
    npc(index) {
      if (!npcs.has(index)) npcs.set(index, parseDef<NpcDef>(world.npc_json(index)));
      return npcs.get(index) ?? null;
    },
    station(index) {
      if (!stations.has(index)) stations.set(index, parseDef<StationDef>(world.station_json(index)));
      return stations.get(index) ?? null;
    },
    ask: (i) => send(DIALOGUE_COMMAND.Ask, i),
    askQuest: () => send(DIALOGUE_COMMAND.Ask, QUEST_TOPIC),
    back: () => send(DIALOGUE_COMMAND.Ask, BACK_TOPIC),
    end: () => send(DIALOGUE_COMMAND.EndTalk),
    appearance: (preset, dye) => send(DIALOGUE_COMMAND.Appearance, preset, dye)
  };
}
