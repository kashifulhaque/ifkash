//! Deterministic math for the simulation core.
//!
//! Vector types come from `nalgebra` 0.35, the crate `rapier3d` 0.36.1 (the latest at the
//! time of writing) is built on. Rapier uses nalgebra, not
//! glam, so we match it. The `libm-force` feature routes nalgebra's own transcendental
//! functions through `libm`, mirroring Rapier's `enhanced-determinism` feature. SIMD
//! features are not enabled.

pub mod dmath;

pub use nalgebra::{Point3, Vector2, Vector3};

use dmath::{atan2, sqrt};

/// Distance in the XZ plane (Y is up).
pub fn distance_xz(a: &Vector3<f32>, b: &Vector3<f32>) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    sqrt(dx * dx + dz * dz)
}

/// Heading of a direction in the XZ plane, in radians from +Z toward +X.
pub fn heading_xz(dir: &Vector3<f32>) -> f32 {
    atan2(dir.x, dir.z)
}

/// Normalizes `v`, returning zero for a zero-length vector. Uses `libm` for the root.
pub fn normalize_or_zero(v: &Vector3<f32>) -> Vector3<f32> {
    let len = sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
    if len > 0.0 { v / len } else { Vector3::zeros() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers() {
        let a = Vector3::new(0.0, 5.0, 0.0);
        let b = Vector3::new(3.0, 9.0, 4.0);
        assert_eq!(distance_xz(&a, &b), 5.0);
        assert_eq!(normalize_or_zero(&Vector3::zeros()), Vector3::zeros());
        assert_eq!(heading_xz(&Vector3::new(0.0, 0.0, 1.0)), 0.0);
    }
}
