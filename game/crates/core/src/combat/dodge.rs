//! Dodging: equip load tiers, the roll and backstep for a tier, and i-frames. See
//! docs/game/03-combat.md, "Combat verbs" (Roll, Backstep) and "Equip load and roll
//! tiers".
//!
//! The dodge moves are data (`data/moves/common.ron`); a weapon's moveset names them.
//! The i-frame window is the move's `iframes`, which `combat::moves::step` turns into the
//! [`IFRAMES`](crate::combat::moves::flags::IFRAMES) flag for the frame executed this
//! tick. [`is_invulnerable`] reads it, and `guard::apply_incoming_hit` skips a hit
//! while it holds.

use lf_content::locomotion::EquipLoad;
use serde::{Deserialize, Serialize};

use crate::combat::moves::{MoveNum, MoveState, Moveset, flags};

/// How loaded an actor is: its equip load as a share of its maximum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadTier {
    Light,
    Medium,
    Heavy,
    /// Over the maximum: walk only. No roll, no backstep, no sprint.
    Overloaded,
}

/// The tier for `load_tenths` of `max_tenths` (both in tenths of a unit). Under
/// `light_below_percent` is Light, up to but not including `heavy_from_percent` Medium,
/// up to and including `overloaded_over_percent` Heavy, and over that Overloaded.
pub fn load_tier(load_tenths: u32, max_tenths: u32, t: &EquipLoad) -> LoadTier {
    // Percent comparisons in integers: `load * 100 < pct * max`.
    let scaled = u64::from(load_tenths) * 100;
    let below = |pct: u16| scaled < u64::from(pct) * u64::from(max_tenths);
    if below(t.light_below_percent) {
        LoadTier::Light
    } else if below(t.heavy_from_percent) {
        LoadTier::Medium
    } else if scaled <= u64::from(t.overloaded_over_percent) * u64::from(max_tenths) {
        LoadTier::Heavy
    } else {
        LoadTier::Overloaded
    }
}

/// The move a `DODGE` tap starts: the tier's roll when the stick points somewhere, the
/// backstep when it doesn't. `None` when overloaded.
pub fn select(set: &Moveset, tier: LoadTier, has_direction: bool) -> Option<MoveNum> {
    if tier == LoadTier::Overloaded {
        None
    } else if !has_direction {
        Some(set.dodge_backstep)
    } else {
        Some(match tier {
            LoadTier::Light => set.roll_light,
            LoadTier::Heavy => set.roll_heavy,
            LoadTier::Medium | LoadTier::Overloaded => set.roll_medium,
        })
    }
}

/// True for any roll of the moveset, any tier.
pub fn is_roll(set: &Moveset, num: MoveNum) -> bool {
    [set.roll_light, set.roll_medium, set.roll_heavy].contains(&num)
}

/// True while the frame executed this tick is inside the move's i-frames: hits don't land.
pub fn is_invulnerable(m: &MoveState) -> bool {
    m.has(flags::IFRAMES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::Actor;
    use crate::combat::guard::{HitOutcome, IncomingHit, apply_incoming_hit};
    use crate::input::{ActionFrame, TickInput, buttons};
    use crate::tick::tick;
    use crate::world::World;
    use lf_content::Content;

    fn flat() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(5, &content, "m0_flat").expect("m0_flat exists")
    }

    fn tick_with(w: &mut World, buttons: u16, x: i8, z: i8) {
        let frame = ActionFrame {
            move_x: x,
            move_z: z,
            buttons,
        };
        tick(
            w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
    }

    fn player(w: &mut World) -> &mut Actor {
        w.actors.get_mut(w.player).unwrap()
    }

    const HIT: IncomingHit = IncomingHit {
        damage: 100,
        poise_damage: 12,
        elemental: false,
        unblockable: false,
    };

    /// Taps `DODGE` with the stick forward and runs until the roll's frame 0 executes.
    fn start_roll(w: &mut World) {
        tick_with(w, buttons::DODGE, 0, 127);
        tick_with(w, 0, 0, 127);
        let c = player(w).moves.current.expect("the roll started");
        assert_eq!(c.frame, 0);
    }

    /// Applies a hit to the player on the frame executed this tick.
    fn hit_now(w: &mut World) -> HitOutcome {
        let t = w.locomotion.guard;
        let stats = w.moves.player_moveset.as_ref().unwrap().guard;
        apply_incoming_hit(player(w), HIT, &stats, &t)
    }

    /// Runs a roll from frame 0 and returns the outcome of a hit on each frame, as
    /// `(frame, connected)`, up to `frames`.
    fn hits_by_frame(w: &mut World, frames: u16) -> Vec<(u16, bool)> {
        start_roll(w);
        let mut out = Vec::new();
        for f in 0..frames {
            assert_eq!(player(w).moves.current.map(|c| c.frame), Some(f));
            let mut probe = w.clone();
            let connected = matches!(hit_now(&mut probe), HitOutcome::Connected { .. });
            out.push((f, connected));
            tick_with(w, 0, 0, 0);
        }
        out
    }

    #[test]
    fn a_hit_during_roll_frames_2_to_14_does_nothing_and_frame_15_connects() {
        let mut w = flat();
        // The player is Medium tier: 14 stamina, i-frames 2-14, 24 frames.
        assert_eq!(
            w.moves.player_moveset.as_ref().unwrap().load_tier,
            LoadTier::Medium
        );
        for (f, connected) in hits_by_frame(&mut w, 24) {
            assert_eq!(connected, !(2..=14).contains(&f), "roll frame {f}");
        }
        // Spelled out for the two edges in the acceptance criterion.
        let mut w = flat();
        start_roll(&mut w);
        tick_with(&mut w, 0, 0, 0);
        tick_with(&mut w, 0, 0, 0); // frame 2
        assert_eq!(hit_now(&mut w), HitOutcome::Ignored);
        for _ in 3..=14 {
            tick_with(&mut w, 0, 0, 0);
        }
        assert_eq!(player(&mut w).moves.current.unwrap().frame, 14);
        assert_eq!(hit_now(&mut w), HitOutcome::Ignored);
        tick_with(&mut w, 0, 0, 0);
        assert_eq!(player(&mut w).moves.current.unwrap().frame, 15);
        assert_eq!(hit_now(&mut w), HitOutcome::Connected { damage: 100 });
    }

    #[test]
    fn tiers_set_roll_length_and_iframes() {
        for (tier, name, total, iframes) in [
            (LoadTier::Light, "dodge_roll_light", 20, (2, 14)),
            (LoadTier::Medium, "dodge_roll_medium", 24, (2, 14)),
            (LoadTier::Heavy, "dodge_roll_heavy", 30, (2, 13)),
        ] {
            let mut w = flat();
            w.moves.player_moveset.as_mut().unwrap().load_tier = tier;
            let want = w.moves.num_of(name).unwrap();
            start_roll(&mut w);
            assert_eq!(player(&mut w).moves.current.unwrap().num, want);
            let d = &w.moves.get(want).unwrap().data;
            assert_eq!(d.total_frames, total, "{name}");
            let w_iframes = d.iframes.unwrap();
            assert_eq!((w_iframes.from, w_iframes.to), iframes, "{name}");
            assert_eq!(d.stamina, 14, "{name}");
            let dist = d.root_motion.last().unwrap().1;
            assert!(
                (dist - [3.4, 3.2, 2.4][tier as usize]).abs() < 1e-6,
                "{name}"
            );
            // The flag follows the window frame by frame.
            for f in 0..total {
                let on = player(&mut w).moves.has(flags::IFRAMES);
                assert_eq!(on, (iframes.0..=iframes.1).contains(&f), "{name} frame {f}");
                tick_with(&mut w, 0, 0, 0);
            }
            assert!(player(&mut w).moves.current.is_none(), "{name} ends");
        }
    }

    #[test]
    fn tier_thresholds() {
        let t = lf_content::EquipLoad {
            light_below_percent: 30,
            heavy_from_percent: 70,
            overloaded_over_percent: 100,
            heavy_regen_percent: 80,
        };
        let tier = |load| load_tier(load, 1000, &t);
        assert_eq!(tier(0), LoadTier::Light);
        assert_eq!(tier(299), LoadTier::Light);
        assert_eq!(tier(300), LoadTier::Medium);
        assert_eq!(tier(699), LoadTier::Medium);
        assert_eq!(tier(700), LoadTier::Heavy);
        assert_eq!(tier(1000), LoadTier::Heavy);
        assert_eq!(tier(1001), LoadTier::Overloaded);
        assert_eq!(load_tier(1, 0, &t), LoadTier::Overloaded);
    }

    #[test]
    fn overloaded_cannot_roll_backstep_or_sprint() {
        let mut w = flat();
        w.moves.player_moveset.as_mut().unwrap().load_tier = LoadTier::Overloaded;
        let set = w.moves.player_moveset.clone().unwrap();
        assert_eq!(select(&set, LoadTier::Overloaded, true), None);
        assert_eq!(select(&set, LoadTier::Overloaded, false), None);
        tick_with(&mut w, buttons::DODGE, 0, 127);
        tick_with(&mut w, 0, 0, 0);
        tick_with(&mut w, 0, 0, 0);
        assert!(player(&mut w).moves.current.is_none());
        let max = player(&mut w).max_stamina;
        assert_eq!(player(&mut w).stamina, max);
        let mut w = flat();
        w.moves.player_moveset.as_mut().unwrap().load_tier = LoadTier::Overloaded;
        for _ in 0..30 {
            tick_with(&mut w, buttons::DODGE, 0, 127);
        }
        assert!(!player(&mut w).motor.sprinting);
    }

    #[test]
    fn roll_and_backstep_cost_stamina() {
        let mut w = flat();
        let max = player(&mut w).max_stamina;
        start_roll(&mut w);
        assert_eq!(player(&mut w).stamina, max - 14.0);
        let mut w = flat();
        tick_with(&mut w, buttons::DODGE, 0, 0);
        tick_with(&mut w, 0, 0, 0);
        assert_eq!(player(&mut w).stamina, max - 10.0);
    }
}
