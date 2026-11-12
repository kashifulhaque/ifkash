// Where the résumé PDF lives, shared by the landing card and the Ledger. No Three.js or WASM
// import. In dev the Worker runs on :8787, so the path resolves to its full URL there.

import { dev } from '$app/environment';
import { resumeDevUrl, resumePath, sections } from '$lib/content';

export function resumeHref(): string {
  return dev ? resumeDevUrl : resumePath;
}

/** The `contact` section's entries (docs/game/08-portfolio-integration.md, "Contact"). */
export function contactItems() {
  return sections.find((s) => s.id === 'contact')?.items ?? [];
}
