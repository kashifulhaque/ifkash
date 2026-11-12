//! Hurtboxes: the capsule an actor can be hit on. See docs/game/01-architecture.md,
//! "Physics".
//!
//! Every actor has one upright capsule standing on its feet. The player's and the
//! training dummies' come from the character controller capsule in
//! `game/data/locomotion.ron` (`capsule_radius`, `capsule_half_height`), so the body you
//! hit is the body that walks. Enemies with their own sizes set `Actor::hurtbox` when
//! they spawn.

use lf_content::Locomotion;
use serde::{Deserialize, Serialize};

use super::hitbox::{Capsule, V3};

/// An upright capsule whose bottom touches the actor's feet. Simulation state: part of
/// the state hash. The default is a zero-size capsule, a point at the feet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hurtbox {
    pub radius: f32,
    /// Half the length of the capsule's straight middle part; the capsule is
    /// `2 * (half_height + radius)` tall.
    pub half_height: f32,
}

impl Hurtbox {
    /// The character controller capsule.
    pub fn from_locomotion(t: &Locomotion) -> Self {
        Self {
            radius: t.capsule_radius,
            half_height: t.capsule_half_height,
        }
    }

    /// The capsule in world space for an actor with its feet at `feet`.
    pub fn capsule(&self, feet: V3) -> Capsule {
        let low = feet[1] + self.radius;
        Capsule {
            a: [feet[0], low, feet[2]],
            b: [feet[0], low + 2.0 * self.half_height, feet[2]],
            radius: self.radius,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_stands_on_the_feet() {
        let h = Hurtbox {
            radius: 0.35,
            half_height: 0.55,
        };
        let c = h.capsule([1.0, 2.0, 3.0]);
        assert_eq!(c.a, [1.0, 2.35, 3.0]);
        assert!((c.b[1] - 3.45).abs() < 1e-6);
        // 1.8 m tall: the top of the capsule is 1.8 m above the feet.
        assert!((c.b[1] + c.radius - 2.0 - 1.8).abs() < 1e-6);
    }
}
