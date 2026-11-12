// Browser replay recording. Every tick's ActionFrame is appended to an in-memory buffer;
// the WASM module (`encode_replay`) turns it into a `.lfr` file, so the format has one
// implementation: lf-core's.

/** Bytes per ActionFrame. */
const FRAME_BYTES = 4;

export class ReplayRecorder {
  private buf = new Uint8Array(FRAME_BYTES * 1024);
  private len = 0;

  /** Ticks recorded so far. */
  get ticks(): number {
    return this.len / FRAME_BYTES;
  }

  /** Appends the 4-byte ActionFrame that was fed to one tick. */
  record(frame: Uint8Array): void {
    if (this.len + FRAME_BYTES > this.buf.length) {
      const bigger = new Uint8Array(this.buf.length * 2);
      bigger.set(this.buf.subarray(0, this.len));
      this.buf = bigger;
    }
    this.buf.set(frame.subarray(0, FRAME_BYTES), this.len);
    this.len += FRAME_BYTES;
  }

  /** A copy of the recorded frames, 4 bytes per tick. */
  frames(): Uint8Array {
    return this.buf.slice(0, this.len);
  }

  clear(): void {
    this.len = 0;
  }
}

/** The subset of the lf-wasm glue the recorder needs. */
export interface ReplayEncoder {
  encode_replay(seed: bigint, scenario: string, frames: Uint8Array): Uint8Array;
}

/** Encodes the recording as `.lfr` bytes for the world started with `seed` and `scenario`. */
export function encodeReplay(
  glue: ReplayEncoder,
  recorder: ReplayRecorder,
  seed: bigint,
  scenario: string
): Uint8Array {
  return glue.encode_replay(seed, scenario, recorder.frames());
}

/** Offers bytes as a file download. Browser only. */
export function downloadBytes(bytes: Uint8Array, filename: string): void {
  const url = URL.createObjectURL(new Blob([bytes as BlobPart], { type: 'application/octet-stream' }));
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
