// Pure ActionFrame maths: no DOM, no device state.

/** Largest stick magnitude. -128 is never produced, so the stick is symmetric. */
export const STICK_MAX = 127;

/** Radial deadzone for gamepad sticks, in the range 0 to 1. */
export const DEADZONE = 0.15;

/**
 * Applies a radial deadzone and rescales so the output still reaches magnitude 1.
 * The result has length at most 1.
 */
export function radialDeadzone(x: number, y: number, dead = DEADZONE): [number, number] {
  const len = Math.hypot(x, y);
  if (len <= dead) return [0, 0];
  const scaled = Math.min((len - dead) / (1 - dead), 1);
  return [(x / len) * scaled, (y / len) * scaled];
}

/**
 * Converts camera-relative stick input to world axes.
 *
 * `right` is positive to the camera's right and `forward` is positive away from the camera.
 * `yaw` is the camera's yaw in radians. At yaw 0 the camera sits on the +Z side looking
 * toward -Z, so forward is world -Z and right is world +X. The camera orbits to
 * (sin yaw, cos yaw) around the player, so forward is (-sin yaw, -cos yaw) and
 * right is (cos yaw, -sin yaw).
 */
export function cameraRelative(right: number, forward: number, yaw: number): [number, number] {
  const s = Math.sin(yaw);
  const c = Math.cos(yaw);
  return [c * right - s * forward, -s * right - c * forward];
}

/** Quantizes an axis in [-1, 1] to the i8 range used by the ActionFrame. */
export function toAxisByte(v: number): number {
  return Math.max(-STICK_MAX, Math.min(STICK_MAX, Math.round(v * STICK_MAX)));
}

/** Packs an ActionFrame as `[move_x, move_z, buttons_lo, buttons_hi]`. */
export function packFrame(moveX: number, moveZ: number, buttons: number): Uint8Array {
  const out = new Uint8Array(4);
  // Uint8Array stores negatives as two's complement, which is the i8 wire format.
  out[0] = toAxisByte(moveX);
  out[1] = toAxisByte(moveZ);
  out[2] = buttons & 0xff;
  out[3] = (buttons >> 8) & 0xff;
  return out;
}
