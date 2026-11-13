import { entriesIn, entryOr404 } from '../../codexPages';
import type { EntryGenerator, PageLoad } from './$types';

// Prerendered: one static page per Chronicle Page in `content.ts`.
export const prerender = true;

export const entries: EntryGenerator = () => entriesIn('work').map((e) => ({ slug: e.slug }));

export const load: PageLoad = ({ params }) => ({ entry: entryOr404('work', params.slug) });
