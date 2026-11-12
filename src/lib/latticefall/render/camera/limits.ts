// Camera constants and user settings. Pure: no Three.js, so the input layer can import the
// pitch clamps. Numbers come from docs/game/07-presentation.md, "Camera".

const DEG = Math.PI / 180;

/** Free-camera pitch clamps. Positive pitch puts the camera above the pivot, looking down. */
export const PITCH_MIN = -35 * DEG;
export const PITCH_MAX = 60 * DEG;
/** Where a new camera starts: a little above and behind the player. */
export const PITCH_DEFAULT = 20 * DEG;

/** Height of the orbit pivot above the player's feet, in meters. */
export const PIVOT_HEIGHT = 1.6;

export interface CameraSettings {
  /** Vertical field of view in degrees. */
  fov: number;
  /** Orbit distance from the pivot in meters. */
  distance: number;
}

export const DEFAULT_CAMERA_SETTINGS: CameraSettings = { fov: 60, distance: 4.5 };
export const FOV_RANGE: readonly [number, number] = [40, 100];
export const DISTANCE_RANGE: readonly [number, number] = [3, 8];

const STORAGE_KEY = 'lf-camera';

function clamp(v: number, [lo, hi]: readonly [number, number], fallback: number): number {
  return Number.isFinite(v) ? Math.min(hi, Math.max(lo, v)) : fallback;
}

/** Returns `s` with both fields clamped to their ranges. */
export function clampSettings(s: Partial<CameraSettings>): CameraSettings {
  return {
    fov: clamp(s.fov ?? NaN, FOV_RANGE, DEFAULT_CAMERA_SETTINGS.fov),
    distance: clamp(s.distance ?? NaN, DISTANCE_RANGE, DEFAULT_CAMERA_SETTINGS.distance)
  };
}

export function loadCameraSettings(): CameraSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return clampSettings(JSON.parse(raw) as Partial<CameraSettings>);
  } catch {
    // Storage is unavailable or the value is corrupt: use the defaults.
  }
  return { ...DEFAULT_CAMERA_SETTINGS };
}

export function saveCameraSettings(s: CameraSettings): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(clampSettings(s)));
  } catch {
    // Not persisted; the settings still apply for this session.
  }
}
