// Pure touch maths: stick vector, drag scale, and the phone-screen test. No DOM access.

import { radialDeadzone } from '../frame';

/** Widest screen, in CSS pixels, that gets the phone codex view first. */
export const PHONE_MAX_WIDTH = 640;
/** Pixels the virtual stick travels from its anchor for full tilt. */
export const STICK_RADIUS = 56;
/** The stick's dead radius as a fraction of full tilt. Smaller than a gamepad's: no drift. */
export const STICK_DEADZONE = 0.1;
/** Camera drag counts for this many times the mouse's radians per pixel. */
export const LOOK_GAIN = 1.6;

/**
 * Turns a finger's offset from the stick anchor (screen pixels, y down) into the
 * camera-relative stick: x is right, y is forward. The length never exceeds 1.
 */
export function stickFromOffset(dx: number, dy: number, radius = STICK_RADIUS): [number, number] {
  if (radius <= 0) return [0, 0];
  return radialDeadzone(dx / radius, -dy / radius, STICK_DEADZONE);
}

/** Where the stick knob is drawn: the offset limited to `radius` from the anchor. */
export function clampKnob(dx: number, dy: number, radius = STICK_RADIUS): [number, number] {
  const len = Math.hypot(dx, dy);
  if (len <= radius || len === 0) return [dx, dy];
  return [(dx / len) * radius, (dy / len) * radius];
}

/** Whether a screen `width` gets the phone codex view. */
export function isPhoneWidth(width: number): boolean {
  return width > 0 && width <= PHONE_MAX_WIDTH;
}

/** `true` when the query string asks for touch mode, `false` when it opts out, else `null`. */
export function touchParam(params: URLSearchParams): boolean | null {
  const v = params.get('touch');
  if (v === '1') return true;
  if (v === '0') return false;
  return null;
}
