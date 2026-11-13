// Where the trailer clip lives. The clip is not recorded yet (no video asset is in the
// repository), so the phone view asks whether the file exists before it offers to play it.
// To add it, put an MP4 (and optionally a WebM) at these paths under `static/latticefall/`.

export const TRAILER_SOURCES = [
  { src: '/latticefall/trailer.webm', type: 'video/webm' },
  { src: '/latticefall/trailer.mp4', type: 'video/mp4' }
] as const;

/**
 * Whether a trailer file is served. A missing file can come back as the app's fallback page
 * with status 200, so the content type has to be a video one.
 */
export async function trailerPresent(): Promise<boolean> {
  for (const s of TRAILER_SOURCES) {
    try {
      const res = await fetch(s.src, { method: 'HEAD' });
      if (res.ok && (res.headers.get('content-type') ?? '').startsWith('video/')) return true;
    } catch {
      // Offline or blocked: treat the clip as missing.
    }
  }
  return false;
}
