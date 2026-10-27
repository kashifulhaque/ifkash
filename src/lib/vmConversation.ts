import { clip } from '$lib/vmFormat';
import type { TranscriptMessage } from '$lib/vmApi';

// Turns Claude's stream-json events (or stored transcript messages) into a flat
// list of renderable items. Tool results attach to their tool_use by id.

export type ToolResult = { text: string; isError: boolean };
export type ConvItem =
  | { id: number; kind: 'user'; text: string }
  | { id: number; kind: 'text'; text: string }
  | { id: number; kind: 'thinking'; text: string }
  | { id: number; kind: 'tool'; toolId: string; tool: string; input: any; summary: string; result?: ToolResult }
  | { id: number; kind: 'result'; isError: boolean; text: string; model: string | null; cost: number | null; turns: number | null; durationMs: number | null }
  | { id: number; kind: 'stderr'; text: string }
  | { id: number; kind: 'note'; text: string; isError: boolean };

let nextId = 1;
export const newId = () => nextId++;

// The model a run reported in its init event, shown later in its result footer.
const runModel = new WeakMap<ConvItem[], string>();

export function userItem(text: string): ConvItem {
  return { id: newId(), kind: 'user', text };
}

const str = (v: unknown) => (typeof v === 'string' ? v : '');

/** One-line summary of a tool call's input. */
export function toolSummary(name: string, input: any): string {
  if (!input || typeof input !== 'object') return str(input);
  const pick = (...keys: string[]) => {
    for (const k of keys) if (typeof input[k] === 'string' && input[k]) return input[k] as string;
    return '';
  };
  switch (name) {
    case 'Bash':
      return clip(pick('command'), 160);
    case 'Read':
    case 'Edit':
    case 'Write':
    case 'MultiEdit':
    case 'NotebookEdit':
      return pick('file_path', 'notebook_path');
    case 'Grep':
    case 'Glob':
      return clip(pick('pattern') + (input.path ? ` in ${input.path}` : ''), 160);
    case 'WebFetch':
      return pick('url');
    case 'WebSearch':
      return clip(pick('query'), 160);
    case 'Task':
    case 'Agent':
      return clip(pick('description', 'prompt'), 160);
  }
  const first = Object.values(input).find((v) => typeof v === 'string' && v) as string | undefined;
  return first ? clip(first, 160) : '';
}

function resultText(content: unknown): string {
  if (typeof content === 'string') return content;
  if (Array.isArray(content)) {
    return content
      .map((c: any) => (typeof c === 'string' ? c : c && c.type === 'text' ? str(c.text) : ''))
      .filter(Boolean)
      .join('\n');
  }
  return '';
}

/** Apply one run event, mutating `items`. Returns session id if the event carried one. */
export function applyEvent(items: ConvItem[], ev: any): string | null {
  if (!ev || typeof ev !== 'object') return null;
  switch (ev.type) {
    case 'system':
      if (ev.subtype !== 'init') return null;
      if (str(ev.model)) runModel.set(items, ev.model);
      return ev.session_id || null;
    case 'assistant': {
      const blocks = ev.message?.content;
      if (!Array.isArray(blocks)) return null;
      for (const b of blocks) {
        if (b.type === 'text' && str(b.text).trim()) {
          items.push({ id: newId(), kind: 'text', text: b.text });
        } else if (b.type === 'thinking' && str(b.thinking).trim()) {
          items.push({ id: newId(), kind: 'thinking', text: b.thinking });
        } else if (b.type === 'tool_use') {
          items.push({
            id: newId(),
            kind: 'tool',
            toolId: str(b.id),
            tool: str(b.name) || 'tool',
            input: b.input,
            summary: toolSummary(str(b.name), b.input)
          });
        }
      }
      return null;
    }
    case 'user': {
      const blocks = ev.message?.content;
      if (!Array.isArray(blocks)) return null;
      for (const b of blocks) {
        if (b.type !== 'tool_result') continue;
        const target = items.find((i) => i.kind === 'tool' && i.toolId === b.tool_use_id);
        const result = { text: resultText(b.content), isError: !!b.is_error };
        if (target && target.kind === 'tool') target.result = result;
        else items.push({ id: newId(), kind: 'note', text: result.text, isError: result.isError });
      }
      return null;
    }
    case 'result':
      items.push({
        id: newId(),
        kind: 'result',
        isError: !!ev.is_error,
        text: ev.is_error ? str(ev.result) : '',
        model: runModel.get(items) ?? null,
        cost: typeof ev.total_cost_usd === 'number' ? ev.total_cost_usd : null,
        turns: typeof ev.num_turns === 'number' ? ev.num_turns : null,
        durationMs: typeof ev.duration_ms === 'number' ? ev.duration_ms : null
      });
      return ev.session_id || null;
    case 'agent.stderr': {
      const last = items[items.length - 1];
      if (last && last.kind === 'stderr') last.text += '\n' + str(ev.text);
      else items.push({ id: newId(), kind: 'stderr', text: str(ev.text) });
      return null;
    }
    case 'agent.exit':
      if (ev.status && ev.status !== 'done') {
        items.push({
          id: newId(),
          kind: 'note',
          isError: ev.status === 'error',
          text: ev.status === 'stopped' ? 'Run stopped.' : `Run ended with an error (exit code ${ev.code ?? '?'}).`
        });
      }
      return null;
  }
  return null;
}

/** Build items from a stored transcript. */
export function fromMessages(messages: TranscriptMessage[]): ConvItem[] {
  const items: ConvItem[] = [];
  for (const m of messages) {
    if (m.kind === 'text') {
      items.push({ id: newId(), kind: m.role === 'user' ? 'user' : 'text', text: m.text });
    } else if (m.kind === 'thinking') {
      items.push({ id: newId(), kind: 'thinking', text: m.text });
    } else if (m.kind === 'tool_use') {
      items.push({ id: newId(), kind: 'tool', toolId: '', tool: m.tool || 'tool', input: null, summary: clip(m.text, 160) });
      const it = items[items.length - 1];
      if (it.kind === 'tool') it.input = m.text;
    } else if (m.kind === 'tool_result') {
      let target: ConvItem | undefined;
      for (let i = items.length - 1; i >= 0; i--) {
        const c = items[i];
        if (c.kind === 'tool') {
          if (!c.result) target = c;
          break;
        }
      }
      if (target && target.kind === 'tool') target.result = { text: m.text, isError: false };
      else items.push({ id: newId(), kind: 'note', text: m.text, isError: false });
    }
  }
  return items;
}
