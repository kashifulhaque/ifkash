//! Developer commands for the browser dev tools (`src/lib/latticefall/dev/`).
//!
//! This module only exists when the crate is built with the `dev` feature, so release
//! builds of the module cannot teleport the player or make them invulnerable. The
//! commands live here, in the host, rather than in `lf-core`: they edit the world through
//! its public fields and add no `Command` variant, so simulation behavior and golden
//! hashes are untouched.
//!
//! A dev command changes state that a replay's recorded inputs don't contain. A session
//! that used one can no longer be reproduced from its `.lfr` file.

use crate::LfWorld;
use lf_core::snapshot::write_snapshot;
use wasm_bindgen::prelude::*;

/// Highest level `dev_set_level` accepts. A sanity bound for the dev tools, not a design value.
pub const MAX_DEV_LEVEL: u32 = 999;

/// State the dev tools keep beside the world. Never hashed.
#[derive(Debug)]
pub struct DevState {
    /// God mode: the player's health and stamina refill after every tick.
    pub god: bool,
    /// The level the dev tools last set. The core has no character level yet, so this
    /// only records the value until the leveling system lands.
    pub level: u32,
}

impl Default for DevState {
    fn default() -> Self {
        Self {
            god: false,
            level: 1,
        }
    }
}

#[wasm_bindgen]
impl LfWorld {
    /// Moves the player's feet to `(x, y, z)` facing `yaw` (radians from +Z toward +X),
    /// clears their velocity, and refreshes the snapshot. Returns false if the player
    /// is gone or a coordinate isn't finite.
    pub fn dev_teleport(&mut self, x: f32, y: f32, z: f32, yaw: f32) -> bool {
        if ![x, y, z, yaw].iter().all(|v| v.is_finite()) {
            return false;
        }
        let Some(player) = self.world.actors.get_mut(self.world.player) else {
            return false;
        };
        player.pos = [x, y, z];
        player.yaw = yaw;
        player.vel = [0.0; 3];
        // Stand the player on the spot so the move isn't counted as a fall.
        player.motor.grounded = true;
        player.motor.jumped = false;
        player.motor.landing = 0;
        player.motor.jump_startup = 0;
        player.motor.ground_y = y;
        player.motor.fall_peak_y = y;
        write_snapshot(&self.world, &mut self.snapshot);
        true
    }

    /// Turns god mode on or off.
    pub fn dev_set_god(&mut self, on: bool) {
        self.dev.god = on;
    }

    /// Whether god mode is on.
    pub fn dev_god(&self) -> bool {
        self.dev.god
    }

    /// Records the character level the dev tools want and returns the clamped value
    /// (1 to `MAX_DEV_LEVEL`). Nothing in the core reads it yet.
    pub fn dev_set_level(&mut self, level: u32) -> u32 {
        self.dev.level = level.clamp(1, MAX_DEV_LEVEL);
        self.dev.level
    }

    /// The level last set with `dev_set_level`.
    pub fn dev_level(&self) -> u32 {
        self.dev.level
    }

    /// Gives the player `n` of the item or weapon with registry number `num` (a weapon, armor
    /// piece, or charm goes into the owned gear; a consumable goes on its stack) and returns
    /// how many were taken. For trying the menus; there is no world pickup yet.
    pub fn dev_give_item(&mut self, num: u32, n: u32) -> u32 {
        let id = self.world.player;
        let Some(p) = self.world.actors.get_mut(id) else {
            return 0;
        };
        if self.world.items.is_weapon(num) {
            return u32::from(p.inventory.give_gear(num));
        }
        let n = u16::try_from(n).unwrap_or(u16::MAX);
        u32::from(p.inventory.pick_up(num, n, &self.world.items))
    }

    /// Sets the Glimmer the player holds, for trying the level-up menu.
    pub fn dev_set_glimmer(&mut self, glimmer: u32) {
        let id = self.world.player;
        if let Some(p) = self.world.actors.get_mut(id) {
            p.inventory.glimmer = glimmer;
        }
    }

    /// Adds a lit Lantern named `lantern_dev_N` at `(x, y, z)` (every scenario has only one
    /// so far, so Travel has nowhere to go) and returns its index. Not saved.
    pub fn dev_add_lantern(&mut self, x: f32, y: f32, z: f32) -> u32 {
        let index = self.world.layout.lanterns.len();
        let id = format!("lantern_dev_{index}");
        self.world
            .state
            .flags
            .raise(&lf_core::world_state::flags::lantern_lit(&id));
        self.world
            .layout
            .lanterns
            .push(lf_content::scenario::Lantern {
                id,
                pos: [x, y, z],
                yaw: 0.0,
            });
        index as u32
    }

    /// Lights the Lantern at `index` of the scenario without resting there (so Travel has
    /// somewhere to go). Returns false if there is no such Lantern.
    pub fn dev_light_lantern(&mut self, index: u32) -> bool {
        let Some(l) = self.world.layout.lanterns.get(index as usize) else {
            return false;
        };
        let key = lf_core::world_state::flags::lantern_lit(&l.id);
        self.world.state.flags.raise(&key);
        true
    }
}

impl LfWorld {
    /// Runs after every tick: refills the player's health and stamina in god mode.
    pub(crate) fn dev_after_tick(&mut self) {
        if !self.dev.god {
            return;
        }
        if let Some(player) = self.world.actors.get_mut(self.world.player) {
            player.hp = player.max_hp;
            player.stamina = player.max_stamina;
            player.dead = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> LfWorld {
        LfWorld::new_world(1, "test_field").unwrap_or_else(|_| panic!("test_field should load"))
    }

    #[test]
    fn teleport_moves_the_player_and_the_snapshot() {
        let mut w = world();
        assert!(w.dev_teleport(0.0, 9.0, 28.0, 1.0));
        let p = w.world.actors.get(w.world.player).unwrap();
        assert_eq!(p.pos, [0.0, 9.0, 28.0]);
        assert_eq!(w.snapshot[0].pos, [0.0, 9.0, 28.0]);
        assert!(!w.dev_teleport(f32::NAN, 0.0, 0.0, 0.0));
    }

    #[test]
    fn god_mode_refills_health_and_level_clamps() {
        let mut w = world();
        w.dev_set_god(true);
        let id = w.world.player;
        w.world.actors.get_mut(id).unwrap().hp = 1;
        w.dev_after_tick();
        let p = w.world.actors.get(id).unwrap();
        assert_eq!(p.hp, p.max_hp);
        assert_eq!(w.dev_set_level(0), 1);
        assert_eq!(w.dev_set_level(10_000), MAX_DEV_LEVEL);
    }
}
