// The fixed message vocabulary. Players build a message from a template and up to two
// words, so there is no free text to moderate. The server stores only the IDs and checks
// them against these bounds (templates 0-63, words 0-511). Append only: an ID's meaning
// must never change once shipped.

/** `[word]` marks a slot. The first template has no slot. */
export const MESSAGE_TEMPLATES: readonly string[] = [
  'Rest ahead.',
  'Try [word] ahead.',
  'Beware of [word].',
  'Look out for [word].',
  'Needs [word].',
  'Time for [word].',
  'Still no [word]...',
  'Praise [word]!',
  'If only I had [word]...',
  'Could this be [word]?',
  'Well met, [word].',
  'Did you see [word]?'
];

export const MESSAGE_WORDS: readonly string[] = [
  'a ledge',
  'a hidden path',
  'a shortcut',
  'a Lantern',
  'a trap',
  'an ambush',
  'a roll',
  'a dodge',
  'a block',
  'a parry',
  'a long jump',
  'a ranged attack',
  'patience',
  'courage',
  'a rest',
  'a second look',
  'light',
  'shadow',
  'water',
  'fire',
  'the left path',
  'the right path',
  'above',
  'below',
  'a treasure',
  'a relic',
  'a friend',
  'a foe',
  'a tough foe',
  'a boss',
  'a weak spot',
  'a gap',
  'a bridge',
  'a door',
  'a lever',
  'a ladder',
  'a view',
  'a nap',
  'a gesture',
  'good fortune'
];

/** Renders a message from its IDs, or `null` when an ID is unknown. */
export function renderMessage(templateId: number, wordIds: readonly number[]): string | null {
  const template = MESSAGE_TEMPLATES[templateId];
  if (template === undefined) return null;
  const words = wordIds.map((id) => MESSAGE_WORDS[id]);
  if (words.some((w) => w === undefined)) return null;
  if (!template.includes('[word]')) return template;
  if (words.length === 0) return null;
  let i = 0;
  return template.replace(/\[word\]/g, () => words[Math.min(i++, words.length - 1)]);
}

/** Whether the IDs fit the server's bounds and the template's slot count. */
export function isValidMessage(templateId: number, wordIds: readonly number[]): boolean {
  return renderMessage(templateId, wordIds) !== null && wordIds.length <= 2;
}
