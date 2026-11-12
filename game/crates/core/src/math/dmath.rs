//! `f32` transcendental wrappers over `libm`. Gameplay code must call these instead of
//! the `std` methods, which can differ across platforms.

pub fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

pub fn cos(x: f32) -> f32 {
    libm::cosf(x)
}

pub fn atan2(y: f32, x: f32) -> f32 {
    libm::atan2f(y, x)
}

pub fn sqrt(x: f32) -> f32 {
    libm::sqrtf(x)
}

pub fn exp(x: f32) -> f32 {
    libm::expf(x)
}

pub fn powf(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

/// Linear interpolation. Deliberately two separate operations: no `mul_add`, because
/// fused multiply-add rounds differently across targets.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(sqrt(9.0), 3.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(sin(0.0), 0.0);
        assert_eq!(cos(0.0), 1.0);
        assert_eq!(exp(0.0), 1.0);
        assert_eq!(powf(2.0, 3.0), 8.0);
        assert_eq!(atan2(0.0, 1.0), 0.0);
    }
}
