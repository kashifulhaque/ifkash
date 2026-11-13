// Photo mode's free camera and saved image (WP-088). See docs/game/06-progression-and-systems.md,
// "Gestures and photo mode": the game pauses, the camera is free within 15 m of the player, the
// HUD can be hidden, three color grades are on offer, and saved images carry a small
// `ifkash.dev` watermark. Pure math and canvas helpers: no Three.js or WASM imports, and
// nothing here writes simulation state (photo mode sends no commands and takes no ticks).

/** How far from the player the camera may go, in meters (spec: 15 m). */
export const PHOTO_RADIUS_M = 15;
/** The camera stays this far above the player's feet or higher, so it can't sink into the floor. */
export const PHOTO_FLOOR_M = 0.3;
/** The text of the watermark. */
export const WATERMARK = 'ifkash.dev';

export interface PhotoPose {
  x: number;
  y: number;
  z: number;
  /** Radians from +Z toward +X. */
  yaw: number;
  /** Radians below the horizon (positive looks down). */
  pitch: number;
}

export interface Point3 {
  x: number;
  y: number;
  z: number;
}

/** What the engine gives photo mode: it pauses the world and holds the camera. */
export interface PhotoHost {
  /** Pauses the game and starts the free camera at the current view; null if there is no player. */
  begin(): { anchor: Point3; pose: PhotoPose } | null;
  /** Moves the camera. */
  set(pose: PhotoPose): void;
  /** Hands the camera back and resumes the game. */
  end(): void;
  /** The image as a PNG, with the grade and watermark applied; null if the browser can't make one. */
  capture(gradeId: string, watermark: boolean): Promise<Blob | null>;
}

export interface Grade {
  id: string;
  name: string;
  /** A CSS `filter` value: the live preview uses it on the canvas and the saved image uses it too. */
  filter: string;
}

/** Natural, then the three color grades (spec: three). */
export const GRADES: readonly Grade[] = [
  { id: 'natural', name: 'Natural', filter: 'none' },
  { id: 'ember', name: 'Ember', filter: 'sepia(0.4) saturate(1.35) contrast(1.05)' },
  { id: 'frost', name: 'Frost', filter: 'sepia(0.25) hue-rotate(160deg) saturate(1.1) brightness(1.04)' },
  { id: 'ash', name: 'Ash', filter: 'grayscale(1) contrast(1.15)' }
];

export function gradeFilter(id: string): string {
  return (GRADES.find((g) => g.id === id) ?? GRADES[0]).filter;
}

/** Keeps the camera within [`PHOTO_RADIUS_M`] of the anchor (the player) and above the floor. */
export function clampPose(pose: PhotoPose, anchor: Point3): PhotoPose {
  let { x, y, z } = pose;
  y = Math.max(y, anchor.y + PHOTO_FLOOR_M);
  const dx = x - anchor.x;
  const dy = y - anchor.y;
  const dz = z - anchor.z;
  const d = Math.hypot(dx, dy, dz);
  if (d > PHOTO_RADIUS_M) {
    const k = PHOTO_RADIUS_M / d;
    x = anchor.x + dx * k;
    y = anchor.y + dy * k;
    z = anchor.z + dz * k;
  }
  const pitch = Math.max(-1.45, Math.min(1.45, pose.pitch));
  return { x, y, z, yaw: pose.yaw, pitch };
}

/** What the player is asking the camera to do this frame, each axis from -1 to 1. */
export interface PhotoInput {
  /** Along the view direction. */
  forward: number;
  /** Sideways, positive to the right. */
  right: number;
  /** Straight up, positive up. */
  up: number;
}

/** Camera speed in meters per second. */
export const PHOTO_SPEED = 4;

/** Moves the pose by `input` for `dt` seconds, then clamps it. */
export function stepPose(pose: PhotoPose, input: PhotoInput, dt: number, anchor: Point3, speed = PHOTO_SPEED): PhotoPose {
  const cp = Math.cos(pose.pitch);
  // The view direction, and the right-hand side on the ground (looking along +Z, right is -X).
  const fx = Math.sin(pose.yaw) * cp;
  const fy = -Math.sin(pose.pitch);
  const fz = Math.cos(pose.yaw) * cp;
  const rx = -Math.cos(pose.yaw);
  const rz = Math.sin(pose.yaw);
  const k = speed * dt;
  return clampPose(
    {
      ...pose,
      x: pose.x + (fx * input.forward + rx * input.right) * k,
      y: pose.y + (fy * input.forward + input.up) * k,
      z: pose.z + (fz * input.forward + rz * input.right) * k
    },
    anchor
  );
}

/** Looks around by `dyaw` and `dpitch` radians. */
export function lookPose(pose: PhotoPose, dyaw: number, dpitch: number): PhotoPose {
  return { ...pose, yaw: pose.yaw + dyaw, pitch: Math.max(-1.45, Math.min(1.45, pose.pitch + dpitch)) };
}

/**
 * Draws the game canvas onto a new canvas with the grade, adds the watermark, and encodes a PNG.
 * Call it right after the canvas was drawn (the engine does, from its render callback): a WebGL
 * canvas can only be read in the task that drew it. A browser without canvas filters (older
 * Safari) saves the image without the grade.
 */
export function composeImage(source: HTMLCanvasElement, filter: string, watermark: boolean): Promise<Blob | null> {
  const out = document.createElement('canvas');
  out.width = source.width;
  out.height = source.height;
  const ctx = out.getContext('2d');
  if (!ctx) return Promise.resolve(null);
  if (filter !== 'none' && 'filter' in ctx) ctx.filter = filter;
  ctx.drawImage(source, 0, 0);
  ctx.filter = 'none';
  if (watermark) {
    const size = Math.max(12, Math.round(out.height / 48));
    ctx.font = `${size}px ui-monospace, SFMono-Regular, Menlo, monospace`;
    ctx.textAlign = 'right';
    ctx.textBaseline = 'bottom';
    const pad = Math.round(size * 0.9);
    ctx.fillStyle = 'rgba(0, 0, 0, 0.35)';
    ctx.fillText(WATERMARK, out.width - pad + 1, out.height - pad + 1);
    ctx.fillStyle = 'rgba(255, 255, 255, 0.78)';
    ctx.fillText(WATERMARK, out.width - pad, out.height - pad);
  }
  return new Promise((resolve) => out.toBlob((b) => resolve(b), 'image/png'));
}

/** `cinderheart-20260102-030405.png`, from the wall clock (a file name, not game time). */
export function photoFileName(now = new Date()): string {
  const two = (n: number) => String(n).padStart(2, '0');
  const d = `${now.getFullYear()}${two(now.getMonth() + 1)}${two(now.getDate())}`;
  const t = `${two(now.getHours())}${two(now.getMinutes())}${two(now.getSeconds())}`;
  return `cinderheart-${d}-${t}.png`;
}
