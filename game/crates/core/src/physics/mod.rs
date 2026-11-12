//! Physics: the static level in Rapier, and the queries the character controller and the
//! hosts run against it. See docs/game/01-architecture.md, "Physics".
//!
//! Rapier is built with `enhanced-determinism` and without SIMD features (see
//! `lf-core/Cargo.toml`). The level is static: one optional heightfield plus box kit
//! pieces, inserted once when the world is built. There are no dynamic bodies, and the
//! physics pipeline never steps; the broad phase is built once and only queried.
//!
//! Actors are not inserted into the Rapier sets yet. Each actor's capsule is a kinematic
//! shape that `KinematicCharacterController` moves against the level (see
//! `actor::controller`). Actor-versus-actor collision arrives with enemies.

use lf_content::Scenario;
use rapier3d::parry::utils::Array2;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

/// The physics world. Clones and serializes with `World`. It is not part of the state
/// hash: it is rebuilt from the scenario, which the content hash already covers, and
/// Rapier's serialized form includes hash-map ordering that isn't canonical.
#[derive(Clone, Serialize, Deserialize)]
pub struct PhysicsState {
    /// Scenario name, for example `test_field`.
    pub scenario: String,
    /// Feet below this height kill an actor.
    pub kill_y: f32,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
}

impl core::fmt::Debug for PhysicsState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhysicsState")
            .field("scenario", &self.scenario)
            .field("colliders", &self.colliders.len())
            .finish_non_exhaustive()
    }
}

fn vec3(v: [f32; 3]) -> Vector {
    Vector::new(v[0], v[1], v[2])
}

impl PhysicsState {
    /// Builds the static level of `scenario`.
    pub fn new(name: &str, scenario: &Scenario) -> Self {
        let bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        if let Some(h) = &scenario.heightfield {
            let rows = h.rows as usize;
            let cols = h.cols as usize;
            // Parry's `Array2` is column-major with rows along Z and columns along X;
            // the RON is row-major.
            let mut data = Vec::with_capacity(rows * cols);
            for col in 0..cols {
                for row in 0..rows {
                    data.push(h.heights[row * cols + col]);
                }
            }
            let heights = Array2::new(rows, cols, data);
            // Parry scales the unit square [-0.5, 0.5] by `scale`, so x and z are the total
            // extents; y multiplies the stored heights.
            let scale = Vector::new(h.size_x, 1.0, h.size_z);
            colliders
                .insert(ColliderBuilder::heightfield(heights, scale).translation(vec3(h.center)));
        }
        for b in &scenario.boxes {
            let [hx, hy, hz] = b.half_extents;
            colliders.insert(ColliderBuilder::cuboid(hx, hy, hz).translation(vec3(b.center)));
        }
        let mut broad_phase = BroadPhaseBvh::new();
        let narrow_phase = NarrowPhase::new();
        let handles: Vec<ColliderHandle> = colliders.iter().map(|(h, _)| h).collect();
        let mut events = Vec::new();
        broad_phase.update(
            &IntegrationParameters::default(),
            &colliders,
            &bodies,
            &handles,
            &[],
            &mut events,
        );
        Self {
            scenario: name.to_owned(),
            kill_y: scenario.kill_y,
            bodies,
            colliders,
            broad_phase,
            narrow_phase,
        }
    }

    /// A query view over the level for the character controller and ray casts.
    pub fn queries(&self) -> QueryPipeline<'_> {
        self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            QueryFilter::default(),
        )
    }

    /// Casts a ray against the level. Returns the distance along `dir` (which need not be
    /// normalized; the distance is in units of its length) to the first hit within `max`.
    pub fn raycast(&self, origin: [f32; 3], dir: [f32; 3], max: f32) -> Option<f32> {
        let ray = Ray::new(vec3(origin), vec3(dir));
        self.queries().cast_ray(&ray, max, true).map(|(_, toi)| toi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lf_content::Content;

    fn field() -> PhysicsState {
        let content = Content::load_embedded().unwrap();
        PhysicsState::new("test_field", content.scenario("test_field").unwrap())
    }

    #[test]
    fn heightfield_matches_the_ron_layout() {
        let p = field();
        let down = [0.0, -1.0, 0.0];
        // Flat ground at the spawn.
        let d = p.raycast([0.0, 20.0, 0.0], down, 100.0).unwrap();
        assert!((20.0 - d).abs() < 1e-4, "{d}");
        // Ramp: 0 m at z = 10, rising 1 m per 2 m.
        // (Probe between vertices: a vertical ray exactly through a heightfield vertex can
        // miss in Parry 0.31.)
        let d = p.raycast([1.0, 20.0, 21.0], down, 100.0).unwrap();
        assert!((20.0 - d - 5.5).abs() < 1e-4, "{d}");
        // Ledge top at 10 m.
        let d = p.raycast([0.0, 20.0, 35.0], down, 100.0).unwrap();
        assert!((20.0 - d - 10.0).abs() < 1e-4, "{d}");
        // Steep ridge at x = 24, z = -5.
        let d = p.raycast([24.5, 20.0, -5.0], down, 100.0).unwrap();
        assert!((20.0 - d - 4.0).abs() < 1e-4, "{d}");
        // Off the field: nothing.
        assert!(p.raycast([100.0, 20.0, 0.0], down, 100.0).is_none());
    }

    #[test]
    fn clones_and_serializes() {
        let p = field();
        let q = p.clone();
        assert_eq!(
            p.raycast([1.0, 20.0, 21.0], [0.0, -1.0, 0.0], 100.0),
            q.raycast([1.0, 20.0, 21.0], [0.0, -1.0, 0.0], 100.0)
        );
        let bytes = postcard::to_allocvec(&p).unwrap();
        let back: PhysicsState = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(
            back.raycast([0.0, 20.0, 35.0], [0.0, -1.0, 0.0], 100.0),
            p.raycast([0.0, 20.0, 35.0], [0.0, -1.0, 0.0], 100.0)
        );
    }
}
