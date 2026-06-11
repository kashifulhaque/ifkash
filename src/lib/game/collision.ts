export type AABB = { minX: number; maxX: number; minZ: number; maxZ: number };

// Pushes a circle (x, z, r) out of an AABB; returns corrected position.
export function resolveCircleAABB(
  x: number,
  z: number,
  r: number,
  box: AABB
): [number, number] {
  const cx = Math.max(box.minX, Math.min(x, box.maxX));
  const cz = Math.max(box.minZ, Math.min(z, box.maxZ));
  const dx = x - cx;
  const dz = z - cz;
  const d2 = dx * dx + dz * dz;
  if (d2 >= r * r) return [x, z];
  if (d2 > 1e-9) {
    const d = Math.sqrt(d2);
    return [cx + (dx / d) * r, cz + (dz / d) * r];
  }
  // Center inside the box: push out along the shallowest axis.
  const left = x - box.minX + r;
  const right = box.maxX - x + r;
  const near = z - box.minZ + r;
  const far = box.maxZ - z + r;
  const m = Math.min(left, right, near, far);
  if (m === left) return [box.minX - r, z];
  if (m === right) return [box.maxX + r, z];
  if (m === near) return [x, box.minZ - r];
  return [x, box.maxZ + r];
}
